mod date;
use date::Date;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Transaction<'a> {
	date: Date,
	description: &'a str,
	tags: Vec<Tag<'a>>,
	mutations: Vec<Mutation<'a>>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Tag<'a> {
	label: &'a str,
	value: &'a str,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Mutation<'a> {
	amount: Cents,
	account: Account<'a>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Cents(pub i32);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Account<'a> {
	raw: &'a str,
}

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
			// Parse mutations.
			} else if line.starts_with('+') || line.starts_with('-') {
				mutations.push(Mutation::parse_from_str(line)?);
			// Treat rest as tags.
			} else {
				tags.push(Tag::parse_from_str(line)?);
			}
		}

		Ok(Some(Self { date, description, tags, mutations }))
	}
}

impl<'a> Tag<'a> {
	fn parse_from_str(data: &'a str) -> Result<Self, ParseError<'a>> {
		let data = data.trim();
		let (label, value) = partition(data, ':').ok_or(MissingTagSeparator.for_token(data))?;
		let label = label.trim();
		let value = value.trim();

		// Check that the label contains only allowed characters.
		if label.chars().find(|c| !c.is_ascii_alphanumeric() && *c != '-').is_some() {
			return Err(InvalidLabel.for_token(label).into());
		}

		Ok(Self { label, value })
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
			_ => return Err(MissingValueSign.for_token(amount)),
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

impl<'a> Account<'a> {
	fn from_raw(raw: &'a str) -> Self {
		Self { raw }
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
	MissingTagSeparator,
	InvalidLabel,
}

use InvalidTagDetails::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InvalidMutationDetails {
	MissingValueSign,
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
