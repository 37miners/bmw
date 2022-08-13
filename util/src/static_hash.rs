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

use crate::ser::{serialize, BinReader};
use crate::types::StaticHashset;
use crate::{
	Serializable, Slab, SlabAllocator, SlabAllocatorConfig, SlabMut, StaticHashtable,
	StaticIterator, GLOBAL_SLAB_ALLOCATOR,
};
use bmw_err::{err, ErrKind, Error};
use bmw_log::*;
use std::collections::hash_map::DefaultHasher;
use std::convert::TryInto;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::io::Cursor;

info!();

const SLAB_OVERHEAD: u64 = 8;
const SLOT_EMPTY: u64 = u64::MAX;
const SLOT_DELETED: u64 = u64::MAX - 1;
const GLOBAL_ALLOCATOR_NOT_INITIALIZED: &str =
	"Thread local slab allocator was not initialized. Initializing with default values.";

#[derive(Debug)]
pub struct StaticHashtableConfig {
	pub max_entries: u64,
	pub max_load_factor: f64,
}

#[derive(Debug)]
pub struct StaticHashsetConfig {
	pub max_entries: u64,
	pub max_load_factor: f64,
}

#[derive(Debug)]
struct StaticHashConfig {
	max_entries: u64,
	max_load_factor: f64,
}

impl From<StaticHashtableConfig> for StaticHashConfig {
	fn from(config: StaticHashtableConfig) -> Self {
		let _ = debug!("converting {:?}", config);
		Self {
			max_entries: config.max_entries,
			max_load_factor: config.max_load_factor,
		}
	}
}

impl From<StaticHashsetConfig> for StaticHashConfig {
	fn from(config: StaticHashsetConfig) -> Self {
		let _ = debug!("converting {:?}", config);
		Self {
			max_entries: config.max_entries,
			max_load_factor: config.max_load_factor,
		}
	}
}

impl Default for StaticHashtableConfig {
	fn default() -> Self {
		Self {
			max_entries: 1_000_000,
			max_load_factor: 0.75,
		}
	}
}

impl Default for StaticHashsetConfig {
	fn default() -> Self {
		Self {
			max_entries: 1_000_000,
			max_load_factor: 0.75,
		}
	}
}

struct StaticHashImpl {
	config: StaticHashConfig,
	slabs: Option<Box<dyn SlabAllocator + Send + Sync>>,
	entry_array: Vec<u64>,
	slab_size: usize,
	first_entry: usize,
}

impl StaticHashImpl {
	fn new(
		config: StaticHashConfig,
		slabs: Option<Box<dyn SlabAllocator + Send + Sync>>,
	) -> Result<Self, Error> {
		let mut entry_array = vec![];
		Self::init(&config, &mut entry_array)?;
		let (slab_size, slabs) = match slabs {
			Some(slabs) => (usize!(slabs.slab_size()?), Some(slabs)),
			None => {
				let slab_size = GLOBAL_SLAB_ALLOCATOR.with(|f| -> Result<usize, Error> {
					let slabs = unsafe { f.get().as_mut().unwrap() };
					let slab_size = match slabs.slab_size() {
						Ok(slab_size) => usize!(slab_size),
						Err(_e) => {
							warn!(GLOBAL_ALLOCATOR_NOT_INITIALIZED)?;
							slabs.init(SlabAllocatorConfig::default())?;
							usize!(slabs.slab_size()?)
						}
					};
					Ok(slab_size)
				})?;
				(slab_size, None)
			}
		};
		Ok(StaticHashImpl {
			config,
			slab_size,
			slabs,
			entry_array,
			first_entry: usize::MAX,
		})
	}

	fn init(config: &StaticHashConfig, entry_array: &mut Vec<u64>) -> Result<(), Error> {
		if config.max_load_factor <= 0.0 || config.max_load_factor > 1.0 {
			return Err(err!(
				ErrKind::IllegalArgument,
				"StaticHash: max_load_factor must be greater than 0 and equal to or less than 1."
			));
		}
		// calculate size of entry_array. Must be possible to have max_entries, with the
		// max_load_factor
		let size: i32 = (config.max_entries as f64 / config.max_load_factor).floor() as i32;
		debug!("entry array init to size = {}", size)?;
		entry_array.resize(usize!(size), SLOT_EMPTY);
		Ok(())
	}
}

pub struct StaticHashtableIter<'a, K, V> {
	cur: usize,
	h: &'a Box<dyn StaticHashtable<K, V>>,
}

