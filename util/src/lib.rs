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

mod array;
mod builder;
mod hash;
mod macros;
mod misc;
mod ser;
mod slabs;
mod suffix_tree;
mod threadpool;
mod types;

pub use crate::ser::{deserialize, serialize, BinReader, BinWriter, SlabReader, SlabWriter};
pub use crate::slabs::GLOBAL_SLAB_ALLOCATOR;
pub use crate::types::{
	Array, ArrayList, Builder, ConfigOption, HashsetIterator, HashtableIterator, List, ListConfig,
	ListIterator, Match, MatchBuilder, Pattern, PoolResult, Queue, Reader, Serializable, Slab,
	SlabAllocator, SlabAllocatorConfig, SlabMut, SortableList, Stack, StaticHashset,
	StaticHashsetConfig, StaticHashtable, StaticHashtableConfig, SuffixTree, ThreadPool,
	ThreadPoolConfig, Writer,
};
