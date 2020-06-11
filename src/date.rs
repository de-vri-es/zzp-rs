#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Year {
	year: i16,
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

pub use Month::*;

impl std::convert::TryFrom<u8> for Month {
	type Error = InvalidMonthNumber;

	fn try_from(other: u8) -> Result<Self, Self::Error> {
		Self::new(other)
	}
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct YearMonth {
	year: Year,
	month: Month,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Date {
	year: Year,
	month: Month,
	day: u8,
}

impl Year {
	pub fn new(year: i16) -> Self {
		Self { year }
	}

	pub fn as_number(self) -> i16 {
		self.year
	}

	pub fn has_leap_day(self) -> bool {
		self.year % 4 == 0 && (self.year % 100 != 0 || self.year % 400 == 0)
	}

	pub fn next(self) -> Self {
		Self::new(self.year + 1)
	}

	pub fn prev(self) -> Self {
		Self::new(self.year - 1)
	}

	pub fn with_month(self, month: Month) -> YearMonth {
		YearMonth::new(self, month)
	}

	pub fn first_month(self) -> YearMonth {
		self.with_month(January)
	}

	pub fn last_month(self) -> YearMonth {
		self.with_month(December)
	}

	pub fn first_day(self) -> Date {
		Date {
			year: self,
			month: January,
			day: 1,
		}
	}

	pub fn last_day(self) -> Date {
		Date {
			year: self,
			month: December,
			day: 31,
		}
	}
}

impl From<i16> for Year {
	fn from(other: i16) -> Self {
		Self::new(other)
	}
}

impl PartialEq<i16> for Year {
	fn eq(&self, other: &i16) -> bool {
		self.as_number() == *other
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

	pub unsafe fn new_unchecked(month: u8) -> Self {
		std::mem::transmute(month)
	}

	pub fn as_number(self) -> u8 {
		self as u8
	}

	pub fn wrapping_next(self) -> Self {
		match self {
			January => Februari,
			Februari  => March,
			March => April,
			April => May,
			May => June,
			June => July,
			July => August,
			August => September,
			September => October,
			October => November,
			November => December,
			December => January,
		}
	}

	pub fn wrapping_prev(self) -> Self {
		match self {
			January => December,
			Februari => January,
			March => Februari,
			April => March,
			May => April,
			June => May,
			July => June,
			August => July,
			September => August,
			October => September,
			November => October,
			December => November,
		}
	}
}

impl PartialEq<u8> for Month {
	fn eq(&self, other: &u8) -> bool {
		self.as_number() == *other
	}
}

impl YearMonth {
	pub fn new(year: impl Into<Year>, month: Month) -> Self {
		let year = year.into();
		Self { year, month }
	}

	pub fn year(self) -> Year {
		self.year
	}

	pub fn month(self) -> Month {
		self.month
	}

	pub fn total_days(self) -> u8 {
		match self.month {
			January => 31,
			Februari => if self.year.has_leap_day() { 29 } else { 28 },
			March => 31,
			April => 30,
			May => 31,
			June => 30,
			July => 31,
			August => 31,
			September => 30,
			October => 31,
			November => 30,
			December => 31,
		}
	}

	fn next(self) -> Self {
		if self.month == December {
			Self::new(self.year.next(), January)
		} else {
			Self::new(self.year, self.month.wrapping_next())
		}
	}

	fn prev(self) -> Self {
		if self.month == January {
			Self::new(self.year.prev(), December)
		} else {
			Self::new(self.year, self.month.wrapping_prev())
		}
	}

	pub fn with_day(self, day: u8) -> Result<Date, InvalidDayForMonth> {
		InvalidDayForMonth::check(self.year, self.month, day)?;
		Ok(Date {
			year: self.year,
			month: self.month,
			day,
		})
	}

	fn first_day(self) -> Date {
		Date {
			year: self.year,
			month: self.month,
			day: 1,
		}
	}

	fn last_day(self) -> Date {
		Date {
			year: self.year,
			month: self.month,
			day: self.total_days(),
		}
	}
}

impl Date {
	pub fn new<M>(year: impl Into<Year>, month: M, day: u8) -> Result<Self, InvalidDate>
	where
		M: std::convert::TryInto<Month>,
		InvalidDate: From<M::Error>,
	{
		let year_month = YearMonth::new(year, month.try_into()?);
		Ok(year_month.with_day(day)?)
	}

	pub fn year(self) -> Year {
		self.year
	}

	pub fn month(self) -> Month {
		self.month
	}

	pub fn day(self) -> u8 {
		self.day
	}

	pub fn year_month(self) -> YearMonth {
		YearMonth::new(self.year(), self.month())
	}

	pub fn next(self) -> Date {
		if self.day == self.year_month().total_days() {
			self.year_month().next().first_day()
		} else {
			Self {
				year: self.year,
				month: self.month,
				day: self.day + 1,
			}
		}
	}

	pub fn prev(self) -> Date {
		if self.day == 1 {
			self.year_month().prev().last_day()
		} else {
			Self {
				year: self.year,
				month: self.month,
				day: self.day - 1,
			}
		}
	}

	pub fn from_str(data: &str) -> Result<Self, DateParseError> {
		// Extract fields.
		let mut fields = data.splitn(3, '-');
		let year = fields.next().unwrap();
		let month = fields.next().ok_or_else(|| InvalidDateSyntax::new(data))?;
		let day = fields.next().ok_or_else(|| InvalidDateSyntax::new(data))?;

		// Parse fields as numbers.
		let year : i16 = year.parse().map_err(|_| InvalidDateSyntax::new(data))?;
		let month : u8 = month.parse().map_err(|_| InvalidDateSyntax::new(data))?;
		let day : u8 = day.parse().map_err(|_| InvalidDateSyntax::new(data))?;

		// Construct date.
		Ok(Self::new(year, month, day)?)
	}
}

impl std::fmt::Display for Year {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{:04}", self.year)
	}
}

impl std::fmt::Display for Month {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		// Implement Display for Month by calling Debug::fmt.
		std::fmt::Debug::fmt(self, f)
	}
}

impl std::fmt::Display for YearMonth {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{:04}-{:02}", self.year.as_number(), self.month().as_number())
	}
}