impl<'a, K, V> Iterator for StaticHashtableIter<'a, K, V>
where
	K: Serializable + Hash,
	V: Serializable,
{
	type Item = (K, V);
	fn next(self: &mut StaticHashtableIter<'a, K, V>) -> Option<(K, V)> {
		match Self::do_next(self) {
			Ok(x) => x,
			Err(e) => {
				let _ = error!("StaticHashtableIter generated unexpected error: {}", e);
				None
			}
		}
	}
}

impl<'a, K, V> StaticHashtableIter<'a, K, V>
where
	K: Serializable,
	V: Serializable,
{
	fn do_next(self: &mut StaticHashtableIter<'a, K, V>) -> Result<Option<(K, V)>, Error> {
		Ok(if self.cur == usize::MAX {
			debug!("NONE")?;
			None
		} else {
			debug!("cur={}", self.cur)?;
			match self.h.slab(self.cur) {
				Ok(slab) => {
					let (k, v) = self.h.read_kv(slab.id().try_into().unwrap()).unwrap();
					let slab = slab.get();
					debug!("slab={:?},k={:?},v={:?}", slab, k, v)?;

					// we are always 64 bit due to check in main. Unwrap is safe.
					self.cur = usize::from_be_bytes(slab[0..8].try_into().unwrap());
					debug!("self.cur is now {}", self.cur)?;
					Some((k, v))
				}
				Err(e) => {
					error!(
						"Error iterating through slab hash. It is in an invalid state: {}",
						e
					)?;
					None
				}
			}
		})
	}
}

impl<'a, K, V> IntoIterator for &'a Box<dyn StaticHashtable<K, V>>
where
	K: Serializable + Hash,
	V: Serializable,
{
	type Item = (K, V);
	type IntoIter = StaticHashtableIter<'a, K, V>;

	fn into_iter(self) -> Self::IntoIter {
		Self::IntoIter {
			cur: self.first_entry(),
			h: &self,
		}
	}
}

impl<K, V> StaticHashtable<K, V> for StaticHashImpl
where
	K: Serializable + Hash,
	V: Serializable,
{
	fn insert(&mut self, key: &K, value: &V) -> Result<(), Error> {
		self.insert_impl::<K, V>(Some(key), 0, None, Some(value), None)
	}
	fn get(&self, key: &K) -> Result<Option<V>, Error> {
		self.get_impl(Some(key), None, 0)
	}
	fn remove(&mut self, key: &K) -> Result<(), Error> {
		self.remove_impl(key)
	}
	fn iter(&self) -> Result<Box<dyn StaticIterator<'_, (K, V)>>, Error> {
		self.iter_impl()
	}
	fn get_raw<'b>(&'b self, key: &[u8], hash: u64) -> Result<Option<Box<dyn Slab + 'b>>, Error> {
		self.get_raw_impl::<K>(key, hash)
	}
	fn get_raw_mut<'b>(
		&'b mut self,
		key: &[u8],
		hash: u64,
	) -> Result<Option<Box<dyn SlabMut + 'b>>, Error> {
		self.get_raw_mut_impl::<K>(key, hash)
	}
	fn insert_raw(&mut self, key: &[u8], hash: u64, value: &[u8]) -> Result<(), Error> {
		self.insert_impl::<K, V>(None, hash, Some(key), None, Some(value))
	}

	fn first_entry(&self) -> usize {
		self.first_entry
	}

	fn slab<'b>(&'b self, id: usize) -> Result<Box<dyn Slab + 'b>, Error> {
		self.get_slab(u64!(id))
	}

	fn read_kv(&self, slab_id: usize) -> Result<(K, V), Error> {
		self.read_kv_ser(slab_id)
	}
}

impl<K> StaticHashset<K> for StaticHashImpl
where
	K: Serializable + Hash,
{
	fn insert(&mut self, key: &K) -> Result<(), Error> {
		self.insert_impl::<K, K>(Some(key), 0, None, None, None)
	}
	fn contains(&self, key: &K) -> Result<bool, Error> {
		debug!("contains:self.config={:?},k={:?}", self.config, key)?;
		Ok(self.find_entry(Some(key), None, 0)?.is_some())
	}
	fn remove(&mut self, key: &K) -> Result<(), Error> {
		self.remove_impl(key)
	}
	fn iter(&self) -> Result<Box<dyn StaticIterator<'_, K>>, Error> {
		self.iter_impl()
	}
	fn insert_raw(&mut self, _key: &[u8]) -> Result<(), Error> {
		todo!()
	}
}

