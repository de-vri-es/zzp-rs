use crate::date::Date;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Transaction<'a> {
	pub date: Date,
	pub description: &'a str,
	pub tags: Vec<Tag<'a>>,
	pub mutations: Vec<Mutation<'a>>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Tag<'a> {
	pub label: &'a str,
	pub value: &'a str,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Mutation<'a> {
	pub amount: Cents,
	pub account: Account<'a>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Cents(pub i32);

impl Cents {
	pub fn is_negative(self) -> bool {
		self.0 < 0
	}
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Account<'a> {
	pub raw: &'a str,
}

impl<'a> Account<'a> {
	pub fn from_raw(raw: &'a str) -> Self {
		Self { raw }
	}
}

impl std::fmt::Display for Cents {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let Cents(amount) = self;
		let whole = amount / 100;
		let cents = amount % 100;
		write!(f, "{:+}.{:02}", whole, cents)
	}
}

impl std::fmt::Display for Account<'_> {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		self.raw.fmt(f)
	}
}
