use dynfmt::{Format, SimpleCurlyFormat};
use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use structopt::StructOpt;
use structopt::clap;
use zzp_tools::invoice::InvoiceFile;

use zzp::gregorian::Date;
use zzp_tools::{CustomerConfig, ZzpConfig};

#[derive(StructOpt)]
#[structopt(setting = clap::AppSettings::DeriveDisplayOrder)]
#[structopt(setting = clap::AppSettings::UnifiedHelpMessage)]
#[structopt(setting = clap::AppSettings::ColoredHelp)]
pub struct Options {
	#[structopt(long, short)]
	#[structopt(parse(from_occurrences))]
	#[structopt(global = true)]
	verbose: i8,

	/// The invoice number to use.
	#[structopt(long)]
	number: String,

	/// The file with hour log entries.
	#[structopt(long, short)]
	#[structopt(value_name = "FILE")]
	input: PathBuf,

	/// Write the generated invoice to this path instead of the default.
	#[structopt(long, short)]
	#[structopt(value_name = "FILE")]
	output: Option<PathBuf>,

	/// Overwrite the output file if it exists.
	#[structopt(long)]
	overwrite: bool,

	/// The date to use for the invoice instead of today.
	#[structopt(long)]
	#[structopt(value_name = "YYYY-MM-DD")]
	date: Option<Date>,

	/// Do not automatically add the invoice to the grootboek.
	#[structopt(long)]
	skip_grootboek: bool,
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
	// Find configuration files.
	let current_dir = std::env::current_dir()
		.map_err(|e| log::error!("failed to determine working directory: {}", e))?;
	let zzp_config_path = ZzpConfig::find("/", &current_dir)
		.ok_or_else(|| log::error!("could not find zzp.toml"))?;
	let root_dir = zzp_config_path.parent().unwrap();
	let customer_config_path = CustomerConfig::find(root_dir, &current_dir)
		.ok_or_else(|| log::error!("could not find customer.toml"))?;

	// Read configuration files.
	let zzp_config = ZzpConfig::read_file(&zzp_config_path)
		.map_err(|e| log::error!("{}", e))?;
	let customer_config = CustomerConfig::read_file(&customer_config_path)
		.map_err(|e| log::error!("{}", e))?;

	// Consolidate command line options with config files.
	let date = options.date.unwrap_or_else(Date::today);

	let args: std::collections::BTreeMap<_, _> = [
		("year", date.year().to_string()),
		("month", format!("{:02}", date.month().to_number())),
		("day", format!("{:02}", date.day())),
	].into_iter().collect();

	let grootboek_path = SimpleCurlyFormat.format(&zzp_config.grootboek.path, &args)
		.map_err(|e| log::error!("failed to expand grootboek path: {}", e))?;
	let grootboek_path = root_dir.join(&*grootboek_path);
	let grootboek_dir = grootboek_path.parent()
		.ok_or_else(|| log::error!("failed to determine parent directory of {}", grootboek_path.display()))?;

	let invoice_directory = SimpleCurlyFormat.format(&zzp_config.invoice.directory, &args)
		.map_err(|e| log::error!("failed to expand invoice directory: {}", e))?;
	let output = options.output
		.map(|path| current_dir.join(path))
		.unwrap_or_else(|| {
		generate_invoice_file_name(root_dir.join(&*invoice_directory), &options.number, &zzp_config)
	});

	// Read invoice entries.
	let mut invoice: InvoiceFile = zzp_tools::read_toml(&options.input)
		.map_err(|e| log::error!("{e}"))?;
	invoice.entries.sort_by(|a, b| a.date.cmp(&b.date));

	let invoice_tag_value = output.strip_prefix(grootboek_dir)
		.map_err(|_| {
			log::error!("invoice path ({}) is not below the grootboek directory ({})", output.display(), grootboek_dir.display());
		})?
		.display()
		.to_string();

	let quarter;
	if date.month() >= zzp::gregorian::October {
		quarter = 4;
	} else if date.month() >= zzp::gregorian::July {
		quarter = 3;
	} else if date.month() >= zzp::gregorian::April {
		quarter = 2;
	} else {
		quarter = 1;
	}

	let format_args: BTreeMap<_, _> = [
		("year", date.year().to_string()),
		("month", format!("{:02}", date.month().to_number())),
		("day", format!("{:02}", date.day())),
		("quarter", quarter.to_string()),
		("debitor", customer_config.customer.grootboek_name.clone()),
		("invoice_number", options.number.clone()),
	].into_iter().collect();

