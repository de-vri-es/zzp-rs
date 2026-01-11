use std::path::PathBuf;
use yansi::Paint;

use zzp::partial_date::PartialDate;
use zzp::grootboek::Account;
use zzp::grootboek::Cents;
use zzp::grootboek::Transaction;
use zzp_tools::grootboek::color_cents;

#[derive(clap::Parser)]
struct Options {
	/// The file to parse.
	file: PathBuf,

	/// Consider only transactions that mutate the given account or a sub-account.
	#[clap(long, short)]
	#[clap(value_name = "ACCOUNT")]
	account: Option<String>,

	/// Check for unbalanced transactions.
	#[clap(long, short)]
	check: bool,

	/// Limit records to this period.
	#[clap(long)]
	#[clap(value_name = "YEAR[-MONTH[-DAY]]")]
	period: Option<PartialDate>,

	/// Only consider records from this date or later.
	#[clap(long)]
	#[clap(value_name = "YEAR[-MONTH[-DAY]]")]
	#[clap(conflicts_with = "period")]
	start_date: Option<PartialDate>,

	/// Only consider records from this date or earlier.
	#[clap(long)]
	#[clap(value_name = "YEAR[-MONTH[-DAY]]")]
	#[clap(conflicts_with = "period")]
	end_date: Option<PartialDate>,

	/// Change the output format.
	#[clap(long, short)]
	#[clap(default_value = "tree")]
	format: Format,

	/// Do not print empty accounts (with a balance of 0.00).
	#[clap(long, short)]
	skip_empty_accounts: bool,
}

#[derive(Copy, Clone, Eq, PartialEq, clap::ValueEnum)]
enum Format {
	Tree,
	Raw,
}

fn do_main(options: &Options) -> Result<(), String> {
	let mut start_date = options.start_date.map(|x| x.as_start_date());
	let mut end_date = options.end_date.map(|x| x.as_end_date().next());
	if let Some(period) = options.period {
		let range = period.as_range();
		start_date = Some(range.start);
		end_date = Some(range.end);
	};

	let data = std::fs::read_to_string(&options.file).map_err(|e| format!("failed to read {:?}: {}", options.file, e))?;
	let transactions = Transaction::parse_from_str(&data).map_err(|e| format!("{}", e))?;
	let transactions = transactions.into_iter().filter(|transaction| {
		if let Some(start_date) = &start_date && transaction.date < *start_date {
			return false;
		}
		if let Some(end_date) = &end_date && transaction.date >= *end_date {
			return false;
		}
		if let Some(account) = &options.account && !transaction.mutates_account(account) {
			return false;
		}
		true
	});

	if options.check {
		let mut unbalanced_transactions = 0;
		for (transaction, balance) in find_unbalanced(transactions) {
			zzp_tools::grootboek::print_full_colored(&transaction);
			println!("{prefix} {balance}",
				prefix = Paint::red("Unbalanced amount:").bold(),
				balance = color_cents(balance),
			);
			unbalanced_transactions += 1;
			println!()
		}

		if unbalanced_transactions != 0 {
			Err(format!("Found {} unbalanced transactions.", unbalanced_transactions))
		} else {
			Ok(())
		}

	} else {
		let totals = compute_totals(transactions);
		print_totals(&totals, options.format, options.skip_empty_accounts);
		Ok(())
	}

	// for (i, transaction) in transactions.enumerate() {
	// 	if i > 0 { println!(); }
	// 	print_full(&transaction);
	// }
}

fn main() {
	if let Err(error) = do_main(&clap::Parser::parse()) {
		eprintln!("Error: {}", error);
		std::process::exit(1);
	}
}

struct Tree<'a> {
	root: Node<'a>,
}

struct Node<'a> {
	account: Account<'a>,
	balance: Cents,
	cumulative_balance: Cents,
	children: Vec<Node<'a>>,
}

impl<'a> Tree<'a> {
	fn new() -> Self {
		Self {
			root: Node::new(Account::from_raw("")),
		}
	}

	fn add_balance(&mut self, account: Account<'a>, amount: Cents) {
		self.root.cumulative_balance += amount;
		let mut current = &mut self.root;
		for node in account.walk_nodes() {
			if let Some(x) = current.children.iter().position(|x| x.account == node) {
				current = &mut current.children[x];
			} else {
				current.children.push(Node::new(node));
				current = current.children.last_mut().unwrap();
			}
			current.cumulative_balance += amount;
			if node == account {
				current.balance += amount;
			}
		}
	}
}

impl<'a> Node<'a> {
	fn new(account: Account<'a>) -> Self {
		Self {
			account,
			balance: Cents(0),
			cumulative_balance: Cents(0),
			children: Vec::new() }
	}
}

fn compute_totals<'a>(transactions: impl IntoIterator<Item = Transaction<'a>>) -> Tree<'a> {
	let mut root = Tree::new();

	for transaction in transactions {
		for mutation in &transaction.mutations {
			root.add_balance(mutation.account, mutation.amount);
		}
	}

	root
}

fn find_unbalanced<'a>(transactions: impl IntoIterator<Item = Transaction<'a>>) -> impl Iterator<Item = (Transaction<'a>, Cents)> {
	transactions.into_iter().filter_map(|transaction| {
		let balance = transaction.mutations.iter().fold(Cents(0), |sum, mutation| sum + mutation.amount);
		if balance != Cents(0) {
			Some((transaction, balance))
		} else {
			None
		}
	})
}

fn print_totals(totals: &Tree, format: Format, skip_empty: bool) {
	if format != Format::Raw {
		println!("Total: {}", color_cents(totals.root.cumulative_balance));
	}
	print_totals_subtree(&totals.root, format, skip_empty, "");
}

fn print_totals_subtree(node: &Node, format: Format, skip_empty: bool, indent: &str) {
	for (i, child) in node.children.iter().enumerate() {
		match format {
			Format::Tree => {
				let (tree_char, subindent) = if i == node.children.len() - 1 {
					("└─", "   ")
				} else {
					("├─", "│  ")
				};
				println!("{}{} {}: {}", indent, tree_char, child.account.name(), color_cents(child.cumulative_balance));
				print_totals_subtree(child, format, skip_empty, &format!("{}{}", indent, subindent));
			},
			Format::Raw => {
				if !skip_empty || child.balance != Cents(0) {
					println!("{} {}", color_cents(child.balance), child.account.raw);
				}
				print_totals_subtree(child, format, skip_empty, "");
			}
		}
	}
}
