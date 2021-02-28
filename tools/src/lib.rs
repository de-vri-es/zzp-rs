use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Main configuration file for the ZZP tools.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "PascalCase")]
pub struct ZzpConfig {
	/// The company details.
	pub company: Company,

	/// The tax details.
	pub tax: Tax,

	/// Localization details.
	pub localization: Localization,
}

/// Configuration file for specific customers.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "PascalCase")]
pub struct CustomerConfig {
	/// Details about the customer itself.
	pub customer: Customer,

	/// Details on how to invoice the customer.
	pub invoice: CustomerInvoice,
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
	pub vat: u32,
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
	/// The path to the invoice template.
	pub template: PathBuf,

	/// The price per hour in cents.
	pub cents_per_hour: u32,

	/// Summarize all hours per day with a single entry.
	pub summarize_per_day: Option<String>,
}

/// Localizaton details for generated content.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Localization {
	/// Translation for the word "invoice".
	pub invoice: String,

	/// Translation for the word "hours".
	pub hours: String,

	/// The sign for money.
	pub money_sign: String,

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
