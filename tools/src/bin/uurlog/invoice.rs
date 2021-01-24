use super::{InvoiceOptions, read_uurlog};

pub(crate) fn make_invoice(options: InvoiceOptions) -> Result<(), ()> {
	let entries = read_uurlog(&options.file, options.period)?;
	todo!();
}
