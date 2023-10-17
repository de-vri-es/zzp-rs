use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use ordered_float::NotNan;

pub mod invoice;
pub mod grootboek;

/// Main configuration file for the ZZP tools.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "PascalCase")]
pub struct ZzpConfig {
	/// The company details.
	pub company: Company,

	/// Details regarding the grootboek.
	pub grootboek: GrootboekConfig,

	/// The tax details.
	pub tax: Tax,

	/// Cosmetic invoice options.
	pub invoice: Invoice,

	/// Invoice localization details.
	pub invoice_localization: InvoiceLocalization,

	/// Date localization details.
	pub date_localization: DateLocalization,
}

/// Configuration file for specific customers.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "PascalCase")]
pub struct CustomerConfig {
	/// Details about the customer itself.
	pub customer: Customer,

	/// Details on how to invoice the customer.
	pub invoice: CustomerInvoice,

	/// Details on tags for hour entries related to invoicing.
	#[serde(default = "Vec::new", skip_serializing_if = "Vec::is_empty")]
	pub tag: Vec<TagConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Company {
	/// The name of the company.
	pub name: String,

	/// The address details of the company.
	pub address: Vec<String>,

	/// Contact details as (key, value) pairs.
	pub contact: Vec<KeyValue>,

	/// Legal details as (key, value) pairs such as Chamber of Commerce number and VAT number.
	pub legal: Vec<KeyValue>,

	/// Payment details such as IBAN and BIC.
	pub payment: Vec<KeyValue>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GrootboekConfig {
	/// The path to the grootboek file.
	pub path: String,

	/// The grootboek account to put revenue on.
	pub revenue_account: String,

	/// The grootboek account to put debts from debitors on.
	pub debitor_account: String,

	/// The grootboek account to put debts to creditors on.
	pub creditor_account: String,

	/// The grootboek account to put VAT debts on.
	pub vat_account: String,

	/// The grootboek account to put paid VAT input tax on.
	///
	/// The VAT input tax is paid when you purchase goods and services,
	/// and can be deducated from the VAT debt.
	pub vat_input_account: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Tax {
	/// Default VAT percentage for delivered goods/services.
	pub vat: NotNan<f64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Invoice {
	/// The font to use for generated invoices.
	pub font: String,

	/// The base font size to use for generated invoices.
	pub font_size: NotNan<f64>,

	/// The directory to save invoices.
	pub directory: String,

	/// The description to use for the generated grootboek transaction.
	pub grootboek_description: String,

	/// The tag to use to link the invoice file to a transaction.
	pub grootboek_tag: String,
}

/// Customer details.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Customer {
	pub name: String,
	pub address: Vec<String>,
	pub grootboek_name: String,
}

/// Details on how to invoice a customer.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CustomerInvoice {
	/// The price per hour in money units (euro, yen, dollar, ...).
	pub price_per_hour: NotNan<f64>,

	/// Summarize all hours per day with a single entry.
	pub summarize_per_day: Option<String>,
}

	/// Details on tags for hour entries related to invoicing.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TagConfig {
	/// Thename of the tag.
	pub name: String,

	/// The price per hour in money units (euro, yen, dollar, ...).
	pub price_per_hour: Option<NotNan<f64>>,

	/// Summarize all hours per day with a single entry.
	pub summarize_per_day: Option<String>,

	/// VAT percentage for tagged entries.
	pub vat: Option<NotNan<f64>>,
}

/// Localizaton details for invoices.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InvoiceLocalization {
	/// Translation for "Invoice".
	pub invoice: String,
	/// Translations for "To" for the recipient prefix.
	pub to: String,
	/// Translations for "From" for the sender prefix.
	pub from: String,
	/// Translation for "Invoice number".
	pub invoice_number: String,
	/// Translation for "Invoice date".
	pub invoice_date: String,
	/// Translation for "Date".
	pub date: String,
	/// Translation for "Description".
	pub description: String,
	/// Translation for "Quantity".
	pub quantity: String,
	/// Translation for "Price per unit".
	pub entry_unit_price: String,
	/// Translation for "Total price".
	pub entry_total_price: String,
	/// Translation for "VAT".
	pub vat: String,
	/// Translations for "Total without VAT".
	pub total_ex_vat: String,
	/// Translations for "Total VAT".
	pub total_vat: String,
	/// Translation for "Total due".
	pub total_due: String,
	/// Translation for "hours".
	pub hours: String,
	/// The currency symbol.
	pub currency_symbol: String,
	/// The footer asking the recipient to please pay on time.
	pub footer: String,
}