impl StaticHashImpl {
	fn get_slab<'a>(&'a self, id: u64) -> Result<Box<dyn Slab + 'a>, Error> {
		match &self.slabs {
			Some(slabs) => Ok(slabs.get(id)?),
			None => GLOBAL_SLAB_ALLOCATOR.with(|f| -> Result<Box<dyn Slab>, Error> {
				Ok(unsafe { f.get().as_ref().unwrap().get(id)? })
			}),
		}
	}

	fn get_mut<'a>(&'a mut self, id: u64) -> Result<Box<dyn SlabMut + 'a>, Error> {
		match &mut self.slabs {
			Some(slabs) => Ok(slabs.get_mut(id)?),
			None => GLOBAL_SLAB_ALLOCATOR.with(|f| -> Result<Box<dyn SlabMut>, Error> {
				Ok(unsafe { f.get().as_mut().unwrap().get_mut(id)? })
			}),
		}
	}

	fn allocate<'a>(&'a mut self) -> Result<Box<dyn SlabMut + 'a>, Error> {
		match &mut self.slabs {
			Some(slabs) => Ok(slabs.allocate()?),
			None => GLOBAL_SLAB_ALLOCATOR.with(|f| -> Result<Box<dyn SlabMut>, Error> {
				Ok(unsafe { f.get().as_mut().unwrap().allocate()? })
			}),
		}
	}

	fn get_free_count(&self) -> Result<usize, Error> {
		match &self.slabs {
			Some(slabs) => Ok(usize!(slabs.free_count()?)),
			None => GLOBAL_SLAB_ALLOCATOR.with(|f| -> Result<usize, Error> {
				Ok(usize!(unsafe { f.get().as_ref().unwrap().free_count()? }))
			}),
		}
	}

	fn get_raw_mut_impl<'b, K>(
		&'b mut self,
		key_raw: &[u8],
		hash: u64,
	) -> Result<Option<Box<dyn SlabMut + 'b>>, Error>
	where
		K: Serializable + Hash,
	{
		let entry = self.find_entry::<K>(None, Some(key_raw), hash)?;
		debug!("entry at {:?}", entry)?;
		match entry {
			Some(entry) => {
				let id = self.entry_array[entry];
				let ret = self.get_mut(id)?;
				Ok(Some(ret))
			}
			None => Ok(None),
		}
	}

	fn get_raw_impl<'b, K>(
		&'b self,
		key_raw: &[u8],
		hash: u64,
	) -> Result<Option<Box<dyn Slab + 'b>>, Error>
	where
		K: Serializable + Hash,
	{
		let entry = self.find_entry::<K>(None, Some(key_raw), hash)?;
		debug!("entry at {:?}", entry)?;
		match entry {
			Some(entry) => {
				let id = self.entry_array[entry];
				let ret = self.get_slab(id)?;
				Ok(Some(ret))
			}
			None => Ok(None),
		}
	}

	fn get_impl<K, V>(
		&self,
		key_ser: Option<&K>,
		key_raw: Option<&[u8]>,
		hash: u64,
	) -> Result<Option<V>, Error>
	where
		K: Serializable + Hash,
		V: Serializable,
	{
		debug!("get_impl:self.config={:?},k={:?}", self.config, key_ser)?;
		let entry = self.find_entry(key_ser, key_raw, hash)?;
		debug!("entry at {:?}", entry)?;

		match entry {
			Some(entry) => Ok(Some(
				self.read_kv_ser::<K, V>(usize!(self.entry_array[entry]))?.1,
			)),
			None => Ok(None),
		}
	}

	fn read_kv_ser<K, V>(&self, slab_id: usize) -> Result<(K, V), Error>
	where
		K: Serializable,
		V: Serializable,
	{
		let (k, v) = self.read_value(slab_id)?;
		debug!("value_read = ({:?}, {:?})", k, v)?;
		let mut cursor = Cursor::new(k);
		cursor.set_position(0);
		let mut reader1 = BinReader::new(&mut cursor);
		let mut cursor = Cursor::new(v);
		cursor.set_position(0);
		let mut reader2 = BinReader::new(&mut cursor);
		let k = K::read(&mut reader1)?;
		let v = V::read(&mut reader2)?;
		Ok((k, v))
	}

	fn read_value(&self, slab_id: usize) -> Result<(Vec<u8>, Vec<u8>), Error> {
		let mut k: Vec<u8> = vec![];
		let mut v: Vec<u8> = vec![];

		let mut slab = self.get_slab(u64!(slab_id))?;
		let bytes_per_slab = self.slab_size.saturating_sub(usize!(SLAB_OVERHEAD));
		let mut krem = usize!(u64::from_be_bytes(slab.get()[8..16].try_into()?));
		debug!("krem read = {},slab={:?}", krem, slab.get())?;
		let klen = krem;
		let mut slab_offset = 16;
		let mut value_len: usize = 0;
		let mut voffset = usize::MAX;
		let mut vrem = usize::MAX;
		loop {
			let slab_bytes = slab.get();
			debug!(
				"slab_id={}, krem={},voffset={},slab_offset={},bytes_per={}",
				slab.id(),
				krem,
				voffset,
				slab_offset,
				bytes_per_slab
			)?;
			if krem <= bytes_per_slab.saturating_sub(slab_offset + 8)
				&& voffset == usize::MAX
				&& (krem + slab_offset) <= bytes_per_slab
			{
				debug!(
					"reading valuelen {}-{}, slab={:?}",
					krem + slab_offset,
					krem + slab_offset + 8,
					slab_bytes,
				)?;
				value_len = usize!(u64::from_be_bytes(
					slab_bytes[krem + slab_offset..krem + slab_offset + 8].try_into()?
				));

				voffset = krem + 8 + slab_offset;
				vrem = value_len;
				debug!(
					"krem={},value_len={},voffset={},klen={}",
					krem, value_len, voffset, klen
				)?;
			}

			if voffset != usize::MAX {
				let mut end = (value_len - v.len()) + voffset;
				if end > bytes_per_slab {
					end = bytes_per_slab;
				}
				debug!("extend = {}-{}", voffset, end)?;
				v.extend(&slab_bytes[voffset..end]);
				debug!("vlen={}, vrem={}, end={}", v.len(), vrem, end)?;
				voffset = 0;
				vrem = vrem.saturating_sub(end - voffset);
			}

			if krem > 0 {
				let mut klen = krem;
				if krem > bytes_per_slab.saturating_sub(slab_offset) {
					klen = bytes_per_slab.saturating_sub(slab_offset);
				}
				k.extend(&slab_bytes[slab_offset..slab_offset + klen]);
			}

			if v.len() >= value_len && voffset != usize::MAX {
				break;
			}

			debug!(
				"kremin={},bytes_per_slab + slab_offset={}",
				krem,
				bytes_per_slab.saturating_sub(slab_offset)
			)?;

			krem = krem.saturating_sub(bytes_per_slab.saturating_sub(slab_offset));
			debug!("kremout={}", krem)?;
			let next =
				u64::from_be_bytes(slab_bytes[bytes_per_slab..bytes_per_slab + 8].try_into()?);
			slab = self.get_slab(next)?;
			slab_offset = 0;
		}

		debug!("break with klen={},vlen={}", k.len(), v.len())?;
		Ok((k, v))
	}

	fn find_entry<K>(
		&self,
		key: Option<&K>,
		key_raw: Option<&[u8]>,
		hash: u64,
	) -> Result<Option<usize>, Error>
	where
		K: Serializable + Hash,
	{
		let mut entry = match key {
			Some(key) => self.entry_hash(&key),
			None => usize!(hash) % self.entry_array.len(),
		};

		let max_iter = self.entry_array.len();
		let mut i = 0;
		loop {
			if i >= max_iter {
				debug!("max iter exceeded")?;
				return Ok(None);
			}
			if self.entry_array[entry] == SLOT_EMPTY {
				debug!("found empty slot at {}", entry)?;
				// empty slot means this key is not in the table
				return Ok(None);
			}

			if self.entry_array[entry] != SLOT_DELETED {
				debug!("found possible slot at {}", entry)?;
				// there's a valid entry here. Check if it's ours
				if self.key_match(self.entry_array[entry], key, key_raw)? {
					return Ok(Some(entry));
				}
			}

			entry = (entry + 1) % max_iter;
			i += 1;
		}
	}

	fn key_match<K>(
		&self,
		id: u64,
		key_ser: Option<&K>,
		key_raw: Option<&[u8]>,
	) -> Result<bool, Error>
	where
		K: Serializable,
	{
		// serialize key
		let mut k = vec![];
		match key_ser {
			Some(key_ser) => serialize(&mut k, key_ser)?,
			None => match key_raw {
				Some(key_raw) => {
					k.extend(key_raw);
				}
				None => {
					return Err(err!(
						ErrKind::IllegalArgument,
						"a serializable key or a raw key must be specified"
					))
				}
			},
		}
		let klen = k.len();
		debug!("key_len={}", klen)?;

		// read first slab
		let mut slab = self.get_slab(id)?;
		let len = u64::from_be_bytes(slab.get()[8..16].try_into()?);
		debug!("len={}", len)?;

		if len != u64!(klen) {
			return Ok(false);
		}

		let bytes_per_slab: usize = usize!(self.slab_size.saturating_sub(usize!(SLAB_OVERHEAD)));

		// compare the first slab
		let mut end = 16 + klen;
		if end > bytes_per_slab {
			end = bytes_per_slab;
		}
		if slab.get()[16..end] != k[0..end - 16] {
			return Ok(false);
		}
		if end < bytes_per_slab {
			return Ok(true);
		}

		let mut offset = end - 16;
		loop {
			let next =
				u64::from_be_bytes(slab.get()[bytes_per_slab..bytes_per_slab + 8].try_into()?);
			slab = self.get_slab(next)?;
			let mut rem = klen.saturating_sub(offset);
			if rem > bytes_per_slab {
				rem = bytes_per_slab;
			}

			if k[offset..offset + rem] != slab.get()[0..rem] {
				return Ok(false);
			}

			offset += bytes_per_slab;
			if offset >= klen {
				break;
			}
		}

		Ok(true)
	}

	fn free_tail(&mut self, mut slab_id: usize) -> Result<(), Error> {
		let bytes_per_slab = self.slab_size.saturating_sub(usize!(SLAB_OVERHEAD));
		info!("free tail id = {}", slab_id);
		let mut slab;
		loop {
			slab = self.get_slab(u64!(slab_id))?;
			let slab_bytes = slab.get();
			slab_id =
				usize::from_be_bytes(slab_bytes[bytes_per_slab..bytes_per_slab + 8].try_into()?);
			info!("next={}", slab_id);

			if slab_id == usize::MAX {
				break;
			}
		}
		Ok(())
	}

	fn insert_impl<K, V>(
		&mut self,
		key_ser: Option<&K>,
		hash: u64,
		key_raw: Option<&[u8]>,
		value_ser: Option<&V>,
		value_raw: Option<&[u8]>,
	) -> Result<(), Error>
	where
		K: Serializable + Hash,
		V: Serializable,
	{
		debug!("insert:self.config={:?},k={:?}", self.config, key_ser)?;

		let entry_array_len = self.entry_array.len();
		let mut entry = match key_ser {
			Some(key_ser) => self.entry_hash(key_ser),
			None => hash as usize % entry_array_len,
		};
		let mut i = 0;
		loop {
			if i >= entry_array_len {
				return Err(err!(
					ErrKind::CapacityExceeded,
					"StaticHashImpl: Capacity exceeded"
				));
			}
			if self.entry_array[entry] == SLOT_EMPTY || self.entry_array[entry] == SLOT_DELETED {
				break;
			}

			// does the current key match ours?
			if self.key_match(self.entry_array[entry], key_ser, key_raw)? {
				self.free_tail(usize!(self.entry_array[entry]))?;
				break;
			}

			entry = (entry + 1) % entry_array_len;
			i += 1;
		}
		debug!("inserting at entry={}", entry)?;

		let bytes_per_slab = self.slab_size.saturating_sub(usize!(SLAB_OVERHEAD));
		let free_count = u64!(self.get_free_count()?);

		let k_len_bytes: &[u8];
		let k_clone;
		let v_clone;
		let x;
		let y;
		let mut krem;
		let mut vrem;
		let v_len_bytes: &[u8];
		let k_bytes: &[u8];
		let v_bytes: &[u8];

		match key_ser {
			Some(key) => {
				debug!("serializing with key")?;
				let mut k = vec![];
				serialize(&mut k, key)?;
				k_clone = k.clone().to_vec();
				k_bytes = &(k_clone);
				krem = k.len();
				x = krem.to_be_bytes();
				k_len_bytes = &x;
			}
			None => match key_raw {
				Some(key_raw) => {
					k_bytes = &key_raw;
					krem = key_raw.len();
					x = krem.to_be_bytes();
					k_len_bytes = &x;
				}
				None => {
					return Err(err!(
						ErrKind::IllegalArgument,
						"both key_raw and key_ser cannot be None"
					));
				}
			},
		}

		match value_ser {
			Some(value) => {
				let mut v = vec![];
				serialize(&mut v, value)?;
				v_clone = v.clone();
				v_bytes = &v_clone;
				vrem = v.len();
				y = vrem.to_be_bytes();
				v_len_bytes = &y;
			}
			None => match value_raw {
				Some(value_raw) => {
					v_bytes = &value_raw;
					vrem = value_raw.len();
					y = vrem.to_be_bytes();
					v_len_bytes = &y;
				}
				None => {
					vrem = 0;
					y = vrem.to_be_bytes();
					v_len_bytes = &y;
					v_bytes = &[];
				}
			},
		}

		let needed_len = v_bytes.len() + k_bytes.len() + 16;
		let slabs_needed = needed_len as u64 / u64!(bytes_per_slab);

		debug!("slabs needed = {}", slabs_needed)?;

		if free_count < slabs_needed {
			return Err(err!(ErrKind::CapacityExceeded, "no more slabs"));
		}

		let mut itt = 0;
		let mut last_id = 0;
		let mut first_id = 0;

		let mut v_len_written = false;
		let first_entry = self.first_entry;
		loop {
			let id;
			{
				let mut slab = self.allocate()?;
				id = slab.id();
				let slab_mut = slab.get_mut();
				// mark the last bytes as pointing to usize::MAX. This will be
				// overwritten later if there's additional slabs
				slab_mut[bytes_per_slab..bytes_per_slab + 8]
					.clone_from_slice(&usize::MAX.to_be_bytes());
				if itt == 0 {
					first_id = id;

					// update iter list
					let first_entry_bytes = first_entry.to_be_bytes();
					debug!("febytes={:?}", first_entry_bytes)?;
					slab_mut[0..8].clone_from_slice(&first_entry_bytes);

					// write klen
					slab_mut[8..16].clone_from_slice(&k_len_bytes);

					// write k
					let mut klen = krem;
					if klen > bytes_per_slab - 16 {
						klen = bytes_per_slab - 16;
					}
					krem = krem.saturating_sub(klen);
					slab_mut[16..16 + klen].clone_from_slice(&k_bytes[0..klen]);

					// write v len if there's room
					if klen + 24 <= bytes_per_slab {
						slab_mut[16 + klen..24 + klen].clone_from_slice(&v_len_bytes);
						v_len_written = true;
					}

					// write v if there's room
					if klen + 24 < bytes_per_slab {
						let mut vlen = bytes_per_slab - (klen + 24);
						if vrem < vlen {
							vlen = vrem;
						}
						slab_mut[klen + 24..klen + 24 + vlen].clone_from_slice(&v_bytes[0..vlen]);
						vrem -= vlen;
					}
				} else {
					// write any remaining k
					let mut klen = krem;
					if krem > 0 {
						if klen > bytes_per_slab {
							klen = bytes_per_slab;
						}
						let k_offset = k_bytes.len() - krem;
						slab_mut[0..klen].clone_from_slice(&k_bytes[k_offset..k_offset + klen]);
						krem -= klen;
					}

					let mut value_written_adjustment = 0;
					// write vlen if we should and can
					if krem == 0 && !v_len_written && klen + 8 <= bytes_per_slab {
						slab_mut[klen..8 + klen].clone_from_slice(&v_len_bytes);
						v_len_written = true;
						value_written_adjustment = 8;
					}

					// if we can, write as much of v as possible
					if krem == 0
						&& v_len_written && klen + value_written_adjustment < bytes_per_slab
					{
						let mut vlen = vrem;
						if vlen > bytes_per_slab - (klen + value_written_adjustment) {
							vlen = bytes_per_slab - (klen + value_written_adjustment);
						}
						let v_offset = v_bytes.len() - vrem;
						slab_mut[klen + value_written_adjustment
							..klen + value_written_adjustment + vlen]
							.clone_from_slice(&v_bytes[v_offset..v_offset + vlen]);
						vrem -= vlen;
					}
				}
			}

			if itt == 0 {
				self.entry_array[entry] = id;
				last_id = id;
			} else {
				let mut slab = self.get_mut(last_id)?;
				slab.get_mut()[bytes_per_slab..bytes_per_slab + 8]
					.clone_from_slice(&id.to_be_bytes());
				last_id = id;
			}
			itt += 1;
			if vrem == 0 && krem == 0 && v_len_written {
				break;
			}
		}
		debug!(
			"first_entry = {}, setting to = {}",
			self.first_entry, first_id
		)?;
		self.first_entry = usize!(first_id);
		Ok(())
	}

	fn remove_impl<K>(&mut self, key: &K) -> Result<(), Error>
	where
		K: Serializable,
	{
		debug!("remove:self.config={:?},k={:?}", self.config, key)?;
		todo!();
	}

	fn iter_impl<K>(&self) -> Result<Box<dyn StaticIterator<'_, K>>, Error>
	where
		K: Serializable,
	{
		debug!("iter:self.config={:?}", self.config)?;
		todo!()
	}

	fn entry_hash<K: Hash>(&self, key: &K) -> usize {
		let mut hasher = DefaultHasher::new();
		key.hash(&mut hasher);
		let hash = hasher.finish() as usize;

		let max_iter = self.entry_array.len();

		hash % max_iter
	}
}

