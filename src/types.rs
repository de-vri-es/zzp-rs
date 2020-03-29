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

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Account<'a> {
	pub raw: &'a str,
}

impl<'a> Account<'a> {
	pub fn from_raw(raw: &'a str) -> Self {
		Self { raw }
	}
}
