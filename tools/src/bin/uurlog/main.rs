use std::path::{Path, PathBuf};
use yansi::Paint;
use std::fmt::Display;

use zzp::partial_date::PartialDate;
use zzp::uurlog::{Date, Entry, Hours};

mod invoice;

#[derive(clap::Parser)]
struct Options {
	#[clap(long, short)]
	#[clap(action = clap::ArgAction::Count)]
	#[clap(global = true)]
	verbose: u8,

	#[clap(subcommand)]
	command: Command,
}

#[derive(clap::Subcommand)]
enum Command {
	Show(ShowOptions),
	Invoice(invoice::InvoiceOptions),
}

#[derive(clap::Args)]
struct ShowOptions {
	/// The file with hour log entries.
	#[clap(long, short)]
	#[clap(value_name = "FILE")]
	file: PathBuf,

	/// The period to synchronize.
	#[clap(long)]
	#[clap(value_name = "YYYY[-MM[-DD]]")]
	period: Option<PartialDate>,

	/// Only consider hour entries from this date or later.
	#[clap(long)]
	#[clap(value_name = "YEAR[-MONTH[-DAY]]")]
	#[clap(conflicts_with = "period")]
	start_date: Option<PartialDate>,

	/// Only consider hour entries from this date or earlier.
	#[clap(long)]
	#[clap(value_name = "YEAR[-MONTH[-DAY]]")]
	#[clap(conflicts_with = "period")]
	end_date: Option<PartialDate>,
}

fn main() {
	if let Err(()) = do_main(clap::Parser::parse()) {
		std::process::exit(1);
	}
}

fn init_logging(verbosity: i8) {
	let level = if verbosity <= -2 {
		log::LevelFilter::Error
	} else if verbosity == -1 {
		log::LevelFilter::Warn
	} else if verbosity == 0 {
		log::LevelFilter::Info
	} else if verbosity == 1 {
		log::LevelFilter::Debug
	} else {
		log::LevelFilter::Trace
	};

	env_logger::Builder::from_default_env().filter_module(module_path!(), level).init();
}

fn do_main(options: Options) -> Result<(), ()> {
	init_logging(options.verbose.try_into().unwrap_or(i8::MAX));
	match options.command {
		Command::Show(x) => show_entries(x),
		Command::Invoice(x) => invoice::make_invoice(x),
	}
}

fn show_entries(options: ShowOptions) -> Result<(), ()> {
	let mut start_date = options.start_date.map(|x| x.as_start_date());
	let mut end_date = options.end_date.map(|x| x.as_end_date().next());
	if let Some(period) = options.period {
		let range = period.as_range();
		start_date = Some(range.start);
		end_date = Some(range.end);
	};

	let entries = read_uurlog(&options.file, start_date, end_date)?;
	let mut total = Hours::from_minutes(0);
	for entry in entries {
		total += entry.hours;
		println!("{date}, {hours}, {tags}{description}",
			date = Paint::cyan(entry.date),
			hours = Paint::red(entry.hours),
			tags = Paint::yellow(format_iterator(&entry.tags, "[", "] [", "] ")),
			description = entry.description,
		);
	}

	println!();
	println!("{} {}", Paint::default("Total time:").bold(), Paint::yellow(total));
	Ok(())
}

fn read_uurlog(path: &Path, start_date: Option<Date>, end_date: Option<Date>) -> Result<Vec<Entry>, ()> {
	// Read all entries from the hour log.
	let mut entries = zzp::uurlog::parse_file(path)
		.map_err(|e| log::error!("failed to read hour entries from {}: {}", path.display(), e))?;

	// Filter on date.
	if let Some(start_date) = start_date {
		entries.retain(|x| x.date >= start_date);
	}
	if let Some(end_date) = end_date {
		entries.retain(|x| x.date < end_date);
	}

	Ok(entries)
}

fn format_iterator<I, Pre, Sep, Post>(iter: I, pre: Pre, sep: Sep, post: Post) -> FormatIterator<I::IntoIter, Pre, Sep, Post>
where
	I: IntoIterator,
	I::IntoIter: Clone,
	I::Item: std::fmt::Display,
	Pre: Display,
	Sep: Display,
	Post: Display,
{
	let iter = iter.into_iter();
	FormatIterator {
		iter,
		pre,
		sep,
		post,
	}
}

struct FormatIterator<I, Pre, Sep, Post> {
	iter: I,
	pre: Pre,
	sep: Sep,
	post: Post,
}

impl<I, Pre, Sep, Post> Display for FormatIterator<I, Pre, Sep, Post>
where
	I: Iterator + Clone,
	I::Item: Display,
	Pre: Display,
	Sep: Display,
	Post: Display,
{
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{}", self.pre)?;

		for (i, item) in self.iter.clone().enumerate() {
			if i == 0 {
				write!(f, "{}", item)?;
			} else {
				write!(f, "{}{}", self.sep, item)?;
			}
		}

		write!(f, "{}", self.post)
	}
}