pub struct StaticHashtableBuilder {}

impl StaticHashtableBuilder {
	pub fn build<K, V>(
		config: StaticHashtableConfig,
		slabs: Option<Box<dyn SlabAllocator + Send + Sync>>,
	) -> Result<Box<dyn StaticHashtable<K, V>>, Error>
	where
		K: Serializable + Hash,
		V: Serializable,
	{
		Ok(Box::new(StaticHashImpl::new(config.into(), slabs)?))
	}
}

pub struct StaticHashsetBuilder {}

impl StaticHashsetBuilder {
	pub fn build<K>(
		config: StaticHashsetConfig,
		slabs: Option<Box<dyn SlabAllocator + Send + Sync>>,
	) -> Result<Box<dyn StaticHashset<K>>, Error>
	where
		K: Serializable + Hash,
	{
		Ok(Box::new(StaticHashImpl::new(config.into(), slabs)?))
	}
}

#[cfg(test)]
mod test {
	use crate::types::SlabAllocatorConfig;
	use crate::types::{Reader, Writer};
	use crate::{
		Serializable, SlabAllocatorBuilder, StaticHashsetBuilder, StaticHashsetConfig,
		StaticHashtableBuilder, StaticHashtableConfig,
	};
	use bmw_err::Error;
	use bmw_log::*;
	use std::collections::hash_map::DefaultHasher;
	use std::hash::{Hash, Hasher};

