#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Date {
	year: i16,
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

pub use Month::*;

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
}

impl Date {
	pub fn new(year: i16, month: u8, day: u8) -> Result<Self, InvalidDate> {
		let month = Month::new(month)?;
		InvalidDayForMonth::check(year, month, day)?;
		Ok(Self { year, month, day })
	}

	pub fn year(self) -> i16 {
		self.year
	}

	pub fn month(self) -> Month {
		self.month
	}

	pub fn day(self) -> u8 {
		self.day
	}

	pub fn days_in_month(self) -> u8 {
		days_in_month(self.year, self.month)
	}

	pub fn first_day_of_month(self) -> Self {
		Self {
			year: self.year,
			month: self.month,
			day: 1,
		}
	}

	pub fn first_day_of_year(self) -> Self {
		Self {
			year: self.year,
			month: January,
			day: 1,
		}
	}

	pub fn next_day(self) -> Date {
		if self.day == self.days_in_month() {
			self.first_day_next_month()
		} else {
			Self {
				year: self.year,
				month: self.month,
				day: self.day + 1,
			}
		}
	}

	pub fn first_day_next_month(self) -> Self {
		if self.month == December {
			self.first_day_next_year()
		} else {
			Self {
				year: self.year,
				month: self.month.wrapping_next(),
				day: 1,
			}
		}
	}

	pub fn first_day_next_year(self) -> Self {
		Self {
			year: self.year + 1,
			month: January,
			day: 1,
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

pub fn is_leap_year(year: i16) -> bool {
	year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

pub fn days_in_month(year: i16, month: Month) -> u8 {
	match month {
		January => 31,
		Februari => if is_leap_year(year) { 29 } else { 28 },
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvalidMonthNumber {
	number: u8,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvalidDayForMonth {
	year: i16,
	month: Month,
	day: u8,
}

impl InvalidDayForMonth {
	pub fn check(year: i16, month: Month, day: u8) -> Result<(), Self> {
		if day < 1 || day > days_in_month(year, month) {
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
			days_in_month(self.year, self.month),
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
		assert!(Date::new(2020, 1, 2).unwrap().next_day() == Date::new(2020, 1, 3).unwrap());
		assert!(Date::new(2020, 1, 31).unwrap().next_day() == Date::new(2020, 2, 1).unwrap());
		assert!(Date::new(2020, 12, 31).unwrap().next_day() == Date::new(2021, 1, 1).unwrap());
	}

	#[test]
	fn test_parse_date() {
		assert!(let Ok(Date { year: 2020, month: January, day: 2 }) = Date::from_str("2020-01-02"));
		assert!(let Err(DateParseError::InvalidDateSyntax(_)) = Date::from_str("not-a-date"));
		assert!(let Err(DateParseError::InvalidDate(_)) = Date::from_str("2019-30-12"));
	}

	#[test]
	fn test_is_leap_year() {
		assert!(is_leap_year(2020) == true);
		assert!(is_leap_year(2021) == false);
		assert!(is_leap_year(1900) == false);
		assert!(is_leap_year(2000) == true);
	}

}
