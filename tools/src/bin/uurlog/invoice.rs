use std::path::{Path, PathBuf};
use structopt::StructOpt;
use structopt::clap;
use super::read_uurlog;

use zzp::gregorian::{Date, Month};
use zzp::partial_date::PartialDate;
use zzp_tools::{CustomerConfig, ZzpConfig};

#[derive(StructOpt)]
#[structopt(setting = clap::AppSettings::DeriveDisplayOrder)]
#[structopt(setting = clap::AppSettings::UnifiedHelpMessage)]
#[structopt(setting = clap::AppSettings::ColoredHelp)]
pub struct InvoiceOptions {
	/// The period to create an invoice for.
	#[structopt(long)]
	#[structopt(value_name = "YYYY[-MM[-DD]]")]
	period: PartialDate,

	/// The invoice number to use.
	#[structopt(long)]
	number: String,

	/// The file with hour log entries.
	#[structopt(long, short)]
	#[structopt(value_name = "FILE")]
	hours: Option<PathBuf>,

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

	/// Add a single invoice entry per day with the given summary.
	#[structopt(long)]
	summarize_days: Option<String>,

	/// The unit to display for time log entries on the invoice.
	#[structopt(long)]
	unit: Option<String>,

	/// The price per hour in cents.
	#[structopt(long)]
	#[structopt(value_name = "CENTS")]
	price_per_hour: Option<u32>,

	/// The VAT percentage.
	#[structopt(long)]
	#[structopt(value_name = "PERCENTAGE")]
	vat: Option<u32>,
}

pub(crate) fn make_invoice(options: InvoiceOptions) -> Result<(), ()> {
	// Find configuration files.
	let current_dir = std::env::current_dir()
		.map_err(|e| log::error!("failed to determine working directory: {}", e))?;
	let zzp_config_path = ZzpConfig::find("/", &current_dir)
		.ok_or_else(|| log::error!("could not find zzp.toml"))?;
	let root_dir = zzp_config_path.parent().unwrap();
	let customer_config_path = CustomerConfig::find(root_dir, &current_dir)
		.ok_or_else(|| log::error!("could not find customer.toml"))?;
	let customer_root_dir = customer_config_path.parent().unwrap();

	// Read configuration files.
	let zzp_config = ZzpConfig::read_file(zzp_config_path)
		.map_err(|e| log::error!("{}", e))?;
	let customer_config = CustomerConfig::read_file(&customer_config_path)
		.map_err(|e| log::error!("{}", e))?;

	// Consolidate command line options with config files.
	let file = options.hours.clone().unwrap_or_else(|| customer_root_dir.join("uurlog"));
	let date = options.date.unwrap_or_else(Date::today);
	let unit = options.unit.as_deref().unwrap_or(&zzp_config.localization.hours);
	let unit_price = options.price_per_hour.unwrap_or(customer_config.invoice.cents_per_hour);
	let vat = options.vat.unwrap_or(zzp_config.tax.vat);
	let template_path = customer_root_dir.join(customer_config.invoice.template);
	let summarize_days = options.summarize_days
		.as_deref()
		.or(customer_config.invoice.summarize_per_day.as_deref());
	let output = options.output.clone().unwrap_or_else(|| {
		generate_invoice_file_name(&customer_root_dir, &options.number, &zzp_config, template_path.extension())
	});

	// Read hour entries.
	let entries = read_uurlog(&file, Some(options.period))?;

	// Parse template file.
	let template = std::fs::read_to_string(&template_path)
		.map_err(|e| log::error!("failed to read {}: {}", template_path.display(), e))?;
	let template = liquid::ParserBuilder::with_stdlib()
		.build()
		.map_err(|e| log::error!("failed to initialize template parser: {}", e))?
		.parse(&template)
		.map_err(|e| log::error!("failed to parse template {}: {}", template_path.display(), e))?;

	// Summarize entries per day, if requested.
	let entries = if let Some(description) = summarize_days {
		summarize_hours_per_day(&entries, description)
	} else {
		entries
	};

	// Create context for template.
	let mut total_price = 0;
	let entries: Vec<_> = entries.iter().map(|entry| {
		let hours = entry.hours.total_minutes() as f64 / 60.0;
		let entry_price = (unit_price as f64 * hours).round() as u32;
		total_price += entry_price;
		InvoiceEntry {
			date: format_date(entry.date, &zzp_config.localization),
			description: entry.description.clone(),
			amount: format!("{}:{:02} {}", entry.hours.hours(), entry.hours.minutes(), unit),
			unit_price: format_cents(unit_price),
			total_price: format_cents(entry_price),
			vat_percentage: vat.to_string(),
		}
	}).collect();

	let total_vat = (total_price as f64 * vat as f64 / 100.0).round() as u32;
	let context = InvoiceContext {
		company: zzp_config.company,
		customer: customer_config.customer,
		date: format_date(date, &zzp_config.localization),
		invoice_number: options.number,
		total_without_vat: format_cents(total_price),
		total_vat: format_cents(total_vat),
		total_with_vat: format_cents(total_price + total_vat),
		entries,
	};

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
	let mut file = std::io::BufWriter::new(file);

	template.render_to(&mut file, &liquid::to_object(&context).unwrap())
		.map_err(|e| log::error!("failed to write invoice to {}: {}", output.display(), e))?;
	Ok(())
}

