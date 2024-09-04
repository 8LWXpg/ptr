//! Retry io operations on error.

use std::io::{Read, Write};
use std::path::Path;
use std::thread;
use std::time::Duration;
use std::{fs, io};

const MAX_RETRIES: u32 = 10;
const RETRY_DELAY: Duration = Duration::from_millis(50);

fn retry<F, T, E>(mut operation: F) -> Result<T, io::Error>
where
	F: FnMut() -> Result<T, E>,
	E: Into<io::Error>,
{
	let mut last_error = None;

	for _ in 0..MAX_RETRIES {
		match operation() {
			Ok(result) => return Ok(result),
			Err(err) => {
				last_error = Some(err.into());
				thread::sleep(RETRY_DELAY);
			}
		}
	}

	Err(last_error.unwrap_or_else(|| io::Error::new(io::ErrorKind::Other, "Max retries reached")))
}

/// Wrapper around `io::copy` that retries on errors.
pub fn copy<R, W>(reader: &mut R, writer: &mut W) -> io::Result<u64>
where
	R: ?Sized,
	W: ?Sized,
	R: Read,
	W: Write,
{
	retry(|| io::copy(reader, writer))
}

/// Wrapper around `fs::remove_dir_all` that retries on errors.
pub fn remove_dir_all<P: AsRef<Path>>(path: P) -> io::Result<()> {
	retry(|| fs::remove_dir_all(path.as_ref()))
}
