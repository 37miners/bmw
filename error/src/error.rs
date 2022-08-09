// Copyright (c) 2022, 37 Miners, LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use bmw_deps::failure::{Backtrace, Context, Fail};
use std::ffi::OsString;
use std::fmt::{Display, Formatter, Result};
use std::num::TryFromIntError;

/// Base Error struct which is used throughout bmw.
#[derive(Debug, Fail)]
pub struct Error {
	inner: Context<ErrorKind>,
}

/// Kinds of errors that can occur.
#[derive(Clone, Eq, PartialEq, Debug, Fail)]
pub enum ErrorKind {
	/// IO Error
	#[fail(display = "IO Error: {}", _0)]
	IO(String),
	/// Log Error
	#[fail(display = "Log Error: {}", _0)]
	Log(String),
	/// UTF8 Error
	#[fail(display = "UTF8 Error: {}", _0)]
	Utf8(String),
	/// ArrayIndexOutOfBounds
	#[fail(display = "ArrayIndexOutofBounds: {}", _0)]
	ArrayIndexOutOfBounds(String),
	/// Configuration Error
	#[fail(display = "Configuration Error: {}", _0)]
	Configuration(String),
	/// Poison error multiple locks
	#[fail(display = "Poison Error: {}", _0)]
	Poison(String),
	/// CorruptedData
	#[fail(display = "Corrupted Data Error: {}", _0)]
	CorruptedData(String),
	/// Timeout
	#[fail(display = "Timeout: {}", _0)]
	Timeout(String),
	/// Capacity Exceeded
	#[fail(display = "Capacity Exceeded: {}", _0)]
	CapacityExceeded(String),
	/// UnexpectedEof Error
	#[fail(display = "UnexpectedEOF: {}", _0)]
	UnexpectedEof(String),
	/// IllegalArgument
	#[fail(display = "IllegalArgument: {}", _0)]
	IllegalArgument(String),
	/// Miscellaneous Error
	#[fail(display = "Miscellaneous Error: {}", _0)]
	Misc(String),
}

/// The names of ErrorKinds in this crate. This enum is used to map to error
/// names using the [`crate::errkind`] and [`crate::map_err`] macros.
pub enum ErrKind {
	/// IO Error.
	IO,
	/// Log Error.
	Log,
	/// A conversion to the utf8 format resulted in an error.
	Utf8,
	/// An array index was out of bounds.
	ArrayIndexOutOfBounds,
	/// Configuration error.
	Configuration,
	/// Attempt to obtain a lock resulted in a poison error. See [`std::sync::PoisonError`]
	/// for further details.
	Poison,
	/// Data is corrupted.
	CorruptedData,
	/// A timeout has occurred.
	Timeout,
	/// The capacity is exceeded.
	CapacityExceeded,
	/// Unexpected end of file.
	UnexpectedEof,
	/// Illegal argument was specified.
	IllegalArgument,
	/// A Miscellaneous Error occurred.
	Misc,
}

impl Display for Error {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result {
		let output = format!("{} \n Backtrace: {:?}", self.inner, self.backtrace());
		Display::fmt(&output, f)
	}
}

impl Error {
	/// get the kind of error that occurred.
	pub fn kind(&self) -> ErrorKind {
		self.inner.get_context().clone()
	}

	/// get the cause (if available) of this error.
	pub fn cause(&self) -> Option<&dyn Fail> {
		self.inner.cause()
	}

	/// get the backtrace (if available) of this error.
	pub fn backtrace(&self) -> Option<&Backtrace> {
		self.inner.backtrace()
	}

	/// get the inner error as a string.
	pub fn inner(&self) -> String {
		self.inner.to_string()
	}
}

impl From<ErrorKind> for Error {
	fn from(kind: ErrorKind) -> Error {
		Error {
			inner: Context::new(kind),
		}
	}
}

impl From<std::io::Error> for Error {
	fn from(e: std::io::Error) -> Error {
		Error {
			inner: Context::new(ErrorKind::IO(format!("{}", e))),
		}
	}
}

impl From<OsString> for Error {
	fn from(e: OsString) -> Error {
		Error {
			inner: Context::new(ErrorKind::Misc(format!("{:?}", e))),
		}
	}
}

impl From<TryFromIntError> for Error {
	fn from(e: TryFromIntError) -> Error {
		Error {
			inner: Context::new(ErrorKind::Misc(format!("TryFromIntError: {}", e))),
		}
	}
}

#[cfg(test)]
mod test {
	use crate::{Error, ErrorKind};
	use bmw_deps::substring::Substring;
	use std::convert::TryInto;
	use std::ffi::OsString;

	fn get_os_string() -> Result<(), Error> {
		Err(OsString::new().into())
	}

	fn check_error<T: Sized, Q>(r: Result<T, Q>, ematch: Error) -> Result<(), Error>
	where
		crate::Error: From<Q>,
	{
		if let Err(r) = r {
			let e: Error = r.into();

			// Some errors are slightly different on different platforms. So, we check
			// the first 10 characters which is specified in the ErrorKind generally.
			assert_eq!(
				e.to_string().substring(0, 10),
				ematch.to_string().substring(0, 10)
			);
			assert_eq!(
				e.kind().to_string().substring(0, 10),
				ematch.to_string().substring(0, 10)
			);
			assert!(e.cause().is_none());
			assert!(e.backtrace().is_some());
			assert_eq!(
				e.inner().substring(0, 10),
				ematch.to_string().substring(0, 10),
			);
			println!("e.backtrace()={:?}", e.backtrace());
			println!("e={}", e);
		}
		Ok(())
	}

	#[test]
	fn test_errors() -> Result<(), Error> {
		check_error(
			std::fs::File::open("/no/path/here"),
			ErrorKind::IO("No such file or directory (os error 2)".to_string()).into(),
		)?;

		check_error(get_os_string(), ErrorKind::Misc("".to_string()).into())?;

		let x: Result<u32, _> = u64::MAX.try_into();
		check_error(x, ErrorKind::Misc(format!("TryFromIntError..")).into())?;

		Ok(())
	}
}
