use structopt::StructOpt;
use structopt::clap::AppSettings;
use std::path::PathBuf;

#[derive(StructOpt)]
#[structopt(setting = AppSettings::ColoredHelp)]
#[structopt(setting = AppSettings::UnifiedHelpMessage)]
#[structopt(setting = AppSettings::DeriveDisplayOrder)]
struct Options {
	/// The file to parse.
	file: PathBuf,
}

fn do_main(options: &Options) -> Result<(), String> {
	let data = std::fs::read_to_string(&options.file).map_err(|e| format!("failed to read {:?}: {}", options.file, e))?;
	let transactions = grootboek::Transaction::parse_from_str(&data).map_err(|e| format!("{:?}", e))?;
	eprintln!("{:#?}", transactions);
	Ok(())
}

fn main() {
	if let Err(error) = do_main(&Options::from_args()) {
		eprintln!("Error: {}", error);
		std::process::exit(1);
	}
}
