use crate::hours::{Hours, HoursParseError};

pub use gregorian::Date;
pub use gregorian::DateParseError;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Entry {
	pub date: Date,
	pub hours: Hours,
	pub tags: Vec<String>,
	pub description: String,
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
		let mut description = fields.next().ok_or(InvalidEntrySyntax::new(data))?.trim();

		// Parse fields.
		let date : Date =  date.parse()?;
		let hours = Hours::from_str(hours)?;

		let mut tags = Vec::new();
		while description.starts_with('[') {
			let end = description.find(']').ok_or_else(|| UnclosedTag { data: description.to_string() })?;
			tags.push(description[1..end].to_string());
			description = &description[end + 1..].trim();
		}

		Ok(Self {
			date,
			hours,
			tags,
			description: description.to_string(),
		})
	}
}

impl std::fmt::Display for Entry {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{}, {}, ", self.date, self.hours)?;
		for tag in &self.tags {
			write!(f, "[{}] ", tag)?;
		}
		write!(f, "{}", self.description)?;
		Ok(())
	}
}

#[derive(Clone, Debug)]
pub enum EntryParseError {
	InvalidUtf8,
	InvalidEntrySyntax(InvalidEntrySyntax),
	DateParseError(DateParseError),
	HoursParseError(HoursParseError),
	UnclosedTag(UnclosedTag),
}

#[derive(Clone, Debug)]
pub struct InvalidEntrySyntax {
	data: String,
}

#[derive(Clone, Debug)]
pub struct UnclosedTag {
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

impl From<UnclosedTag> for EntryParseError {
	fn from(other: UnclosedTag) -> Self {
		EntryParseError::UnclosedTag(other)
	}
}

impl std::fmt::Display for EntryParseError {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			Self::InvalidUtf8 => write!(f, "invalid UTF-8 in entry"),
			Self::InvalidEntrySyntax(e) => write!(f, "{}", e),
			Self::DateParseError(e) => write!(f, "{}", e),
			Self::HoursParseError(e) => write!(f, "{}", e),
			Self::UnclosedTag(e) => write!(f, "{}", e),
		}
	}
}

impl std::fmt::Display for InvalidEntrySyntax {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "invalid syntax: expected \"date, hours, description\", got {:?}", self.data)
	}
}

impl std::fmt::Display for UnclosedTag {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "invalid syntax: unclosed tag in description: {:?}", self.data)
	}
}

#[cfg(test)]
#[test]
fn test_parse_entry_ok() {
	use assert2::assert;
	let parsed = Entry::from_str("2020-01-02, 10h12m, [one][two] [three] goofing around");
	assert!(let Ok(_) = parsed);
	let parsed = parsed.unwrap();
	assert!(parsed.date.year() == 2020);
	assert!(parsed.date.month() == gregorian::Month::January);
	assert!(parsed.date.day() == 2);
	assert!(parsed.date.day() == 2);
	assert!(parsed.hours.total_minutes() == 612);
	assert!(parsed.tags == &["one", "two", "three"]);
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
