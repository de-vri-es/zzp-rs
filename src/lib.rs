#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Transaction<'a> {
	date: Date,
	description: &'a str,
	mutations: Vec<Mutation<'a>>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Date {
	year: i32,
	month: i32,
	day: i32,
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
		while let Some(transaction) = Self::parse_from_lines(&mut lines)? {
			output.push(transaction);
		}
		Ok(output)
	}

	pub fn parse_from_lines(lines: &mut std::str::Lines<'a>) -> Result<Option<Self>, ParseError<'a>> {
		let header = match lines.next() {
			Some(x) => x,
			None => return Ok(None),
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
		let date = Date::parse_from_str(date).map_err(|e| e.for_token(date))?;

		// Parse mutation till we encounter an empty line.
		let mut mutations = Vec::new();
		while let Some(line) = lines.next() {
			let line = line.trim();
			if line.is_empty() {
				break;
			}
			mutations.push(Mutation::parse_from_str(line)?);
		}

		Ok(Some(Self { date, description, mutations }))
	}
}

impl Date {
	pub fn from_parts(year: i32, month: i32, day: i32) -> Result<Self, ()> {
		if month < 1 || month > 12 {
			Err(())
		} else if day < 1 || day > days_in_month(month, is_leap_year(year)) {
			Err(())
		} else {
			Ok(Self { year, month, day })
		}
	}

	pub fn parse_from_str<'a>(data: &'a str) -> Result<Self, InvalidTransactionHeaderDetails> {
		let mut components = data.splitn(3, '-')
			.map(|x| x.parse::<i32>().map_err(|_| InvalidDate));

		let year = components.next().ok_or(InvalidDate)??;
		let month = components.next().ok_or(InvalidDate)??;
		let day = components.next().ok_or(InvalidDate)??;

		Self::from_parts(year, month, day).map_err(|_| InvalidDate)
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
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InvalidTransactionHeaderDetails {
	MissingHeader,
	MissingDescription,
	InvalidDate,
}

use InvalidTransactionHeaderDetails::*;

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

impl InvalidMutationDetails {
	fn for_token(self, token: &str) -> ParseError {
		ParseError { details: self.into(), token }
	}
}

fn partition(data: &str, seperator: char) -> Option<(&str, &str)> {
	let mut split = data.splitn(2, seperator);
	Some((split.next()?, split.next()?))
}

fn is_leap_year(year: i32) -> bool {
	if year % 400 == 0 {
		true
	} else if year % 100 == 0 {
		false
	} else {
		year % 4 == 0
	}
}

fn days_in_month(month: i32, leap_year: bool) -> i32 {
	match month {
		01 => 31,
		02 if !leap_year => 28,
		02 if leap_year => 29,
		03 => 31,
		04 => 30,
		05 => 31,
		06 => 30,
		07 => 31,
		08 => 31,
		09 => 30,
		10 => 31,
		11 => 30,
		12 => 31,
		_ => panic!("invalid month: {}", month),
	}
}
