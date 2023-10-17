use std::path::{Path, PathBuf};
use structopt::StructOpt;
use structopt::clap;
use yansi::Paint;
use std::fmt::Display;

use zzp::partial_date::PartialDate;
use zzp::uurlog::{Date, Entry, Hours};

mod invoice;

#[derive(StructOpt)]
#[structopt(setting = clap::AppSettings::DeriveDisplayOrder)]
#[structopt(setting = clap::AppSettings::UnifiedHelpMessage)]
#[structopt(setting = clap::AppSettings::ColoredHelp)]
#[structopt(setting = clap::AppSettings::VersionlessSubcommands)]
struct Options {
	#[structopt(long, short)]
	#[structopt(parse(from_occurrences))]
	#[structopt(global = true)]
	verbose: i8,

	#[structopt(subcommand)]
	command: Command,
}

#[derive(StructOpt)]
enum Command {
	Show(ShowOptions),
	Invoice(invoice::InvoiceOptions),
}

#[derive(StructOpt)]
#[structopt(setting = clap::AppSettings::DeriveDisplayOrder)]
#[structopt(setting = clap::AppSettings::UnifiedHelpMessage)]
#[structopt(setting = clap::AppSettings::ColoredHelp)]
struct ShowOptions {
	/// The file with hour log entries.
	#[structopt(long, short)]
	#[structopt(value_name = "FILE")]
	file: PathBuf,

	/// The period to synchronize.
	#[structopt(long)]
	#[structopt(value_name = "YYYY[-MM[-DD]]")]
	period: Option<PartialDate>,

	/// Only consider hour entries from this date or later.
	#[structopt(long)]
	#[structopt(value_name = "YEAR[-MONTH[-DAY]]")]
	#[structopt(conflicts_with = "period")]
	start_date: Option<PartialDate>,

	/// Only consider hour entries from this date or earlier.
	#[structopt(long)]
	#[structopt(value_name = "YEAR[-MONTH[-DAY]]")]
	#[structopt(conflicts_with = "period")]
	end_date: Option<PartialDate>,
}

fn main() {
	let options = Options::from_args();
	init_logging(options.verbose);

	if do_main(options).is_err() {
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

	env_logger::from_env("RUST_LOG").filter_module(module_path!(), level).init();
}

fn do_main(options: Options) -> Result<(), ()> {
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
