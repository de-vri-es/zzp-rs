use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use ordered_float::NotNan;

pub mod invoice;

/// Main configuration file for the ZZP tools.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "PascalCase")]
pub struct ZzpConfig {
	/// The company details.
	pub company: Company,

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
pub struct Tax {
	/// Default VAT percentage for delivered goods/services.
	pub vat: NotNan<f64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Invoice {
	/// The font to use for generated invoices.
	font: String,

	/// The base font size to use for generated invoices.
	font_size: NotNan<f64>,
}

/// Customer details.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Customer {
	pub name: String,
	pub address: Vec<String>,
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
		let path = path.as_ref();
		let bytes = std::fs::read(path)
			.map_err(|e| ReadFileError::Io(path.into(), e))?;
		Self::parse(&bytes)
			.map_err(|e| ReadFileError::Toml(path.into(), e))
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
		let path = path.as_ref();
		let bytes = std::fs::read(path)
			.map_err(|e| ReadFileError::Io(path.into(), e))?;
		Self::parse(&bytes)
			.map_err(|e| ReadFileError::Toml(path.into(), e))
	}
}

#[derive(Debug)]
pub enum ReadFileError {
	Io(PathBuf, std::io::Error),
	Toml(PathBuf, toml::de::Error),
}

impl std::error::Error for ReadFileError {}
impl std::fmt::Display for ReadFileError {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			Self::Io(path, error) => write!(f, "failed to read {}: {}", path.display(), error),
			Self::Toml(path, error) => write!(f, "failed to parse {}: {}", path.display(), error),
		}
	}
}
