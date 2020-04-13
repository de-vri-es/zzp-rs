use structopt::StructOpt;
use structopt::clap::AppSettings;
use std::path::PathBuf;
use grootboek::Date;
use yansi::Paint;

#[derive(StructOpt)]
#[structopt(setting = AppSettings::ColoredHelp)]
#[structopt(setting = AppSettings::UnifiedHelpMessage)]
#[structopt(setting = AppSettings::DeriveDisplayOrder)]
struct Options {
	/// The file to parse.
	file: PathBuf,

	/// Give a summary for one node.
	#[structopt(long, short)]
	#[structopt(value_name = "NODE")]
	node: Option<String>,

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
	Year(i32),
	Month(i32, i32),
	Day(i32, i32, i32),
}

impl PartialDate {
	fn as_start_date(self) -> Date {
		match self {
			Self::Year(y) => Date::from_parts(y, 1, 1).unwrap(),
			Self::Month(y, m) => Date::from_parts(y, m, 1).unwrap(),
			Self::Day(y, m, d) => Date::from_parts(y, m, d).unwrap(),
		}
	}

	fn as_end_date(self) -> Date {
		match self {
			Self::Year(y) => Date::from_parts(y, 1, 1).unwrap().last_day_of_year(),
			Self::Month(y, m) => Date::from_parts(y, m, 1).unwrap().last_day_of_month(),
			Self::Day(y, m, d) => Date::from_parts(y, m, d).unwrap(),
		}
	}

	fn as_inclusive_date_range(self) -> (Date, Date) {
		(self.as_start_date(), self.as_end_date())
	}
}

impl std::str::FromStr for PartialDate {
	type Err = String;

	fn from_str(value: &str) -> Result<Self, String> {
		let value = value.trim();

		let check_date = |y, m, d| Date::from_parts(y, m, d).map_err(|_| format!("invalid date: {}-{}-{}", y, m, d));

		let mut parts = value.trim().splitn(3, '-');
		let year = parts.next().map(|x| x.parse::<i32>().map_err(|_| format!("invalid year: {:?}", x))).unwrap()?;

		let month = match parts.next() {
			None => return Ok(PartialDate::Year(year)),
			Some(x) => x.parse::<i32>().map_err(|_| format!("invalid month: {:?}", x))?,
		};

		check_date(year, month, 1)?;

		let day = match parts.next() {
			None => return Ok(PartialDate::Month(year, month)),
			Some(x) => x.parse::<i32>().map_err(|_| format!("invalid day: {:?}", x))?,
		};

		check_date(year, month, day)?;
		Ok(PartialDate::Day(year, month, day))
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
	let transactions = grootboek::Transaction::parse_from_str(&data).map_err(|e| format!("{}", e))?;
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
		true
	});

	for (i, transaction) in transactions.enumerate() {
		if i > 0 { println!(); }
		print_full(&transaction);
	}
	Ok(())
}

fn main() {
	if let Err(error) = do_main(&Options::from_args()) {
		eprintln!("Error: {}", error);
		std::process::exit(1);
	}
}

fn print_full(transaction: &grootboek::Transaction) {
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
		let color = match mutation.amount.is_negative() {
			true => yansi::Color::Red.style(),
			false => yansi::Color::Green.style(),
		};
		println!("{amount} {account}",
			amount  = color.paint(mutation.amount),
			account = mutation.account,
		);
	}
}