	debug!();

	pub fn initialize() -> Result<(), Error> {
		let _ = debug!("init");
		crate::GLOBAL_SLAB_ALLOCATOR.with(|f| -> Result<(), Error> {
			let sa = unsafe { f.get().as_mut().unwrap() };
			sa.init(SlabAllocatorConfig {
				slab_count: 30_000,
				..Default::default()
			})?;
			Ok(())
		})
	}

	#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
	struct BigThing {
		val1: u32,
		val2: u32,
		middle: Vec<u8>,
	}

	impl BigThing {
		fn new(val1: u32, val2: u32) -> Self {
			let mut middle = vec![];
			for i in 0..val1 {
				middle.push((i % 256) as u8);
			}
			Self { val1, val2, middle }
		}
	}

	impl Serializable for BigThing {
		fn read<R: Reader>(reader: &mut R) -> Result<Self, Error> {
			let val1 = reader.read_u32()?;
			let mut middle = vec![];
			for _ in 0..val1 {
				middle.push(reader.read_u8()?);
			}
			let val2 = reader.read_u32()?;
			Ok(Self { val1, val2, middle })
		}
		fn write<W: Writer>(&self, writer: &mut W) -> Result<(), Error> {
			writer.write_u32(self.val1)?;
			for i in 0..self.val1 {
				writer.write_u8(self.middle[i as usize])?;
			}
			writer.write_u32(self.val2)?;
			Ok(())
		}
	}