/// Localizaton details for dates.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DateLocalization {
	pub january: String,
	pub february: String,
	pub march: String,
	pub april: String,
	pub may: String,
	pub june: String,
	pub july: String,
	pub august: String,
	pub september: String,
	pub october: String,
	pub november: String,
	pub december: String,
}

/// A generic key/value pair.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct KeyValue {
	pub name: String,
	pub value: String,
}

impl ZzpConfig {
	/// Find the ZZP configuration file by searching the filesystem.
	///
	/// This looks for `zzp.toml` in the start dir and each parent dir until it is found,
	/// or until the search leaves the `root_dir`.
	pub fn find(root_dir: impl AsRef<Path>, start_dir: impl AsRef<Path>) -> Option<PathBuf> {
		let root_dir = root_dir.as_ref();
		let mut dir = start_dir.as_ref();
		loop {
			if !dir.starts_with(root_dir) {
				return None;
			}
			let candidate = dir.join("zzp.toml");
			if candidate.is_file() {
				return Some(candidate);
			}
			dir = dir.parent()?;
		}
	}

	/// Parse a ZZP configuration from a byte slice.
	pub fn parse(bytes: &[u8]) -> Result<Self, toml::de::Error> {
		toml::from_slice(bytes)
	}

	/// Parse a file as ZZP configuration.
	pub fn read_file(path: impl AsRef<Path>) -> Result<Self, ReadFileError> {
		read_toml(path)
	}
}

impl CustomerConfig {
	/// Find the customer configuration file by searching the filesystem.
	///
	/// This looks for `customer.toml` in the start dir and each parent dir until it is found,
	/// or until the search leaves the `root_dir`.
	pub fn find(root_dir: impl AsRef<Path>, start_dir: impl AsRef<Path>) -> Option<PathBuf> {
		let root_dir = root_dir.as_ref();
		let mut dir = start_dir.as_ref();
		loop {
			if !dir.starts_with(root_dir) {
				return None;
			}
			let candidate = dir.join("customer.toml");
			if candidate.is_file() {
				return Some(candidate);
			}
			dir = dir.parent()?;
		}
	}

	/// Parse a customer configuration from a byte slice.
	pub fn parse(bytes: &[u8]) -> Result<Self, toml::de::Error> {
		toml::from_slice(bytes)
	}

	/// Parse a file as customer configuration.
	pub fn read_file(path: impl AsRef<Path>) -> Result<Self, ReadFileError> {
		read_toml(path)
	}
}

#[derive(Debug)]
pub enum ReadFileError {
	Open(PathBuf, std::io::Error),
	Read(PathBuf, std::io::Error),
	Toml(PathBuf, toml::de::Error),
}

impl std::error::Error for ReadFileError {}
impl std::fmt::Display for ReadFileError {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			Self::Open(path, error) => write!(f, "failed to open {} for reading: {}", path.display(), error),
			Self::Read(path, error) => write!(f, "failed to read from {}: {}", path.display(), error),
			Self::Toml(path, error) => write!(f, "failed to parse {}: {}", path.display(), error),
		}
	}
}

pub fn read_toml<T: serde::de::DeserializeOwned>(path: impl AsRef<Path>) -> Result<T, ReadFileError> {
	use std::io::Read;

	let path = path.as_ref();
	let mut file = std::fs::File::open(path)
		.map_err(|e| ReadFileError::Open(path.into(), e))?;
	let mut bytes = Vec::new();
	file.read_to_end(&mut bytes)
		.map_err(|e| ReadFileError::Read(path.into(), e))?;
	toml::from_slice(&bytes)
		.map_err(|e| ReadFileError::Toml(path.into(), e))
}
