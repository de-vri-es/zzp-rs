use ordered_float::NotNan;
use std::collections::BTreeMap;
use zzp::gregorian::{Date, Month};

use pdf_writer::{A4, BoxPosition, PdfWriter, Margins, mm, pt, MM_PER_PT};

use crate::{ZzpConfig, Customer, DateLocalization};

pub struct InvoiceEntry {
	pub date: Date,
	pub description: String,
	pub quantity: NotNan<f64>,
	pub unit: String,
	pub unit_price: NotNan<f64>,
	pub vat_percentage: NotNan<f64>,
}

impl InvoiceEntry {
	pub fn total_ex_vat(&self) -> NotNan<f64> {
		self.quantity * self.unit_price
	}

	pub fn total_vat_only(&self) -> NotNan<f64> {
		self.quantity * self.unit_price * self.vat_percentage * 0.01
	}

	pub fn total_inc_vat(&self) -> NotNan<f64> {
		self.quantity * self.unit_price * (self.vat_percentage * 0.01 + 1.0)
	}
}

pub fn make_invoice<W>(
	stream: W,
	config: &ZzpConfig,
	recipient: &Customer,
	invoice_number: &str,
	invoice_date: Date,
	entries: &[InvoiceEntry],
) -> Result<(), String>
where
	W: std::io::Write + 'static,
{
	let mut writer = PdfWriter::new(stream)?;
	let lang = &config.invoice_localization;

	let font_size = pt(*config.invoice.font_size);

	let plain = |font_size| pdf_writer::TextStyle {
		font: pdf_writer::FontSpec::plain(&config.invoice.font, font_size),
		align: pdf_writer::TextAlign::Left,
		justify: false,
		line_height: 1.0,
	};
	let basic = plain(font_size);
	let basic_right = pdf_writer::TextStyle {
		align: pdf_writer::TextAlign::Right,
		.. basic.clone()
	};

	let bold = |font_size| pdf_writer::TextStyle {
		font: pdf_writer::FontSpec::bold(&config.invoice.font, font_size),
		align: pdf_writer::TextAlign::Left,
		justify: false,
		line_height: 0.8,
	};

	let page = writer.page(A4, Margins::vh(mm(30.0), mm(20.0)))?;

	// Add reciepient name and address.
	{
		let mut table = pdf_writer::TableBuilder::new(&writer, page.text_width() * 0.5);
		table.position(BoxPosition::at_xy(mm(20.0), mm(42.0)));
		table.cell_padding(Margins::tblr(mm(0.0), -font_size * 0.2 * MM_PER_PT, mm(0.0), mm(0.0)));

		table.add_column(false, None);
		table.add_column(false, None);

		table.add_cell(&format!("{}:    ", &lang.to), &basic_right)?;
		table.add_cell(&recipient.name, &basic)?;
		for line in &recipient.address {
			table.add_cell("", &basic_right)?;
			table.add_cell(line, &basic)?;
		}

		let table = table.build();
		table.draw(&page);
	}

	let mut y;
	let vskip = font_size * 1.5 * MM_PER_PT;
	// Add sender name and address.
	{
		let mut table = pdf_writer::TableBuilder::new(&writer, page.text_width() * 0.5);
		table.position(BoxPosition::at(page.line_right()).anchor_right());
		table.cell_padding(Margins::tblr(mm(0.0), -font_size * 0.2 * MM_PER_PT, mm(0.0), mm(0.0)));

		table.add_column(false, None);
		table.add_column(false, None);

		table.add_cell(&format!("{}:    ", &lang.from), &basic_right)?;
		table.add_cell(&config.company.name, &basic)?;
		for line in &config.company.address {
			table.add_cell("", &basic_right)?;
			table.add_cell(line, &basic)?;
		}

		table.add_cell("", &basic)?;
		table.add_cell("", &basic)?;
		for line in &config.company.contact {
			table.add_cell(&format!("{}:    ", line.name), &basic_right)?;
			table.add_cell(&line.value, &basic)?;
		}

		table.add_cell("", &basic)?;
		table.add_cell("", &basic)?;
		for line in &config.company.legal {
			table.add_cell(&format!("{}:    ", line.name), &basic_right)?;
			table.add_cell(&line.value, &basic)?;
		}

		table.add_cell("", &basic)?;
		table.add_cell("", &basic)?;
		for line in &config.company.payment {
			table.add_cell(&format!("{}:    ", line.name), &basic_right)?;
			table.add_cell(&line.value, &basic)?;
		}

		let table = table.build();
		y = table.baseline(table.rows() - 1);
		table.draw(&page);
	}

	{
		let title = page.draw_text_box(&lang.invoice, &bold(font_size * 2.8), BoxPosition::at_xy(mm(20.0), y).anchor_baseline(), None)?;
		y = mm(title.logical.max.y) + vskip;

		let mut table = pdf_writer::TableBuilder::new(&writer, page.text_width());
		table.position(BoxPosition::at_xy(mm(20.0), y));
		table.cell_padding(Margins::tblr(mm(0.0), -font_size * 0.2 * MM_PER_PT, mm(0.0), mm(0.0)));
		table.add_column(false, None);
		table.add_column(false, None);
		table.add_cell(&format!("{}:    ", lang.invoice_number), &basic_right)?;
		table.add_cell(invoice_number, &basic)?;
		table.add_cell(&format!("{}:    ", lang.invoice_date), &basic_right)?;
		table.add_cell(&format_date(invoice_date, &config.date_localization), &basic)?;
		let table = table.build();
		y += mm(table.size().height) + vskip;
		table.draw(&page);
	}

	let mut total_ex_vat = NotNan::new(0.0).unwrap();
	let mut totals_vat: BTreeMap<NotNan<f64>, NotNan<f64>> = BTreeMap::new();
	{
		let mut table = pdf_writer::TableBuilder::new(&writer, page.text_width());
		table.position(BoxPosition::at_xy(mm(20.0), y));
		table.cell_padding(Margins::vh(font_size * 0.25 * MM_PER_PT, font_size * 0.5 * MM_PER_PT));
		table.add_column(false, None);
		table.add_column(true, None);
		table.add_column(false, None);
		table.add_column(false, None);
		table.add_column(false, None);
		table.add_column(false, None);
		table.add_cell(&lang.date, &basic)?;
		table.add_cell(&lang.description, &basic)?;
		table.add_cell(&lang.quantity, &basic)?;
		table.add_cell(&lang.entry_unit_price, &basic)?;
		table.add_cell(&lang.entry_total_price, &basic)?;
		table.add_cell(&lang.vat, &basic)?;

		for entry in entries {
			let price = entry.quantity * entry.unit_price;
			total_ex_vat += price;
			*totals_vat.entry(entry.vat_percentage).or_default() += price * entry.vat_percentage / 100.0;

			table.add_cell(&format_date(entry.date, &config.date_localization), &basic_right)?;
			table.add_cell(&entry.description, &basic)?;
			table.add_cell(&format!("{:.02} {}", entry.quantity, entry.unit), &basic_right)?;
			table.add_cell(&format!("{} {:.02}", lang.currency_symbol, entry.unit_price), &basic_right)?;
			table.add_cell(&format!("{} {:.02}", lang.currency_symbol, price), &basic_right)?;
			table.add_cell(&format!("{}%", entry.vat_percentage), &basic_right)?;
		}

		let table = table.build();
		y += mm(table.size().height) + vskip;
		table.draw(&page);
		table.draw_horizontal_border(&page, 1, .., pt(0.5));
	}

	{
		let mut table = pdf_writer::TableBuilder::new(&writer, page.text_width());
		y = (y + mm(A4.height - 40.0) - vskip) * 0.5;
		table.position(BoxPosition::at_xy(page.text_width() + mm(20.0), y).anchor_right().anchor_middle_y());
		table.cell_padding(Margins::vh(font_size * 0.25 * MM_PER_PT, font_size * 0.5 * MM_PER_PT));
		table.add_column(false, None);
		table.add_column(false, None);
		table.add_cell(&format!("{}:", lang.total_ex_vat), &basic_right)?;
		table.add_cell(&format!("{} {:.02}", lang.currency_symbol, total_ex_vat), &basic_right)?;
		let mut total_inc_vat = total_ex_vat;
		for (percentage, total) in &totals_vat {
			total_inc_vat += *total;
			table.add_cell(&format!("{} {}%:", lang.total_vat, percentage), &basic_right)?;
			table.add_cell(&format!("{}{:.02}", lang.currency_symbol, total), &basic_right)?;
		}

		let bold_right = pdf_writer::TextStyle {
			align: pdf_writer::TextAlign::Right,
			.. bold(font_size)
		};
		table.add_cell(&format!("{}:", lang.total_due), &bold_right)?;
		table.add_cell(&format!("{}{:.02}", lang.currency_symbol, total_inc_vat), &bold_right)?;
		let table = table.build();
		y += mm(table.size().height) + vskip;
		table.draw(&page);
		table.draw_horizontal_border(&page, table.rows() - 1, .., pt(0.5));
	}

	page.draw_text_box(&lang.footer, &basic, BoxPosition::at_xy(mm(20.0), mm(A4.height - 40.0)), Some(page.text_width()))?;
	page.draw_text_box("1 / 1", &basic, BoxPosition::at_xy(mm(20.0) + page.text_width() * 0.5, mm(A4.height - 20.0)), Some(page.text_width()))?;

	page.emit(&writer)?;
	Ok(())
}

fn format_date(date: Date, localization: &DateLocalization) -> String {
	let month = format_month(date.month(), localization);
	format!("{} {} {}", date.day(), month, date.year())
}

fn format_month(month: Month, localization: &DateLocalization) -> &str {
	match month {
		Month::January => &localization.january,
		Month::February => &localization.february,
		Month::March => &localization.march,
		Month::April => &localization.april,
		Month::May => &localization.may,
		Month::June => &localization.june,
		Month::July => &localization.july,
		Month::August => &localization.august,
		Month::September => &localization.september,
		Month::October => &localization.october,
		Month::November => &localization.november,
		Month::December => &localization.december,
	}
}