	#[test]
	fn test_small_static_hashtable() -> Result<(), Error> {
		let mut slabs = SlabAllocatorBuilder::build();
		slabs.init(SlabAllocatorConfig::default())?;
		let mut sh = StaticHashtableBuilder::build(StaticHashtableConfig::default(), Some(slabs))?;
		sh.insert(&1, &2)?;
		assert_eq!(sh.get(&1)?, Some(2));

		let mut sh2 = StaticHashtableBuilder::build(StaticHashtableConfig::default(), None)?;
		for i in 0..20 {
			sh2.insert(&BigThing::new(i, i), &BigThing::new(i, i))?;
			info!("i={}", i)?;
			assert_eq!(sh2.get(&BigThing::new(i, i))?, Some(BigThing::new(i, i)));
		}

		let mut sh3 = StaticHashtableBuilder::build(StaticHashtableConfig::default(), None)?;
		sh3.insert(&10, &20)?;
		assert_eq!(sh3.get(&10)?, Some(20));

		let mut count = 0;
		let mut ks = vec![];
		let mut vs = vec![];
		for (x, y) in &sh2 {
			info!("x={:?},y={:?}", x, y)?;
			ks.push(x);
			vs.push(y);
			count += 1;
		}
		assert_eq!(count, 20);
		ks.sort();
		vs.sort();
		for i in 0..20 {
			assert_eq!(ks[i].val1, i as u32);
			assert_eq!(vs[i].val1, i as u32);
			assert_eq!(ks[i].val2, i as u32);
			assert_eq!(vs[i].val2, i as u32);
		}
		sh2.insert(&BigThing::new(8, 3), &BigThing::new(1, 3))?;

		Ok(())
	}

