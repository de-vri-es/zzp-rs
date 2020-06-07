#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Hours {
	minutes: u32,
}

impl Hours {
	pub fn from_minutes(minutes: u32) -> Self {
		Self { minutes }
	}

	pub fn from_hours_minutes(hours: u32, minutes: u32) -> Self {
		Self::from_minutes(hours * 60 + minutes)
	}

	pub fn total_minutes(self) -> u32 {
		self.minutes
	}

	pub fn hours(self) -> u32 {
		self.minutes / 60
	}

	pub fn minutes(self) -> u32 {
		self.minutes % 60
	}

	pub fn from_str(data: &str) -> Result<Self, HoursParseError> {
		let mut total = 0;
		let remaining = data;

		if data.is_empty() {
			return Err(HoursParseError::new(data));
		}

		// Parse hours (must precede minutes).
		let remaining = if let Some((hours, rest)) = partition(remaining, 'h') {
			let hours : u32 = hours.parse().map_err(|_| HoursParseError::new(data))?;
			total += hours * 60;
			rest
		} else {
			remaining
		};

		// Parse minutes.
		let remaining = if let Some((minutes, rest)) = partition(remaining, 'm') {
			let minutes : u32 = minutes.parse().map_err(|_| HoursParseError::new(data))?;
			total += minutes;
			rest
		} else {
			remaining
		};

		// Make sure no garbage remains.
		if !remaining.is_empty() {
			return Err(HoursParseError::new(data));
		}

		Ok(Self::from_minutes(total))
	}
}

fn partition(input: &str, split: char) -> Option<(&str, &str)> {
	let mut fields = input.splitn(2, split);
	let first = fields.next().unwrap();
	if let Some(rest) = fields.next() {
		Some((first, rest))
	} else {
		None
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HoursParseError {
	data: String,
}

impl HoursParseError {
	fn new(data: impl Into<String>) -> Self {
		Self { data: data.into() }
	}
}

impl std::error::Error for HoursParseError {}

impl std::fmt::Display for HoursParseError {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "invalid hours syntax: expected something like 3h30m, got {:?}", self.data)
	}
}

#[cfg(test)]
#[test]
fn test_parse_hours() {
	use assert2::assert;

	assert!(let Err(_) = Hours::from_str(""));
	assert!(let Err(_) = Hours::from_str("10"));
	assert!(let Err(_) = Hours::from_str("10h 50m"));
	assert!(Hours::from_str("10h") == Ok(Hours::from_hours_minutes(10, 0)));
	assert!(Hours::from_str("11h30m") == Ok(Hours::from_hours_minutes(11, 30)));
	assert!(Hours::from_str("12h70m") == Ok(Hours::from_hours_minutes(13, 10)));
}
