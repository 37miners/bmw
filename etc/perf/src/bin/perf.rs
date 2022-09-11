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

use bmw_deps::num_format::{Locale, ToFormattedString};
use bmw_err::*;
use bmw_evh::*;
use bmw_log::LogConfigOption::*;
use bmw_log::*;
use bmw_util::*;
use clap::{load_yaml, App, ArgMatches};
use std::net::TcpStream;
use std::sync::mpsc::sync_channel;
use std::thread::{sleep, spawn};
use std::time::{Duration, Instant};

#[cfg(unix)]
use std::os::unix::io::IntoRawFd;
#[cfg(windows)]
use std::os::windows::io::IntoRawSocket;

info!();

// include build information
pub mod built_info {
	include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

const DICTIONARY: &[u8] = b"abcdefghijklmnopqrstuvwx";

#[derive(Clone, Debug)]
struct ThreadState {
	itt: usize,
	count: usize,
	last: usize,
}

impl ThreadState {
	fn new() -> Self {
		Self {
			itt: 0,
			count: 0,
			last: 0,
		}
	}
}

fn run_eventhandler(args: ArgMatches) -> Result<(), Error> {
	let threads: usize = match args.is_present("threads") {
		true => args.value_of("threads").unwrap().parse()?,
		false => 1,
	};
	let port = match args.is_present("port") {
		true => args.value_of("port").unwrap().parse()?,
		false => 8081,
	};
	info!("Using port: {}", port)?;
	let addr = &format!("127.0.0.1:{}", port)[..];
	let config = EventHandlerConfig {
		threads,
		housekeeping_frequency_millis: 10_000,
		read_slab_count: 20,
		max_handles_per_thread: 30,
		..Default::default()
	};
	let mut evh = bmw_evh::Builder::build_evh(config)?;

	evh.set_on_read(move |conn_data, _thread_context| {
		debug!("on read fs = {}", conn_data.slab_offset())?;
		let first_slab = conn_data.first_slab();
		let last_slab = conn_data.last_slab();
		let slab_offset = conn_data.slab_offset();
		let res = conn_data.borrow_slab_allocator(move |sa| {
			let mut slab_id = first_slab;
			let mut ret: Vec<u8> = vec![];
			loop {
				let slab = sa.get(slab_id.try_into()?)?;
				let slab_bytes = slab.get();
				debug!("read bytes = {:?}", &slab.get()[0..slab_offset as usize])?;
				if slab_id != last_slab {
					ret.extend(&slab_bytes[0..READ_SLAB_DATA_SIZE as usize]);
				} else {
					ret.extend(&slab_bytes[0..slab_offset as usize]);
					break;
				}
				slab_id = u32::from_be_bytes(try_into!(
					slab_bytes[READ_SLAB_DATA_SIZE..READ_SLAB_DATA_SIZE + 4]
				)?);
			}
			Ok(ret)
		})?;
		conn_data.clear_through(last_slab)?;
		conn_data.write_handle().write(&res)?;
		debug!("res={:?}", res)?;
		Ok(())
	})?;

	evh.set_on_accept(move |conn_data, _thread_context| {
		debug!(
			"accept a connection handle = {}, tid={}",
			conn_data.get_handle(),
			conn_data.tid()
		)?;
		Ok(())
	})?;
	evh.set_on_close(move |conn_data, _thread_context| {
		debug!(
			"on close: {}/{}",
			conn_data.get_handle(),
			conn_data.get_connection_id()
		)?;
		Ok(())
	})?;
	evh.set_on_panic(move |_thread_context| Ok(()))?;
	evh.set_housekeeper(move |_thread_context| Ok(()))?;

	evh.start()?;
	let handles = create_listeners(threads, addr, 10)?;
	debug!("handles.size={},handles={:?}", handles.size(), handles)?;
	let sc = ServerConnection {
		tls_config: None,
		handles,
		is_reuse_port: true,
	};
	evh.add_server(sc)?;

	std::thread::park();

	Ok(())
}

fn run_client(args: ArgMatches) -> Result<(), Error> {
	let port = match args.is_present("port") {
		true => args.value_of("port").unwrap().parse()?,
		false => 8081,
	};
	let itt: usize = match args.is_present("itt") {
		true => args.value_of("itt").unwrap().parse()?,
		false => 1_000,
	};
	let count: usize = match args.is_present("count") {
		true => args.value_of("count").unwrap().parse()?,
		false => 10,
	};
	let clients: usize = match args.is_present("clients") {
		true => args.value_of("clients").unwrap().parse()?,
		false => 1,
	};
	let threads: usize = match args.is_present("threads") {
		true => args.value_of("threads").unwrap().parse()?,
		false => 1,
	};

	info!("itt={},count={},clients={}", itt, count, clients)?;

	info!("Using port: {}", port)?;
	let addr = format!("127.0.0.1:{}", port);
	let config = EventHandlerConfig {
		threads: 1,
		housekeeping_frequency_millis: 10_000,
		read_slab_count: 10000,
		max_handles_per_thread: 300,
		..Default::default()
	};

	let pool = thread_pool!(MinSize(threads), MaxSize(threads))?;
	let mut completions = vec![];
	let mut state = array!(threads, &lock_box!(ThreadState::new())?)?;
	let mut state_clone = state.clone();
	for i in 0..threads {
		state[i] = lock_box!(ThreadState::new())?;
		state_clone[i] = state[i].clone();
		let local_state = state[i].clone();
		let addr = addr.clone();
		let config = config.clone();
		completions.push(execute!(pool, {
			run_thread(&config, addr, itt, count, clients, i, local_state)
		})?);
	}

	spawn(move || -> Result<(), Error> {
		loop {
			sleep(Duration::from_millis(1000));
			info_plain!("--------------------------------------------------------------------------------------------------------------------------------")?;
			for i in 0..threads {
				let state = state_clone[i].rlock()?;
				let guard = state.guard();
				info!("state[{}]={:?}", i, **guard)?;
			}
		}
	});

	for i in 0..completions.len() {
		block_on!(completions[i]);
	}

	Ok(())
}

fn run_thread(
	config: &EventHandlerConfig,
	addr: String,
	itt: usize,
	count: usize,
	clients: usize,
	tid: usize,
	mut state: Box<dyn LockBox<ThreadState>>,
) -> Result<(), Error> {
	let state_clone = state.clone();
	let total = count * itt * clients;
	let mut evh = bmw_evh::Builder::build_evh(config.clone())?;

	let recv_count = lock_box!((0usize, 0u8))?;
	let mut recv_count_clone = recv_count.clone();
	let (tx, rx) = sync_channel(1);
	let sender = lock_box!(tx)?;

	evh.set_on_read(move |conn_data, _thread_context| {
		debug!("on read fs = {}", conn_data.slab_offset())?;
		let first_slab = conn_data.first_slab();
		let slab_offset = conn_data.slab_offset();
		let last_slab = conn_data.last_slab();
		let mut sender = sender.clone();
		let mut recv_count = recv_count.clone();
		let mut state_clone = state_clone.clone();
		conn_data.borrow_slab_allocator(move |sa| {
			let mut slab_id = first_slab;
			loop {
				let slab = sa.get(slab_id.try_into()?)?;
				let slab_bytes = slab.get();
				let offset = if slab_id == last_slab {
					slab_offset as usize
				} else {
					READ_SLAB_DATA_SIZE
				};

				debug!("read bytes[{}] = {:?}", offset, &slab_bytes[0..offset])?;
				{
					let mut recv_count = recv_count.wlock()?;
					let guard = recv_count.guard();
					if (**guard).1 != 0 {
						if (**guard).1 != 'x' as u8 {
							if slab_bytes[0] != (**guard).1 + 1 {
								info!(
									"ne. o={},rc={},si={},f={},l={}",
									offset,
									(**guard).0,
									slab_id,
									first_slab,
									last_slab
								)?;
							}
							assert_eq!(slab_bytes[0], (**guard).1 + 1);
						} else {
							if slab_bytes[0] != 'a' as u8 {
								info!(
									"ne. o={},r={},s={},f={},l={}",
									offset,
									(**guard).0,
									slab_id,
									first_slab,
									last_slab
								)?;
							}
							assert_eq!(slab_bytes[0], 'a' as u8);
						}
					}
					for i in 1..offset {
						if slab_bytes[i - 1] != 'x' as u8 {
							assert_eq!(slab_bytes[i - 1] + 1, slab_bytes[i]);
						} else {
							assert_eq!(slab_bytes[i], 'a' as u8);
						}
					}
					(**guard).0 += offset as usize;
					(**guard).1 = slab_bytes[offset - 1];
					if (**guard).0 == count * 4 * clients {
						let mut tx = sender.wlock()?;
						(**tx.guard()).send(1)?;
					}
				}

				{
					let mut state = state_clone.wlock()?;
					let guard = state.guard();
					(**guard).count += offset as usize;
					(**guard).last = offset as usize;
				}
				if slab_id == last_slab {
					break;
				} else {
					slab_id = u32::from_be_bytes(try_into!(
						slab_bytes[READ_SLAB_DATA_SIZE..READ_SLAB_DATA_SIZE + 4]
					)?);
				}
			}
			Ok(())
		})?;

		conn_data.clear_through(last_slab)?;
		Ok(())
	})?;

	evh.set_on_accept(move |conn_data, _thread_context| {
		debug!(
			"accept a connection handle = {}, tid={}",
			conn_data.get_handle(),
			conn_data.tid()
		)?;
		Ok(())
	})?;
	evh.set_on_close(move |conn_data, _thread_context| {
		debug!(
			"on close: {}/{}",
			conn_data.get_handle(),
			conn_data.get_connection_id()
		)?;
		Ok(())
	})?;
	evh.set_on_panic(move |_thread_context| Ok(()))?;
	evh.set_housekeeper(move |_thread_context| Ok(()))?;
	evh.start()?;

	let mut whs = vec![];
	for _ in 0..clients {
		let connection = TcpStream::connect(addr.clone())?;
		#[cfg(unix)]
		let connection_handle = connection.into_raw_fd();
		#[cfg(windows)]
		let connection_handle = connection.into_raw_socket().try_into()?;

		let client = ClientConnection {
			handle: connection_handle,
			tls_config: None,
		};
		let wh = evh.add_client(client)?;
		whs.push(wh);
	}

	let now = Instant::now();
	let dictionary_len = DICTIONARY.len();
	for _ in 0..itt {
		let mut dict_offset = 0;
		for _ in 0..count {
			for wh in &mut whs {
				wh.write(&DICTIONARY[dict_offset..dict_offset + 4])?;
			}
			dict_offset += 4;
			if dict_offset >= dictionary_len {
				dict_offset = 0;
			}
		}

		rx.recv()?;
		{
			let mut state = state.wlock()?;
			let guard = state.guard();
			(**guard).itt += 1;
			(**guard).count = 0;
		}
		{
			let mut recv_count = recv_count_clone.wlock()?;
			let guard = recv_count.guard();
			(**guard) = (0, 0);
		}
	}
	let elapsed = now.elapsed();
	let elapsed_nanos = elapsed.as_nanos() as f64;
	let qps = ((itt * count * clients) as f64 / elapsed_nanos) * 1_000_000_000.0;
	info!(
		"thread {} received {} messages in {:?}, QPS = {:.2}",
		tid,
		total.to_formatted_string(&Locale::en),
		elapsed,
		qps,
	)?;

	Ok(())
}

fn main() -> Result<(), Error> {
	global_slab_allocator!()?;
	log_init!(LogConfig {
		show_bt: ShowBt(false),
		..Default::default()
	})?;

	let yml = load_yaml!("perf.yml");
	let args = App::from_yaml(yml)
		.version(built_info::PKG_VERSION)
		.get_matches();

	let client = args.is_present("client");
	let eventhandler = args.is_present("eventhandler");

	if client {
		info!("Starting perf client")?;
		run_client(args)?;
	} else if eventhandler {
		info!("Starting eventhandler")?;
		run_eventhandler(args)?;
	}

	Ok(())
}