	#[test]
	fn test_hashtable_replace() -> Result<(), Error> {
		/*
		let mut sh = StaticHashtableBuilder::build(StaticHashtableConfig::default(), None)?;
		sh.insert(&1, &2)?;
		assert_eq!(sh.get(&1)?, Some(2));
		sh.insert(&1, &3)?;
		assert_eq!(sh.get(&1)?, Some(3));
		let mut count = 0;
		for (k, v) in &sh {
			info!("k={:?},v={:?}", k, v)?;
			count += 1;
		}
		assert_eq!(count, 1);
			*/
		Ok(())
	}

	#[test]
	fn test_static_hashtable() -> Result<(), Error> {
		initialize()?;
		let mut slabs1 = SlabAllocatorBuilder::build();
		slabs1.init(SlabAllocatorConfig {
			slab_count: 30_000,
			..Default::default()
		})?;
		let mut slabs2 = SlabAllocatorBuilder::build();
		slabs2.init(SlabAllocatorConfig::default())?;
		let mut sh = StaticHashtableBuilder::build(StaticHashtableConfig::default(), None)?;
		sh.insert(&1, &2)?;
		assert_eq!(sh.get(&1)?, Some(2));

		let mut sh2 = StaticHashtableBuilder::build(StaticHashtableConfig::default(), None)?;

		for i in 0..4000 {
			info!("i={}", i)?;
			sh2.insert(&BigThing::new(i, i), &BigThing::new(i, i))?;
			assert_eq!(sh2.get(&BigThing::new(i, i))?, Some(BigThing::new(i, i)));
			//info!("bigthing={:?}", BigThing::new(i, i));
		}

		let mut sh3 = StaticHashtableBuilder::build(StaticHashtableConfig::default(), None)?;
		sh3.insert(&10, &20)?;
		assert_eq!(sh3.get(&10)?, Some(20));

		Ok(())
	}

