#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Date {
	year: i32,
	month: Month,
	day: u8,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Month {
	January = 1,
	Februari = 2,
	March = 3,
	April = 4,
	May = 5,
	June = 6,
	July = 7,
	August = 8,
	September = 9,
	October = 10,
	November = 11,
	December = 12,
}

impl Date {
	pub fn new(year: i32, month: u8, day: u8) -> Result<Self, InvalidDate> {
		let month = Month::new(month)?;
		InvalidDayForMonth::check(year, month, day)?;
		Ok(Self { year, month, day })
	}

	pub fn year(self) -> i32 {
		self.year
	}

	pub fn month(self) -> Month {
		self.month
	}

	pub fn day(self) -> u8 {
		self.day
	}

	pub fn from_str(data: &str) -> Result<Self, DateParseError> {
		// Extract fields.
		let mut fields = data.splitn(3, '-');
		let year = fields.next().unwrap();
		let month = fields.next().ok_or(InvalidDateSyntax { data })?;
		let day = fields.next().ok_or(InvalidDateSyntax { data })?;

		// Parse fields as numbers.
		let year : i32 = year.parse().map_err(|_| InvalidDateSyntax { data })?;
		let month : u8 = month.parse().map_err(|_| InvalidDateSyntax { data })?;
		let day : u8 = day.parse().map_err(|_| InvalidDateSyntax { data })?;

		// Construct date.
		Ok(Self::new(year, month, day)?)
	}
}

impl Month {
	pub fn new(month: u8) -> Result<Self, InvalidMonthNumber> {
		match month {
			1 => Ok(Self::January),
			2 => Ok(Self::Februari),
			3 => Ok(Self::March),
			4 => Ok(Self::April),
			5 => Ok(Self::May),
			6 => Ok(Self::June),
			7 => Ok(Self::July),
			8 => Ok(Self::August),
			9 => Ok(Self::September),
			10 => Ok(Self::October),
			11 => Ok(Self::November),
			12 => Ok(Self::December),
			number => Err(InvalidMonthNumber { number }),
		}
	}

	pub fn as_number(self) -> u8 {
		self as u8
	}
}

impl std::fmt::Display for Month {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		// Implement Display for Month by calling Debug::fmt.
		std::fmt::Debug::fmt(self, f)
	}
}

impl std::fmt::Display for Date {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{:04}-{:02}-{:02}", self.year, self.month.as_number(), self.day)
	}
}

pub fn is_leap_year(year: i32) -> bool {
	year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

#[cfg(test)]
#[test]
fn test_is_leap_year() {
	use assert2::check;
	check!(is_leap_year(2020) == true);
	check!(is_leap_year(2021) == false);
	check!(is_leap_year(1900) == false);
	check!(is_leap_year(2000) == true);
}

pub fn days_in_month(year: i32, month: Month) -> u8 {
	match month {
		Month::January => 31,
		Month::Februari => if is_leap_year(year) { 29 } else { 28 },
		Month::March => 31,
		Month::April => 30,
		Month::May => 31,
		Month::June => 30,
		Month::July => 31,
		Month::August => 31,
		Month::September => 30,
		Month::October => 31,
		Month::November => 30,
		Month::December => 31,
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DateParseError<'a> {
	InvalidDateSyntax(InvalidDateSyntax<'a>),
	InvalidDate(InvalidDate),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvalidDateSyntax<'a> {
	data: &'a str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InvalidDate {
	InvalidMonthNumber(InvalidMonthNumber),
	InvalidDayForMonth(InvalidDayForMonth),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvalidMonthNumber {
	number: u8,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvalidDayForMonth {
	year: i32,
	month: Month,
	day: u8,
}

impl InvalidDayForMonth {
	pub fn check(year: i32, month: Month, day: u8) -> Result<(), Self> {
		if day < 1 || day > days_in_month(year, month) {
			Err(Self { year, month, day })
		} else {
			Ok(())
		}
	}
}

impl<'a> From<InvalidDateSyntax<'a>> for DateParseError<'a> {
	fn from(other: InvalidDateSyntax<'a>) -> Self {
		Self::InvalidDateSyntax(other)
	}
}

impl From<InvalidDate> for DateParseError<'_> {
	fn from(other: InvalidDate) -> Self {
		Self::InvalidDate(other)
	}
}

impl From<InvalidMonthNumber> for InvalidDate {
	fn from(other: InvalidMonthNumber) -> Self {
		Self::InvalidMonthNumber(other)
	}
}

impl From<InvalidDayForMonth> for InvalidDate {
	fn from(other: InvalidDayForMonth) -> Self {
		Self::InvalidDayForMonth(other)
	}
}

impl std::error::Error for DateParseError<'_> {}
impl std::error::Error for InvalidDateSyntax<'_> {}
impl std::error::Error for InvalidDate {}
impl std::error::Error for InvalidMonthNumber {}
impl std::error::Error for InvalidDayForMonth {}

impl std::fmt::Display for DateParseError<'_> {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			Self::InvalidDateSyntax(e) => write!(f, "{}", e),
			Self::InvalidDate(e) => write!(f, "{}", e),
		}
	}
}

impl std::fmt::Display for InvalidDateSyntax<'_> {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "invalid date syntax: expected \"YYYY-MM-DD\", got {:?}", self.data)
	}
}

impl std::fmt::Display for InvalidDate {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			Self::InvalidMonthNumber(e) => write!(f, "{}", e),
			Self::InvalidDayForMonth(e) => write!(f, "{}", e),
		}
	}
}

impl std::fmt::Display for InvalidMonthNumber {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "invalid month number: expected 1-12, got {}", self.number)
	}
}

impl std::fmt::Display for InvalidDayForMonth {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(
			f,
			"invalid day for {} {}: expected 1-{}, got {}",
			self.month,
			self.year,
			days_in_month(self.year, self.month),
			self.day,
		)
	}
}
