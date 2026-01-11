use dynfmt::{Format, SimpleCurlyFormat};
use ordered_float::NotNan;
use zzp_tools::invoice::InvoiceFile;
use std::collections::{btree_map, BTreeMap};
use std::io::Write;
use std::path::{Path, PathBuf};
use structopt::StructOpt;
use structopt::clap;
use super::read_uurlog;

use zzp::gregorian::Date;
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
	#[structopt(group = "period-group")]
	period: Option<PartialDate>,

	/// Only consider hour entries from this date or later.
	#[structopt(long)]
	#[structopt(value_name = "YEAR[-MONTH[-DAY]]")]
	#[structopt(group = "period-group")]
	start_date: Option<PartialDate>,

	/// Only consider hour entries from this date or earlier.
	#[structopt(long)]
	#[structopt(value_name = "YEAR[-MONTH[-DAY]]")]
	#[structopt(conflicts_with = "period")]
	#[structopt(requires = "start-date")]
	end_date: Option<PartialDate>,

	/// The invoice number to use.
	#[structopt(long)]
	number: String,

	/// The file with hour log entries.
	#[structopt(long, short)]
	#[structopt(value_name = "FILE")]
	hours: Option<PathBuf>,

	/// Add extra entries to the invoice from a file.
	#[structopt(long)]
	#[structopt(value_name = "FILE.toml")]
	extra_entries: Option<PathBuf>,

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
	///
	/// Note that entries with tags will be excluded from the summary.
	#[structopt(long)]
	#[structopt(group = "summarize")]
	summarize_days: Option<String>,

	/// Add a single invoice entry for the entire invoice with the given summary.
	///
	/// Note that entries with tags will be excluded from the summary.
	#[structopt(long)]
	#[structopt(group = "summarize")]
	summarize_all: Option<String>,

	/// The unit to display for time log entries on the invoice.
	#[structopt(long)]
	unit: Option<String>,

	/// The price per hour.
	#[structopt(long)]
	#[structopt(value_name = "CENTS")]
	price_per_hour: Option<NotNan<f64>>,

	/// The VAT percentage.
	#[structopt(long)]
	#[structopt(value_name = "PERCENTAGE")]
	vat: Option<NotNan<f64>>,

	/// Do not automatically add the invoice to the grootboek.
	#[structopt(long)]
	skip_grootboek: bool,
}