	#[test]
	fn test_static_hashset() -> Result<(), Error> {
		initialize()?;
		let mut slabs1 = SlabAllocatorBuilder::build();
		slabs1.init(SlabAllocatorConfig::default())?;
		let mut sh = StaticHashsetBuilder::build(StaticHashsetConfig::default(), None)?;
		sh.insert(&1u32)?;
		sh.insert(&9)?;
		sh.insert(&18)?;

		for i in 0..20 {
			if i == 1 || i == 9 || i == 18 {
				assert!(sh.contains(&i)?);
			} else {
				assert!(!sh.contains(&i)?);
			}
		}
		Ok(())
	}

	#[test]
	fn test_hashtable_raw() -> Result<(), Error> {
		initialize()?;
		let mut slabs1 = SlabAllocatorBuilder::build();
		slabs1.init(SlabAllocatorConfig::default())?;
		let mut sh =
			StaticHashtableBuilder::build::<(), ()>(StaticHashtableConfig::default(), None)?;

		let mut hasher = DefaultHasher::new();
		(b"hi").hash(&mut hasher);
		let hash = hasher.finish();
		sh.insert_raw(b"hi", hash, b"ok")?;
		let slab = sh.get_raw(b"hi", hash)?.unwrap();
		// key = 104/105 (hi), value = 111/107 (ok)
		assert_eq!(
			slab.get()[0..28],
			[
				255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0, 2, 104, 105, 0, 0, 0,
				0, 0, 0, 0, 2, 111, 107
			]
		);
		Ok(())
	}
}
