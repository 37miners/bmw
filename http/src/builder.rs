// Copyright (c) 2022, 37 Miners, LLC
// Some code and concepts from:
// * Grin: https://github.com/mimblewimble/grin
// * Arti: https://gitlab.torproject.org/tpo/core/arti
// * BitcoinMW: https://github.com/bitcoinmw/bitcoinmw
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

use crate::types::HttpServerImpl;
use crate::Builder;
use crate::{HttpConfig, HttpServer};
use bmw_err::*;

impl Builder {
	pub fn build_http_server(config: HttpConfig) -> Result<Box<dyn HttpServer>, Error> {
		Ok(Box::new(HttpServerImpl::new(config)?))
	}
}

#[cfg(test)]
mod test {
	use crate::{Builder, HttpConfig};
	use bmw_err::*;

	#[test]
	fn test_http_builder() -> Result<(), Error> {
		let config = HttpConfig { threads: 6 };
		let mut server = Builder::build_http_server(config)?;
		server.start()?;
		Ok(())
	}
}
