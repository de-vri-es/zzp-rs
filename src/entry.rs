use crate::date::{Date, DateParseError};
use crate::hours::{Hours, HoursParseError};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Entry {
	date: Date,
	hours: Hours,
	description: String,
	tags: Vec<String>,
}

impl Entry {
	pub fn from_bytes(data: &[u8]) -> Result<Self, EntryParseError> {
		let data = std::str::from_utf8(data).map_err(|_| EntryParseError::InvalidUtf8)?;
		Self::from_str(data)
	}

	pub fn from_str(data: &str) -> Result<Self, EntryParseError> {
		// Extract and trim fields.
		let mut fields = data.splitn(3, ',');
		let date = fields.next().unwrap().trim();
		let hours = fields.next().ok_or(InvalidEntrySyntax::new(data))?.trim();
		let description = fields.next().ok_or(InvalidEntrySyntax::new(data))?.trim();

		// Parse fields.
		let date = Date::from_str(date)?;
		let hours = Hours::from_str(hours)?;
		let description = String::from(description);

		Ok(Self {
			date,
			hours,
			description,
			tags: Vec::new(),
		})
	}
}

#[derive(Clone, Debug)]
pub enum EntryParseError {
	InvalidUtf8,
	InvalidEntrySyntax(InvalidEntrySyntax),
	DateParseError(DateParseError),
	HoursParseError(HoursParseError),
}

#[derive(Clone, Debug)]
pub struct InvalidEntrySyntax {
	data: String,
}

impl InvalidEntrySyntax {
	fn new(data: impl Into<String>) -> Self {
		Self { data: data.into() }
	}
}

impl From<InvalidEntrySyntax> for EntryParseError {
	fn from(other: InvalidEntrySyntax) -> Self {
		EntryParseError::InvalidEntrySyntax(other)
	}
}

impl From<DateParseError> for EntryParseError {
	fn from(other: DateParseError) -> Self {
		EntryParseError::DateParseError(other)
	}
}

impl From<HoursParseError> for EntryParseError {
	fn from(other: HoursParseError) -> Self {
		EntryParseError::HoursParseError(other)
	}
}

impl std::fmt::Display for EntryParseError {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			Self::InvalidUtf8 => write!(f, "invalid UTF-8 in entry"),
			Self::InvalidEntrySyntax(e) => write!(f, "{}", e),
			Self::DateParseError(e) => write!(f, "{}", e),
			Self::HoursParseError(e) => write!(f, "{}", e),
		}
	}
}

impl std::fmt::Display for InvalidEntrySyntax {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "invalid syntax: expected \"date, hours, description\", got {:?}", self.data)
	}
}

#[cfg(test)]
#[test]
fn test_parse_entry_ok() {
	use assert2::assert;
	let parsed = Entry::from_str("2020-01-02, 10h12m, goofing around");
	assert!(let Ok(_) = parsed);
	let parsed = parsed.unwrap();
	assert!(parsed.date.year() == 2020);
	assert!(parsed.date.month() == crate::date::Month::January);
	assert!(parsed.date.day() == 2);
	assert!(parsed.date.day() == 2);
	assert!(parsed.hours.total_minutes() == 612);
	assert!(parsed.description == "goofing around");
}

#[cfg(test)]
#[test]
fn test_parse_not_ok() {
	use assert2::assert;
	assert!(let Err(EntryParseError::InvalidEntrySyntax(_)) = Entry::from_str("20m, stabbing co-workers"));
	assert!(let Err(EntryParseError::DateParseError(_)) = Entry::from_str("when was this again?, 1h30m, swapping production and test environment"));
	assert!(let Err(EntryParseError::HoursParseError(_)) = Entry::from_str("2020-01-01, 17hhh20mmm, wrokking onnnn new yeaarss *hiccup*"));
}
