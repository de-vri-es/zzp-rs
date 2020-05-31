use crate::date::{Date, DateParseError};
use crate::hours::{Hours, HoursParseError};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Entry<'a> {
	date: Date,
	hours: Hours,
	description: &'a str,
	tags: BTreeMap<&'a str, &'a str>,
}

impl<'a> Entry<'a> {
	pub fn from_str(data: &'a str) -> Result<Self, EntryParseError> {
		// Extract and trim fields.
		let mut fields = data.splitn(3, ',');
		let date = fields.next().unwrap().trim();
		let hours = fields.next().ok_or(InvalidEntrySyntax { data })?.trim();
		let description = fields.next().ok_or(InvalidEntrySyntax { data })?.trim();

		// Parse fields.
		let date = Date::from_str(date)?;
		let hours = Hours::from_str(hours)?;

		Ok(Self {
			date,
			hours,
			description,
			tags: BTreeMap::new(),
		})
	}
}

pub enum EntryParseError<'a> {
	InvalidEntrySyntax(InvalidEntrySyntax<'a>),
	DateParseError(DateParseError<'a>),
	HoursParseError(HoursParseError<'a>),
}

pub struct InvalidEntrySyntax<'a> {
	data: &'a str,
}

impl<'a> From<InvalidEntrySyntax<'a>> for EntryParseError<'a> {
	fn from(other: InvalidEntrySyntax<'a>) -> Self {
		EntryParseError::InvalidEntrySyntax(other)
	}
}

impl<'a> From<DateParseError<'a>> for EntryParseError<'a> {
	fn from(other: DateParseError<'a>) -> Self {
		EntryParseError::DateParseError(other)
	}
}

impl<'a> From<HoursParseError<'a>> for EntryParseError<'a> {
	fn from(other: HoursParseError<'a>) -> Self {
		EntryParseError::HoursParseError(other)
	}
}

impl std::fmt::Display for EntryParseError<'_> {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			Self::InvalidEntrySyntax(e) => write!(f, "{}", e),
			Self::DateParseError(e) => write!(f, "{}", e),
			Self::HoursParseError(e) => write!(f, "{}", e),
		}
	}
}

impl std::fmt::Display for InvalidEntrySyntax<'_> {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "invalid syntax: expected \"date, hours, description\", got {:?}", self.data)
	}
}