pub(crate) fn make_invoice(options: InvoiceOptions) -> Result<(), ()> {
	let mut start_date = options.start_date.map(|x| x.as_start_date());
	let mut end_date = options.end_date.map(|x| x.as_end_date().next());
	if let Some(period) = options.period {
		let range = period.as_range();
		start_date = Some(range.start);
		end_date = Some(range.end);
	};

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
	let zzp_config = ZzpConfig::read_file(&zzp_config_path)
		.map_err(|e| log::error!("{}", e))?;
	let customer_config = CustomerConfig::read_file(&customer_config_path)
		.map_err(|e| log::error!("{}", e))?;

	// Consolidate command line options with config files.
	let file = options.hours.clone().unwrap_or_else(|| customer_root_dir.join("uurlog"));
	let date = options.date.unwrap_or_else(Date::today);
	let unit = options.unit.as_deref().unwrap_or(&zzp_config.invoice_localization.hours);
	let unit_price = options.price_per_hour.unwrap_or(customer_config.invoice.price_per_hour);
	let vat_percentage = options.vat.unwrap_or(zzp_config.tax.vat);
	let summarize_untagged = if let Some(description) = options.summarize_days {
		Some(zzp_tools::SummerizeConfig {
			description,
			period: zzp_tools::SummerizePeriod::Day,
		})
	} else if let Some(description) = options.summarize_all {
		Some(zzp_tools::SummerizeConfig {
			description,
			period: zzp_tools::SummerizePeriod::Invoice,
		})
	} else {
		customer_config.invoice.summarize.clone()
	};

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

	// Read hour entries.
	let hour_entries = read_uurlog(&file, start_date, end_date)?;

	// Split hour entries on tags that we care about.
	let mut tagged_hour_entries = BTreeMap::new();
	let mut untagged_hour_entries = Vec::new();
	for tag in &customer_config.tag {
		tagged_hour_entries.insert(tag.name.as_str(), Vec::new());
	}

	for entry in &hour_entries {
		let mut matched = None;
		for tag in &entry.tags {
			if let Some(tagged_entries) = tagged_hour_entries.get_mut(tag.as_str()) {
				if let Some(old_tag) = &matched {
					log::error!("Multiple important tags found for entry {}: it has both the {} and {} tags", entry.date, old_tag, tag);
					return Err(());
				} else {
					tagged_entries.push(entry.clone());
					matched = Some(tag.as_str());
				}
			}
		}
		if matched.is_none() {
			untagged_hour_entries.push(entry.clone());
		}
	}

	let mut invoice_entries = Vec::new();

	if let Some(path) = options.extra_entries {
		let mut invoice: InvoiceFile = zzp_tools::read_toml(path).map_err(|e| log::error!("{e}"))?;
		invoice_entries.append(&mut invoice.entries)
	}

	let invoice_tag_value = output.strip_prefix(grootboek_dir)
		.map_err(|_| {
			log::error!("invoice path ({}) is not below the grootboek directory ({})", output.display(), grootboek_dir.display());
		})?
		.display()
		.to_string();

	// Summarize entries per day, if requested.
	let untagged_hour_entries = match &summarize_untagged {
		None => untagged_hour_entries.clone(),
		Some(config) => match config.period {
			zzp_tools::SummerizePeriod::Day => summarize_hours_per_day(untagged_hour_entries, &config.description),
			zzp_tools::SummerizePeriod::Invoice => summarize_hours_per_invoice(untagged_hour_entries, &config.description, date).into_iter().collect(),
		}
	};

	invoice_entries.extend(untagged_hour_entries.into_iter().map(|entry| {
		zzp_tools::invoice::InvoiceEntry {
			description: entry.description,
			quantity: NotNan::new(f64::from(entry.hours.total_minutes()) / 60.0).unwrap(),
			unit: unit.to_string(),
			date: entry.date,
			unit_price,
			vat_percentage,
		}
	}));

	for tag in &customer_config.tag {
		let entries = tagged_hour_entries.get(tag.name.as_str()).unwrap();
		let hour_entries = match &tag.summarize {
			None => entries.clone(),
			Some(config) => match config.period {
				zzp_tools::SummerizePeriod::Day => summarize_hours_per_day(entries.clone(), &config.description),
				zzp_tools::SummerizePeriod::Invoice => summarize_hours_per_invoice(entries.clone(), &config.description, date).into_iter().collect(),
			}
		};
		invoice_entries.extend(hour_entries.into_iter().map(|entry| {
			zzp_tools::invoice::InvoiceEntry {
				description: entry.description,
				quantity: NotNan::new(f64::from(entry.hours.total_minutes()) / 60.0).unwrap(),
				unit: unit.to_string(),
				date: entry.date,
				unit_price: tag.price_per_hour.unwrap_or(unit_price),
				vat_percentage: tag.vat.unwrap_or(vat_percentage),
			}
		}));
	}

	invoice_entries.sort_by(|a, b| a.date.cmp(&b.date));

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
	for entry in &invoice_entries {
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
		&invoice_entries,
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

fn summarize_hours_per_invoice<I>(entries: I, description: &str, date: Date) -> Option<zzp::uurlog::Entry>
where
	I: IntoIterator,
	I::Item: std::borrow::Borrow<zzp::uurlog::Entry>,
{
	use std::borrow::Borrow;
	let mut summarized = zzp::uurlog::Entry {
		date,
		hours: zzp::uurlog::Hours::from_minutes(0),
		tags: Vec::new(),
		description: description.into(),
	};
	for entry in entries {
		let entry = entry.borrow();
		summarized.hours += entry.hours;
	}

	if summarized.hours.total_minutes() > 0 {
		Some(summarized)
	} else {
		None
	}
}

fn summarize_hours_per_day<I>(entries: I, description: &str) -> Vec<zzp::uurlog::Entry>
where
	I: IntoIterator,
	I::Item: std::borrow::Borrow<zzp::uurlog::Entry>,
{
	use std::borrow::Borrow;
	let mut entries_per_day = BTreeMap::new();
	for entry in entries {
		let entry = entry.borrow();
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
