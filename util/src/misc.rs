// Copyright (c) 2022, 37 Miners, LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use bmw_err::*;
use bmw_log::*;

info!();

/// Utility to convert a usize to an arbitrary length slice (up to 8 bytes).
pub fn usize_to_slice(mut n: usize, slice: &mut [u8]) -> Result<(), Error> {
	let len = slice.len();
	if len > 8 {
		let fmt = format!("slice must be equal to or less than 8 bytes ({})", len);
		return Err(err!(ErrKind::IllegalArgument, fmt));
	}

	for i in (0..len).rev() {
		slice[i] = (n & 0xFF) as u8;
		n >>= 8;
	}

	if n != 0 {
		// this is an overflow, but for our purposes we return "MAX".
		for i in 0..len {
			slice[i] = 0xFF;
		}
	}

	Ok(())
}

/// Utility to convert an arbitrary length slice (up to 8 bytes) to a usize.
pub fn slice_to_usize(slice: &[u8]) -> Result<usize, Error> {
	let len = slice.len();
	if len > 8 {
		let fmt = format!("slice must be equal to or less than 8 bytes ({})", len);
		return Err(err!(ErrKind::IllegalArgument, fmt));
	}
	let mut ret = 0;
	for i in 0..len {
		ret <<= 8;
		ret |= (slice[i] & 0xFF) as usize;
	}

	Ok(ret)
}

pub fn set_max(slice: &mut [u8]) {
	for i in 0..slice.len() {
		slice[i] = 0xFF;
	}
}

#[cfg(test)]
mod test {
	use crate::misc::{slice_to_usize, usize_to_slice};
	use bmw_err::*;
	use bmw_log::*;

	info!();

	#[test]
	fn test_usize_to_slice() -> Result<(), Error> {
		// test 1 byte
		for i in 0..u8::MAX {
			let mut b = [0u8; 1];
			usize_to_slice(i as usize, &mut b)?;
			assert_eq!(slice_to_usize(&b)?, i as usize);
		}

		// test 2 bytes
		for i in 0..u16::MAX {
			let mut b = [0u8; 2];
			usize_to_slice(i as usize, &mut b)?;
			assert_eq!(slice_to_usize(&b)?, i as usize);
		}

		// test 3 bytes
		for i in 0..16777216 {
			let mut b = [0u8; 3];
			usize_to_slice(i as usize, &mut b)?;
			assert_eq!(slice_to_usize(&b)?, i as usize);
		}

		// one bigger is an error
		let mut b = [0u8; 3];
		usize_to_slice(16777216, &mut b)?;
		assert_eq!(b, [0xFF, 0xFF, 0xFF]);

		// 4 bytes is too big to test whole range,
		// try some bigger ones with a partial range
		for i in 1099511620000usize..1099511627776usize {
			let mut b = [0u8; 6];
			usize_to_slice(i as usize, &mut b)?;
			assert_eq!(slice_to_usize(&b)?, i as usize);
		}

		for i in 11099511620000usize..11099511627776usize {
			let mut b = [0u8; 7];
			usize_to_slice(i as usize, &mut b)?;
			assert_eq!(slice_to_usize(&b)?, i as usize);
		}

		assert!(usize_to_slice(1, &mut [0u8; 9]).is_err());

		assert!(slice_to_usize(&mut [0u8; 9]).is_err());

		Ok(())
	}
}
