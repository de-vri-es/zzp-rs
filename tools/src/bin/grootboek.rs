use std::path::PathBuf;
use structopt::StructOpt;
use structopt::clap::AppSettings;
use yansi::Paint;

use zzp::gregorian::{Date, Month, Year, YearMonth};
use zzp::grootboek::Account;
use zzp::grootboek::Cents;
use zzp::grootboek::Transaction;

#[derive(StructOpt)]
#[structopt(setting = AppSettings::ColoredHelp)]
#[structopt(setting = AppSettings::UnifiedHelpMessage)]
#[structopt(setting = AppSettings::DeriveDisplayOrder)]
struct Options {
	/// The file to parse.
	file: PathBuf,

	/// Consider only transactions that mutate the given account or a sub-account.
	#[structopt(long, short)]
	#[structopt(value_name = "ACCOUNT")]
	account: Option<String>,

	/// Check for unbalanced transactions.
	#[structopt(long, short)]
	check: bool,

	/// Limit records to this period.
	#[structopt(long)]
	#[structopt(value_name = "YEAR[-MONTH[-DAY]]")]
	period: Option<PartialDate>,

	/// Only consider records from this date or later.
	#[structopt(long)]
	#[structopt(value_name = "YEAR[-MONTH[-DAY]]")]
	#[structopt(conflicts_with = "during")]
	start_date: Option<PartialDate>,

	/// Only consider records from this date or earlier.
	#[structopt(long)]
	#[structopt(value_name = "YEAR[-MONTH[-DAY]]")]
	#[structopt(conflicts_with = "during")]
	end_date: Option<PartialDate>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
enum PartialDate {
	Year(Year),
	Month(YearMonth),
	Day(Date),
}

impl PartialDate {
	fn as_start_date(self) -> Date {
		match self {
			Self::Year(x) => x.first_day(),
			Self::Month(x) => x.first_day(),
			Self::Day(x) => x,
		}
	}

	fn as_end_date(self) -> Date {
		match self {
			Self::Year(x) => x.last_day(),
			Self::Month(x) => x.last_day(),
			Self::Day(x) => x,
		}
	}

	fn as_inclusive_date_range(self) -> (Date, Date) {
		(self.as_start_date(), self.as_end_date())
	}
}

impl std::str::FromStr for PartialDate {
	type Err = String;

	fn from_str(value: &str) -> Result<Self, String> {
		let mut parts = value.trim().splitn(3, '-');

		let year = parts.next().unwrap();
		let year: i16 = year.parse().map_err(|_| format!("invalid year: {:?}", year))?;
		let year = Year::new(year);

		let month: u8 = match parts.next() {
			None => return Ok(PartialDate::Year(year.into())),
			Some(x) => x.parse().map_err(|_| format!("invalid month: {:?}", x))?,
		};

		let month = Month::new(month).map_err(|_| format!("invalid month: {}", month))?;
		let year_month = year.with_month(month);

		let day: u8 = match parts.next() {
			None => return Ok(PartialDate::Month(year_month)),
			Some(x) => x.parse().map_err(|_| format!("invalid day: {:?}", x))?,
		};

		let date = year_month.with_day(day)
			.map_err(|_| format!("invalid date: {}-{}-{}", year, month.to_number(), day))?;
		Ok(PartialDate::Day(date))
	}
}

fn do_main(options: &Options) -> Result<(), String> {
	let mut start_date = options.start_date.map(|x| x.as_start_date());
	let mut end_date = options.end_date.map(|x| x.as_end_date());
	if let Some(period) = options.period {
		let (start, end) = period.as_inclusive_date_range();
		start_date = Some(start);
		end_date = Some(end);
	};

	let data = std::fs::read_to_string(&options.file).map_err(|e| format!("failed to read {:?}: {}", options.file, e))?;
	let transactions = Transaction::parse_from_str(&data).map_err(|e| format!("{}", e))?;
	let transactions = transactions.into_iter().filter(|transaction| {
		if let Some(start_date) = &start_date {
			if transaction.date < *start_date {
				return false;
			}
		}
		if let Some(end_date) = &end_date {
			if transaction.date > *end_date {
				return false;
			}
		}
		if let Some(account) = &options.account {
			if !transaction.mutates_account(account) {
				return false;
			}
		}
		true
	});

	if options.check {
		let mut unbalanced_transactions = 0;
		for (transaction, balance) in find_unbalanced(transactions) {
			print_full(&transaction);
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
		print_totals(&totals);
		Ok(())
	}

	// for (i, transaction) in transactions.enumerate() {
	// 	if i > 0 { println!(); }
	// 	print_full(&transaction);
	// }
}

fn main() {
	if let Err(error) = do_main(&Options::from_args()) {
		eprintln!("Error: {}", error);
		std::process::exit(1);
	}
}

fn color_cents(cents: Cents) -> yansi::Paint<Cents> {
	if cents.total_cents() > 0 {
		yansi::Color::Green.style().paint(cents)
	} else if cents.total_cents() < 0 {
		yansi::Color::Red.style().paint(cents)
	} else {
		yansi::Color::Fixed(241).paint(cents)
	}
}

fn print_full(transaction: &Transaction) {
	println!("{date}: {desc}",
		date = Paint::cyan(transaction.date),
		desc = Paint::magenta(transaction.description),
	);
	for tag in &transaction.tags {
		println!("{label}: {value}",
			label = Paint::cyan(tag.label),
			value = Paint::cyan(tag.value),
		);
	}
	for mutation in &transaction.mutations {
		println!("{amount} {account}",
			amount  = color_cents(mutation.amount),
			account = mutation.account,
		);
	}
}

struct Tree<'a, T> {
	root: Node<'a, T>,
}

struct Node<'a, T> {
	account: Account<'a>,
	data: T,
	children: Vec<Node<'a, T>>,
}

impl<'a, T> Tree<'a, T> {
	fn new(root_data: T) -> Self {
		Self {
			root: Node::new(Account::from_raw(""), root_data),
		}
	}

	fn insert(&mut self, account: Account<'a>, update: impl Fn(&mut T), initial_data: T)
	where
		T: Clone
	{
		update(&mut self.root.data);
		let mut current = &mut self.root;
		for account in account.walk_nodes() {
			if let Some(x) = current.children.iter().position(|x| x.account == account) {
				current = &mut current.children[x];
			} else {
				current.children.push(Node::new(account, initial_data.clone()));
				current = current.children.last_mut().unwrap();
			}
			update(&mut current.data);
		}
	}
}

impl<'a, T> Node<'a, T> {
	fn new(account: Account<'a>, data: T) -> Self {
		Self { account, data, children: Vec::new() }
	}
}

fn compute_totals<'a>(transactions: impl IntoIterator<Item = Transaction<'a>>) -> Tree<'a, Cents> {
	let mut root = Tree::new(Cents(0));

	for transaction in transactions {
		for mutation in &transaction.mutations {
			root.insert(mutation.account, |x| *x += mutation.amount, Cents(0));
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

fn print_totals(totals: &Tree<Cents>) {
	println!("Total: {}", color_cents(totals.root.data));
	print_totals_subtree(&totals.root, "");
}

fn print_totals_subtree(node: &Node<Cents>, indent: &str) {
	for (i, child) in node.children.iter().enumerate() {
		let (tree_char, subindent) = if i == node.children.len() - 1 {
			("└─", "   ")
		} else {
			("├─", "│  ")
		};

		println!("{}{} {}: {}", indent, tree_char, child.account.name(), color_cents(child.data));
		print_totals_subtree(child, &format!("{}{}", indent, subindent));
	}
}
