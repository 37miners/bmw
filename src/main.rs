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

use bmw_err::{err, ErrKind, Error};
use bmw_log::*;
use std::mem::size_of;

info!();

fn main() -> Result<(), Error> {
	real_main(false)?;
	Ok(())
}

fn real_main(debug_startup_32: bool) -> Result<(), Error> {
	// ensure we only support 64 bit
	match size_of::<&char>() == 8 && debug_startup_32 == false {
		true => {}
		false => return Err(err!(ErrKind::IllegalState, "Only 64 bit arch supported")),
	}

	info!("not implemented yet")?;

	Ok(())
}

#[cfg(test)]
mod test {
	use crate::{main, real_main};
	use bmw_err::Error;

	#[test]
	fn test_main() -> Result<(), Error> {
		assert!(main().is_ok());
		Ok(())
	}

	#[test]
	fn test_debug_startup_32() -> Result<(), Error> {
		assert!(real_main(true).is_err());
		Ok(())
	}
}