fn summarize_hours_per_day(entries: &[zzp::uurlog::Entry], description: &str) -> Vec<zzp::uurlog::Entry> {
	use std::collections::{BTreeMap, btree_map};

	let mut entries_per_day = BTreeMap::new();
	for entry in entries {
		match entries_per_day.entry(entry.date) {
			btree_map::Entry::Vacant(x) => {
				x.insert(entry.hours);
			},
			btree_map::Entry::Occupied(mut x) => {
				*x.get_mut() += entry.hours;
			}
		}
	}

	entries_per_day.into_iter().map(|(date, hours)| {
		zzp::uurlog::Entry {
			date,
			hours,
			tags: Vec::new(),
			description: description.to_owned(),
		}
	}).collect()
}

fn generate_invoice_file_name(customer_root_dir: &Path, number: &str, config: &ZzpConfig, extension: Option<&std::ffi::OsStr>) -> PathBuf {
	let mut invoice = config.localization.invoice.clone();
	unsafe {
		invoice.as_bytes_mut()[0].make_ascii_uppercase();
	}
	let path = customer_root_dir.join(format!("invoices/{number}/{company} - {invoice} {number}",
		company = config.company.name,
		number = number,
		invoice = invoice,
	));
	if let Some(extension) = extension {
		let mut path = path.into_os_string();
		path.push(".");
		path.push(extension);
		path.into()
	} else {
		path
	}
}

fn format_cents(cents: u32) -> String {
	format!("{}.{:02}", cents / 100, cents % 100)
}

fn format_date(date: Date, localization: &zzp_tools::Localization) -> String {
	let month = format_month(date.month(), localization);
	format!("{} {} {}", date.day(), month, date.year())
}

fn format_month(month: Month, localization: &zzp_tools::Localization) -> &str {
	match month {
		Month::January => &localization.january,
		Month::February => &localization.february,
		Month::March => &localization.march,
		Month::April => &localization.april,
		Month::May => &localization.may,
		Month::June => &localization.june,
		Month::July => &localization.july,
		Month::August => &localization.august,
		Month::September => &localization.september,
		Month::October => &localization.october,
		Month::November => &localization.november,
		Month::December => &localization.december,
	}
}

#[derive(serde::Serialize)]
struct InvoiceContext {
	company: zzp_tools::Company,
	customer: zzp_tools::Customer,
	date: String,
	invoice_number: String,
	total_without_vat: String,
	total_vat: String,
	total_with_vat: String,
	entries: Vec<InvoiceEntry>,
}

#[derive(serde::Serialize)]
struct InvoiceEntry {
	date: String,
	description: String,
	amount: String,
	unit_price: String,
	total_price: String,
	vat_percentage: String,
}