	let mut total_ex_vat = 0.0;
	let mut total_vat = BTreeMap::new();
	for entry in &invoice.entries {
		total_ex_vat += entry.total_ex_vat().into_inner();
		let vat = total_vat.entry(entry.vat_percentage).or_insert(0.0);
		*vat += entry.total_vat_only().into_inner();
	}

	let total_vat: BTreeMap<_, _> = total_vat.into_iter().map(|(key, value)| {
		let mut format_args = format_args.clone();
		format_args.insert("percentage", key.to_string());

		let key = SimpleCurlyFormat.format(&zzp_config.grootboek.vat_account, format_args)
			.map_err(|e| log::error!("failed to expand VAT account: {}", e))?;
		let value = zzp::grootboek::Cents((value * 100.0).round() as i32);
		Ok((key, value))
	}).collect::<Result<_, _>>()?;

	let total_vat_all = total_vat.values().sum();
	let total_ex_vat = zzp::grootboek::Cents((total_ex_vat * 100.0).round() as i32);

	let description = SimpleCurlyFormat.format(&zzp_config.invoice.grootboek_description, &format_args)
		.map_err(|e| log::error!("failed to expand grootboek description: {}", e))?;
	let debitor_account = SimpleCurlyFormat.format(&zzp_config.grootboek.debitor_account, &format_args)
		.map_err(|e| log::error!("failed to expand debitor account: {}", e))?;
	let revenue_account = SimpleCurlyFormat.format(&zzp_config.grootboek.revenue_account, &format_args)
		.map_err(|e| log::error!("failed to expand revenue account: {}", e))?;

	let mut grootboek_entry = zzp::grootboek::Transaction {
		date,
		description: &description,
		tags: vec![
			zzp::grootboek::Tag {
				label: &zzp_config.invoice.grootboek_tag,
				value: &invoice_tag_value,
			},
		],
		mutations: vec![
			zzp::grootboek::Mutation {
				amount: total_ex_vat + total_vat_all,
				account: zzp::grootboek::Account::from_raw(&debitor_account),
			},
			zzp::grootboek::Mutation {
				amount: -total_ex_vat,
				account: zzp::grootboek::Account::from_raw(&revenue_account),
			},
		],
	};

	for (account, &amount) in &total_vat {
		grootboek_entry.mutations.push(zzp::grootboek::Mutation {
			account: zzp::grootboek::Account::from_raw(account),
			amount: -amount,
		})
	}

	if let Some(parent) = output.parent() {
		std::fs::create_dir_all(parent)
			.map_err(|e| log::error!("failed to create directory {}: {}", parent.display(), e))?;
	}

	let file = std::fs::OpenOptions::new()
		.create(true)
		.truncate(true)
		.create_new(!options.overwrite)
		.write(true)
		.open(&output)
		.map_err(|e| log::error!("failed to create {}: {}", output.display(), e))?;
	let file = std::io::BufWriter::new(file);

	zzp_tools::invoice::make_invoice(
		file,
		&zzp_config,
		&customer_config.customer,
		&options.number,
		date,
		&invoice.entries,
	)
		.map_err(|e| log::error!("{}", e))?;

	zzp_tools::grootboek::print_full_colored(&grootboek_entry);
	if !options.skip_grootboek {
		let mut grootboek_file = std::fs::OpenOptions::new()
			.append(true)
			.create(true)
			.open(&grootboek_path)
			.map_err(|e| log::error!("failed to open {} for writing: {}", grootboek_path.display(), e))?;
		writeln!(grootboek_file)
			.map_err(|e| log::error!("failed to write to {}: {}", grootboek_path.display(), e))?;
		zzp_tools::grootboek::write_full(&mut grootboek_file, &grootboek_entry)
			.map_err(|e| log::error!("failed to write to {}: {}", grootboek_path.display(), e))?;
	}

	Ok(())
}

fn generate_invoice_file_name(invoice_dir: impl AsRef<Path>, number: &str, config: &ZzpConfig) -> PathBuf {
	let mut invoice = config.invoice_localization.invoice.clone();
	unsafe {
		invoice.as_bytes_mut()[0].make_ascii_uppercase();
	}
	invoice_dir.as_ref().join(format!("{company} - {invoice} {number}.pdf",
		company = config.company.name,
		number = number,
		invoice = invoice,
	))
}
