use super::date::Date;
use super::types::Account;
use super::types::Cents;
use super::types::Mutation;
use super::types::Tag;
use super::types::Transaction;

impl<'a> Transaction<'a> {
	pub fn parse_from_str(data: &'a str) -> Result<Vec<Self>, ParseError<'a>> {
		let mut lines = data.lines();
		let mut output = Vec::new();
		output.reserve((lines.size_hint().0 + 3) / 4);

		while let Some(transaction) = Self::parse_from_lines(&mut lines)? {
			output.push(transaction);
		}

		Ok(output)
	}

	pub fn parse_from_lines(lines: &mut std::str::Lines<'a>) -> Result<Option<Self>, ParseError<'a>> {
		let header = loop {
			let line = match lines.next() {
				Some(x) => x.trim(),
				None => return Ok(None),
			};

			// Skip comments and empty lines.
			if !line.starts_with('#')  && !line.is_empty() {
				break line;
			}
		};

		// Split header in date and description.
		let (date, description) = partition(header, ':')
			.ok_or_else(|| MissingDescription.for_token(header))?;
		let date = date.trim();
		let description = description.trim();

		// Reject empty descriptions.
		if description.is_empty() {
			return Err(MissingDescription.for_token(header));
		}

		// Parse the date.
		let date = Date::parse_from_str(date).map_err(|_| InvalidTransactionHeaderDetails::InvalidDate.for_token(date))?;

		// Parse tags and mutations until there are none left.
		let mut tags = Vec::new();
		let mut mutations = Vec::new();

		while let Some(line) = lines.next() {
			let line = line.trim();
			// Stop on empty line.
			if line.is_empty() {
				break;
			// Ignore comments.
			} else if line.starts_with('#') {
				continue;
			// See if the line looks like a tag.
			} else if let Some(tag) = Tag::parse_from_str(line) {
				if mutations.is_empty() {
					tags.push(tag?);
				} else {
					return Err(InvalidTagDetails::TagAfterMutation.for_token(line));
				}
			// Parse mutations.
			} else {
				mutations.push(Mutation::parse_from_str(line)?);
			}
		}

		Ok(Some(Self { date, description, tags, mutations }))
	}
}

impl<'a> Tag<'a> {
	fn parse_from_str(data: &'a str) -> Option<Result<Self, ParseError<'a>>> {
		let data = data.trim();
		let (label, value) = partition(data, ':')?;
		let label = label.trim();
		let value = value.trim();

		// Check that the label contains only allowed characters.
		if label.chars().find(|c| !c.is_ascii_alphanumeric() && *c != '-').is_some() {
			return Some(Err(InvalidLabel.for_token(label).into()));
		}

		Some(Ok(Self { label, value }))
	}
}

impl<'a> Mutation<'a> {
	fn parse_from_str(data: &'a str) -> Result<Self, ParseError<'a>> {
		let data = data.trim();
		let (amount, account) = partition(data, ' ').ok_or(MissingAccount.for_token(data))?;
		let amount = amount.trim();
		let account = account.trim();

		let sign = match &amount[0..1] {
			"-" => -1,
			"+" => 1,
			_ => return Err(MissingSign.for_token(amount)),
		};

		let Cents(amount) = Cents::parse_from_str(&amount[1..])
			.map_err(|_| InvalidAmount.for_token(amount))?;
		let amount = Cents(amount * sign);

		Ok(Self { amount, account: Account::from_raw(account) })
	}
}

impl Cents {
	fn parse_from_str(data: &str) -> Result<Self, ()> {
		if let Some((whole, decimals)) = partition(data, '.') {
			if decimals.len() != 2 {
				Err(())
			} else {
				let whole : i32 = whole.parse().map_err(|_| ())?;
				let decimals : i32 = decimals.parse().map_err(|_| ())?;
				Ok(Self(whole * 100 + decimals))
			}
		} else {
			let whole : i32 = data.parse().map_err(|_| ())?;
			Ok(Self(whole * 100))
		}
	}
}

#[derive(Clone, Debug)]
pub struct ParseError<'a> {
	pub details: ParseErrorDetails,
	pub token: &'a str,
}


#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParseErrorDetails {
	InvalidTransactionHeader(InvalidTransactionHeaderDetails),
	InvalidMutation(InvalidMutationDetails),
	InvalidTag(InvalidTagDetails),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InvalidTransactionHeaderDetails {
	MissingHeader,
	MissingDescription,
	InvalidDate,
}

use InvalidTransactionHeaderDetails::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InvalidTagDetails {
	InvalidLabel,
	TagAfterMutation,
}

use InvalidTagDetails::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InvalidMutationDetails {
	MissingSign,
	MissingAccount,
	InvalidAmount,
}

use InvalidMutationDetails::*;

impl From<InvalidTransactionHeaderDetails> for ParseErrorDetails {
	fn from(other: InvalidTransactionHeaderDetails) -> Self {
		Self::InvalidTransactionHeader(other)
	}
}

impl From<InvalidTagDetails> for ParseErrorDetails {
	fn from(other: InvalidTagDetails) -> Self {
		Self::InvalidTag(other)
	}
}

impl From<InvalidMutationDetails> for ParseErrorDetails {
	fn from(other: InvalidMutationDetails) -> Self {
		Self::InvalidMutation(other)
	}
}

impl InvalidTransactionHeaderDetails {
	fn for_token(self, token: &str) -> ParseError {
		ParseError { details: self.into(), token }
	}
}

impl InvalidTagDetails {
	fn for_token(self, token: &str) -> ParseError {
		ParseError { details: self.into(), token }
	}
}

impl InvalidMutationDetails {
	fn for_token(self, token: &str) -> ParseError {
		ParseError { details: self.into(), token }
	}
}

fn partition(data: &str, seperator: char) -> Option<(&str, &str)> {
	let mut split = data.splitn(2, seperator);
	Some((split.next()?, split.next()?))
}

impl std::fmt::Display for ParseError<'_> {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "parse error at token: {:?}: {}", self.token, self.details)
	}
}

impl std::fmt::Display for ParseErrorDetails {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			Self::InvalidTransactionHeader(e) => write!(f, "{}", e),
			Self::InvalidTag(e)               => write!(f, "{}", e),
			Self::InvalidMutation(e)          => write!(f, "{}", e),
		}
	}
}

impl std::fmt::Display for InvalidTransactionHeaderDetails {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			Self::MissingHeader      => write!(f, "missing transaction header"),
			Self::MissingDescription => write!(f, "missing transaction description"),
			Self::InvalidDate        => write!(f, "invalid date"),
		}
	}
}

impl std::fmt::Display for InvalidTagDetails {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			Self::InvalidLabel     => write!(f, "invalid tag label"),
			Self::TagAfterMutation => write!(f, "tags are only allowed before the first mutation"),
		}
	}
}

impl std::fmt::Display for InvalidMutationDetails {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			Self::MissingSign    => write!(f, "missing sign (+/-)"),
			Self::MissingAccount => write!(f, "missing account for mutation"),
			Self::InvalidAmount  => write!(f, "invalid mutation amount"),
		}
	}
}

impl std::error::Error for ParseError<'_> {}
impl std::error::Error for ParseErrorDetails {}
impl std::error::Error for InvalidTransactionHeaderDetails {}
impl std::error::Error for InvalidTagDetails {}
impl std::error::Error for InvalidMutationDetails {}
