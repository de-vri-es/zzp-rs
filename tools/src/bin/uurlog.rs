use std::path::PathBuf;
use structopt::StructOpt;
use structopt::clap;
use yansi::Paint;
use std::fmt::Display;

use zzp::partial_date::PartialDate;

#[derive(StructOpt)]
#[structopt(setting = clap::AppSettings::DeriveDisplayOrder)]
#[structopt(setting = clap::AppSettings::UnifiedHelpMessage)]
#[structopt(setting = clap::AppSettings::ColoredHelp)]
struct Options {
	#[structopt(long, short)]
	#[structopt(parse(from_occurrences))]
	verbose: i8,

	/// Synchronize logged hours to Paymo.
	file: PathBuf,

	/// The period to synchronize.
	#[structopt(long)]
	#[structopt(value_name = "YYYY[-MM[-DD]]")]
	period: Option<PartialDate>,
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

	let main_module = env!("CARGO_PKG_NAME").replace("-", "_");
	env_logger::from_env("RUST_LOG").filter_module(&main_module, level).init();
}

fn do_main(options: Options) -> Result<(), ()> {
	// Read all entries from the hour log.
	let mut entries = zzp::uurlog::parse_file(&options.file)
		.map_err(|e| log::error!("failed to read {}: {}", options.file.display(), e))?;

	// Filter on date.
	if let Some(period) = options.period {
		let period = period.as_range();
		entries.retain(|x| period.contains(&x.date));
	}

	let mut total = zzp::uurlog::Hours::from_minutes(0);
	for entry in &entries {
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
