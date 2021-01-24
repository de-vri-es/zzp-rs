use super::date::Date;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Transaction<'a> {
	pub date: Date,
	pub description: &'a str,
	pub tags: Vec<Tag<'a>>,
	pub mutations: Vec<Mutation<'a>>,
}

impl Transaction<'_> {
	pub fn mutates_account(&self, prefix: &str) -> bool {
		for mutation in &self.mutations {
			if mutation.account.matches_prefix(prefix) {
				return true;
			}
		}
		false
	}
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
	pub fn total_cents(self) -> i32 {
		self.0
	}

	pub fn is_negative(self) -> bool {
		self.0 < 0
	}
}

impl std::ops::Add<Cents> for Cents {
	type Output = Cents;

	fn add(self, other: Cents) -> Self::Output {
		Cents(self.0 + other.0)
	}
}

impl std::ops::Add<&Cents> for &Cents {
	type Output = Cents;

	fn add(self, other: &Cents) -> Self::Output {
		*self + *other
	}
}

impl std::ops::AddAssign<Cents> for Cents {
	fn add_assign(&mut self, other: Cents) {
		self.0 += other.0;
	}
}

impl std::ops::AddAssign<&Cents> for Cents {
	fn add_assign(&mut self, other: &Cents) {
		*self += *other
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

	pub fn as_str(self) -> &'a str {
		self.raw
	}

	pub fn matches_prefix(&self, mut prefix: &str) -> bool {
		if prefix.ends_with('/') {
			prefix = &prefix[..prefix.len() - 1];
		}
		if self.raw == prefix {
			true
		} else {
			self.raw.starts_with(prefix) && self.raw.as_bytes()[prefix.len()] == b'/'
		}
	}

	pub fn name(self) -> &'a str {
		match self.raw.rfind('/') {
			Some(i) => &self.raw[i + 1..],
			None => self.raw,
		}
	}

	pub fn walk_nodes(self) -> impl Iterator<Item = Account<'a>> {
		self.raw.match_indices('/')
			.map(move |(i, _)| Account::from_raw(&self.raw[..i]))
			.chain(Some(self))
	}

	pub fn parent(self) -> Option<Self> {
		let index = self.raw.rfind('/')?;
		Some(Self::from_raw(&self.raw[..index]))
	}

	pub fn parents(self) -> AccountParents<'a> {
		AccountParents{ current: Some(self) }
	}

	pub fn common_parent(self, other: Self) -> Option<Self> {
		self.parents()
			.zip(other.parents())
			.take_while(|(a, b)| a == b)
			.last()
			.map(|(a, _b)| a)
	}
}

pub struct AccountParents<'a> {
	current: Option<Account<'a>>,
}

impl<'a> Iterator for AccountParents<'a> {
	type Item = Account<'a>;

	fn next(&mut self) -> Option<Self::Item> {
		self.current = self.current.and_then(|x| x.parent());
		self.current.clone()
	}
}

impl std::fmt::Display for Cents {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let Cents(amount) = self;
		let whole = amount / 100;
		let cents = (amount % 100).abs();
		write!(f, "{:+}.{:02}", whole, cents)
	}
}

impl std::fmt::Display for Account<'_> {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		self.raw.fmt(f)
	}
}
