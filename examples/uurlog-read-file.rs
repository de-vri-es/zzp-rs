use std::path::Path;

fn main() {
	let args: Vec<_> = std::env::args().collect();
	if args.len() != 2 {
		eprintln!("Usage: {} FILE", args[0]);
		std::process::exit(1);
	}
	if let Err(e) = do_main(args[1].as_ref()) {
		eprintln!("Error: {}", e);
		std::process::exit(1);
	}
}

fn do_main(file: &Path) -> Result<(), String> {
	let data = std::fs::read(file).map_err(|e| format!("failed to read {}: {}", file.display(), e))?;

	for (i, line) in data.split(|c| *c == b'\n').enumerate() {
		let line = std::str::from_utf8(line).map_err(|_| format!("invalid UTF-8 on line {}", i))?;
		let line = line.trim();
		if line.is_empty() || line.starts_with('#') {
			continue;
		}

		let entry = zzp::uurlog::Entry::from_str(line).map_err(|e| format!("parse error on line {}: {}", i, e))?;
		println!("{:?}", entry);
	}

	Ok(())
}