impl std::fmt::Display for Date {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{:04}-{:02}-{:02}", self.year.as_number(), self.month.as_number(), self.day)
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DateParseError {
	InvalidDateSyntax(InvalidDateSyntax),
	InvalidDate(InvalidDate),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvalidDateSyntax {
	data: String,
}

impl InvalidDateSyntax {
	fn new(data: impl Into<String>) -> Self {
		Self { data: data.into() }
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InvalidDate {
	InvalidMonthNumber(InvalidMonthNumber),
	InvalidDayForMonth(InvalidDayForMonth),
}

impl From<std::convert::Infallible> for InvalidDate {
	fn from(_: std::convert::Infallible) -> Self {
		unreachable!()
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvalidMonthNumber {
	number: u8,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvalidDayForMonth {
	year: Year,
	month: Month,
	day: u8,
}

impl InvalidDayForMonth {
	pub fn check(year: Year, month: Month, day: u8) -> Result<(), Self> {
		if day < 1 || day > YearMonth::new(year, month).total_days() {
			Err(Self { year, month, day })
		} else {
			Ok(())
		}
	}
}

impl From<InvalidDateSyntax> for DateParseError {
	fn from(other: InvalidDateSyntax) -> Self {
		Self::InvalidDateSyntax(other)
	}
}

impl From<InvalidDate> for DateParseError {
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

impl std::error::Error for DateParseError {}
impl std::error::Error for InvalidDateSyntax {}
impl std::error::Error for InvalidDate {}
impl std::error::Error for InvalidMonthNumber {}
impl std::error::Error for InvalidDayForMonth {}

impl std::fmt::Display for DateParseError {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			Self::InvalidDateSyntax(e) => write!(f, "{}", e),
			Self::InvalidDate(e) => write!(f, "{}", e),
		}
	}
}

impl std::fmt::Display for InvalidDateSyntax {
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
			YearMonth::new(self.year, self.month).total_days(),
			self.day,
		)
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use assert2::assert;

	#[test]
	fn test_make_date() {
		assert!(let Ok(_) = Date::new(2020, 1, 2));
		assert!(Date::new(2020, 1, 2).unwrap().year() == 2020);
		assert!(Date::new(2020, 1, 2).unwrap().month() == January);
		assert!(Date::new(2020, 1, 2).unwrap().day() == 2);

		assert!(let Ok(_) = Date::new(2020, 2, 29));
		assert!(let Err(_) = Date::new(2020, 2, 30));
		assert!(let Ok(_) = Date::new(2019, 2, 28));
		assert!(let Err(_) = Date::new(2019, 2, 29));
	}

	#[test]
	fn test_next_date() {
		assert!(Date::new(2020, 1, 2).unwrap().next() == Date::new(2020, 1, 3).unwrap());
		assert!(Date::new(2020, 1, 31).unwrap().next() == Date::new(2020, 2, 1).unwrap());
		assert!(Date::new(2020, 12, 31).unwrap().next() == Date::new(2021, 1, 1).unwrap());
	}

	#[test]
	fn test_parse_date() {
		assert!(Date::from_str("2020-01-02").unwrap().year() == 2020);
		assert!(Date::from_str("2020-01-02").unwrap().month() == January);
		assert!(Date::from_str("2020-01-02").unwrap().day() == 2);
		assert!(let Err(DateParseError::InvalidDateSyntax(_)) = Date::from_str("not-a-date"));
		assert!(let Err(DateParseError::InvalidDate(_)) = Date::from_str("2019-30-12"));
	}

	#[test]
	fn test_is_leap_year() {
		assert!(Year::new(2020).has_leap_day() == true);
		assert!(Year::new(2021).has_leap_day() == false);
		assert!(Year::new(1900).has_leap_day() == false);
		assert!(Year::new(2000).has_leap_day() == true);
	}

}
