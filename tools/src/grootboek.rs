use yansi::Paint;
use zzp::grootboek::{Cents, Transaction};

#[allow(clippy::comparison_chain)]
pub fn color_cents(cents: Cents) -> yansi::Paint<Cents> {
	if cents.total_cents() > 0 {
		yansi::Color::Green.style().paint(cents)
	} else if cents.total_cents() < 0 {
		yansi::Color::Red.style().paint(cents)
	} else {
		yansi::Color::Fixed(241).paint(cents)
	}
}

pub fn print_full(transaction: &Transaction) {
	println!("{date}: {desc}",
		date = Paint::cyan(transaction.date),
		desc = Paint::magenta(transaction.description),
	);
	for tag in &transaction.tags {
		println!("{label}: {value}",
			label = Paint::cyan(tag.label),
			value = Paint::cyan(tag.value),
		);
	}
	for mutation in &transaction.mutations {
		println!("{amount} {account}",
			amount  = color_cents(mutation.amount),
			account = mutation.account,
		);
	}
}

