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

//! This is the dependency crate. All bmw dependencies are included in this crate as re-exports and
//! used by the other crates within the repo.

#[cfg(windows)]
pub use ws2_32;

pub use backtrace;
pub use chrono;
pub use colored;
pub use dyn_clone;
pub use errno;
pub use failure;
pub use failure_derive;
pub use futures;
pub use interprocess;
pub use kqueue_sys;
pub use lazy_static;
pub use libc;
pub use nix;
pub use num_format;
pub use portpicker;
pub use rand;
pub use random_string;
pub use substring;
pub use try_traits;
pub use winapi;
