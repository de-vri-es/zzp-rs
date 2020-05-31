use std::path::PathBuf;
use structopt::StructOpt;
use structopt::clap;

pub mod date;
pub mod hours;
pub mod parse;
pub mod entry;

#[derive(StructOpt)]
#[structopt(setting = clap::AppSettings::DeriveDisplayOrder)]
#[structopt(setting = clap::AppSettings::UnifiedHelpMessage)]
#[structopt(setting = clap::AppSettings::ColoredHelp)]
struct Options {
	file: PathBuf,
}

fn main() {
	if let Err(e) = do_main(Options::from_args()) {
		eprintln!("Error: {}", e);
		std::process::exit(1);
	}
}

fn do_main(options: Options) -> Result<(), String> {
	let data = std::fs::read(&options.file).map_err(|e| format!("failed to read {}: {}", options.file.display(), e))?;

	for (i, line) in data.split(|c| *c == b'\n').enumerate() {
		let line = std::str::from_utf8(line).map_err(|_| format!("invalid UTF-8 on line {}", i))?;
		let line = line.trim();
		if line.is_empty() || line.starts_with('#') {
			continue;
		}

		let entry = entry::Entry::from_str(line).map_err(|e| format!("parse error on line {}: {}", i, e))?;
		println!("{:?}", entry);
	}

	Ok(())
}
