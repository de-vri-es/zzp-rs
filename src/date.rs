#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Date {
	year: i32,
	month: i32,
	day: i32,
}

pub struct InvalidDate;

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

	pub fn parse_from_str<'a>(data: &'a str) -> Result<Self, InvalidDate> {
		let mut components = data.splitn(3, '-')
			.map(|x| x.parse::<i32>().map_err(|_| InvalidDate));

		let year = components.next().ok_or(InvalidDate)??;
		let month = components.next().ok_or(InvalidDate)??;
		let day = components.next().ok_or(InvalidDate)??;

		Self::from_parts(year, month, day).map_err(|_| InvalidDate)
	}
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
