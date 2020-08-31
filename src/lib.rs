use std::path::Path;

mod hours;
mod entry;
mod partial_date;

pub use hours::*;
pub use entry::*;
pub use partial_date::*;

pub use gregorian;

pub fn parse_file(path: impl AsRef<Path>) -> Result<Vec<Entry>, FileParseError> {
	let data = std::fs::read(path)?;
	parse_bytes(&data).map_err(|e| e.into())
}

pub fn parse_bytes(data: &[u8]) -> Result<Vec<Entry>, FileEntryParseError> {
	let mut result = Vec::new();

	for (i, line) in data.split(|c| *c == b'\n').enumerate() {
		let line = std::str::from_utf8(line).map_err(|_| FileEntryParseError::new(i, EntryParseError::InvalidUtf8))?;
		let line = line.trim();
		if line.is_empty() || line.starts_with('#') {
			continue;
		}

		let entry = Entry::from_str(line).map_err(|e| FileEntryParseError::new(i + 1, e))?;
		result.push(entry);
	}

	Ok(result)
}

#[derive(Debug)]
pub enum FileParseError {
	Io(std::io::Error),
	Entry(FileEntryParseError)
}

#[derive(Debug)]
pub struct FileEntryParseError {
	pub line: usize,
	pub error: EntryParseError,
}

impl FileEntryParseError {
	fn new(line: usize, error: EntryParseError) -> Self {
		Self { line, error }
	}
}

impl std::error::Error for FileParseError {}
impl std::error::Error for FileEntryParseError {}

impl From<std::io::Error> for FileParseError {
	fn from(other: std::io::Error) -> Self {
		Self::Io(other)
	}
}

impl From<FileEntryParseError> for FileParseError {
	fn from(other: FileEntryParseError) -> Self {
		Self::Entry(other)
	}
}

impl std::fmt::Display for FileParseError {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			Self::Io(e) => write!(f, "{}", e),
			Self::Entry(e) => write!(f, "{}", e),
		}
	}
}

impl std::fmt::Display for FileEntryParseError {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "on line {}: {}", self.line, self.error)
	}
}
