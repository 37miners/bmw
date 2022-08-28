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

use bmw_log::*;

info!();

#[macro_export]
macro_rules! init_slab_allocator {
( $( $config:expr ),* ) => {{
            let mut config = bmw_util::SlabAllocatorConfig::default();
            let mut error: Option<String> = None;
            let mut slab_size_specified = false;
            let mut slab_count_specified = false;

            // compiler sees macro as not used if it's not used in one part of the code
            // these lines make the warnings go away
            if config.slab_size == 0 { config.slab_size = 0; }
            if slab_count_specified { slab_count_specified = false; }
            if slab_size_specified { slab_size_specified = false; }
            if slab_count_specified {}
            if slab_size_specified {}
            if error.is_some() { error = None; }

            $(
                match $config {
                    bmw_util::ConfigOption::SlabSize(slab_size) => {
                        config.slab_size = slab_size;

                        if slab_size_specified {
                            error = Some("SlabSize was specified more than once!".to_string());
                        }
                        slab_size_specified = true;
                        if slab_size_specified {}

                    },
                    bmw_util::ConfigOption::SlabCount(slab_count) => {
                        config.slab_count = slab_count;

                        if slab_count_specified {
                            error = Some("SlabCount was specified more than once!".to_string());
                        }

                        slab_count_specified = true;
                        if slab_count_specified {}
                    },
                    _ => {
                        error = Some(format!("'{:?}' is not allowed for hashset", $config));
                    }
                }
            )*

            match error {
                Some(error) => Err(bmw_err::err!(bmw_err::ErrKind::Configuration, error)),
                None => {
                        bmw_util::GLOBAL_SLAB_ALLOCATOR.with(|f| -> Result<(), Error> {
                        unsafe {
                                f.get().as_mut().unwrap().init(config)?;
                                Ok(())
                        }
                        })
                }
            }
        }
    }
}

#[macro_export]
macro_rules! slab_allocator {
( $( $config:expr ),* ) => {{
            let slabs = bmw_util::Builder::build_slabs_ref();
            let mut config = bmw_util::SlabAllocatorConfig::default();
            let mut error: Option<String> = None;
            let mut slab_size_specified = false;
            let mut slab_count_specified = false;

            // compiler sees macro as not used if it's not used in one part of the code
            // these lines make the warnings go away
            if config.slab_size == 0 { config.slab_size = 0; }
            if slab_count_specified { slab_count_specified = false; }
            if slab_size_specified { slab_size_specified = false; }
            if slab_count_specified {}
            if slab_size_specified {}
            if error.is_some() { error = None; }

            $(
                match $config {
                    bmw_util::ConfigOption::SlabSize(slab_size) => {
                        config.slab_size = slab_size;

                        if slab_size_specified {
                            error = Some("SlabSize was specified more than once!".to_string());
                        }
                        slab_size_specified = true;
                        if slab_size_specified {}

                    },
                    bmw_util::ConfigOption::SlabCount(slab_count) => {
                        config.slab_count = slab_count;

                        if slab_count_specified {
                            error = Some("SlabCount was specified more than once!".to_string());
                        }

                        slab_count_specified = true;
                        if slab_count_specified {}
                    },
                    _ => {
                        error = Some(format!("'{:?}' is not allowed for hashset", $config));
                    }
                }
            )*
            match error {
                Some(error) => Err(bmw_err::err!(bmw_err::ErrKind::Configuration, error)),
                None => {
                        {
                                let mut slabs: std::cell::RefMut<_> = slabs.borrow_mut();
                                slabs.init(config)?;
                        }
                        Ok(slabs)
                },
            }
     }};
}

#[macro_export]
macro_rules! hashtable_config {
	( $config_list:expr ) => {{
		let mut config = bmw_util::HashtableConfig::default();
		let mut error: Option<String> = None;
		let mut max_entries_specified = false;
		let mut max_load_factor_specified = false;

		// compiler sees macro as not used if it's not used in one part of the code
		// these lines make the warnings go away
		if config.max_entries == 0 {
			config.max_entries = 0;
		}
		if max_entries_specified {
			max_entries_specified = false;
		}
		if max_load_factor_specified {
			max_load_factor_specified = false;
		}
		if max_load_factor_specified {}
		if max_entries_specified {}
		if error.is_some() {
			error = None;
		}

		for config_item in $config_list.iter() {
			match config_item {
				bmw_util::ConfigOption::MaxEntries(max_entries) => {
					config.max_entries = max_entries;

					if max_entries_specified {
						error = Some("MaxEntries was specified more than once!".to_string());
					}

					max_entries_specified = true;
					if max_entries_specified {}
				}
				bmw_util::ConfigOption::MaxLoadFactor(max_load_factor) => {
					config.max_load_factor = max_load_factor;

					if max_load_factor_specified {
						error = Some("MaxLoadFactor was specified more than once!".to_string());
					}

					max_load_factor_specified = true;
					if max_load_factor_specified {}
				}
				_ => {
					error = Some(format!("'{:?}' is not allowed for hashtable", config_item));
				}
			}
		}

		match error {
			Some(error) => (Some(error), None),
			None => (None, Some(config)),
		}
	}};
}

#[macro_export]
macro_rules! hashtable_sync_config {
	( $config_list:expr ) => {{
		let mut slab_config = bmw_util::SlabAllocatorConfig::default();
		let mut config = bmw_util::HashtableConfig::default();
		let mut error: Option<String> = None;
		let mut max_entries_specified = false;
		let mut max_load_factor_specified = false;
		let mut slab_size_specified = false;
		let mut slab_count_specified = false;

		// compiler sees macro as not used if it's not used in one part of the code
		// these lines make the warnings go away
		if config.max_entries == 0 {
			config.max_entries = 0;
		}
		if max_entries_specified {
			max_entries_specified = false;
		}
		if max_load_factor_specified {
			max_load_factor_specified = false;
		}
		if max_load_factor_specified {}
		if max_entries_specified {}
		if error.is_some() {
			error = None;
		}

		for config_item in $config_list.iter() {
			match config_item {
				bmw_util::ConfigOption::MaxEntries(max_entries) => {
					config.max_entries = max_entries;

					if max_entries_specified {
						error = Some("MaxEntries was specified more than once!".to_string());
					}

					max_entries_specified = true;
					if max_entries_specified {}
				}
				bmw_util::ConfigOption::MaxLoadFactor(max_load_factor) => {
					config.max_load_factor = max_load_factor;

					if max_load_factor_specified {
						error = Some("MaxLoadFactor was specified more than once!".to_string());
					}

					max_load_factor_specified = true;
					if max_load_factor_specified {}
				}
				bmw_util::ConfigOption::SlabSize(slab_size) => {
					slab_config.slab_size = slab_size;

					if slab_size_specified {
						error = Some("SlabSize was specified more than once!".to_string());
					}
					slab_size_specified = true;
					if slab_size_specified {}
				}
				bmw_util::ConfigOption::SlabCount(slab_count) => {
					slab_config.slab_count = slab_count;

					if slab_count_specified {
						error = Some("SlabCount was specified more than once!".to_string());
					}

					slab_count_specified = true;
					if slab_count_specified {}
				}
				_ => {
					error = Some(format!("'{:?}' is not allowed for hashtable", config_item));
				}
			}
		}

		match error {
			Some(error) => (Some(error), None, None),
			None => (None, Some(config), Some(slab_config)),
		}
	}};
}

#[macro_export]
macro_rules! hashset_config {
	( $config_list:expr ) => {{
		let mut config = bmw_util::HashsetConfig::default();
		let mut error: Option<String> = None;
		let mut max_entries_specified = false;
		let mut max_load_factor_specified = false;

		// compiler sees macro as not used if it's not used in one part of the code
		// these lines make the warnings go away
		if config.max_entries == 0 {
			config.max_entries = 0;
		}
		if max_entries_specified {
			max_entries_specified = false;
		}
		if max_load_factor_specified {
			max_load_factor_specified = false;
		}
		if max_load_factor_specified {}
		if max_entries_specified {}
		if error.is_some() {
			error = None;
		}

		for config_item in $config_list.iter() {
			match config_item {
				bmw_util::ConfigOption::MaxEntries(max_entries) => {
					config.max_entries = max_entries;

					if max_entries_specified {
						error = Some("MaxEntries was specified more than once!".to_string());
					}

					max_entries_specified = true;
					if max_entries_specified {}
				}
				bmw_util::ConfigOption::MaxLoadFactor(max_load_factor) => {
					config.max_load_factor = max_load_factor;

					if max_load_factor_specified {
						error = Some("MaxLoadFactor was specified more than once!".to_string());
					}

					max_load_factor_specified = true;
					if max_load_factor_specified {}
				}
				_ => {
					error = Some(format!("'{:?}' is not allowed for hashset", config_item));
				}
			}
		}

		match error {
			Some(error) => (Some(error), None),
			None => (None, Some(config)),
		}
	}};
}

#[macro_export]
macro_rules! hashset_sync_config {
	( $config_list:expr ) => {{
		let mut slab_config = bmw_util::SlabAllocatorConfig::default();
		let mut config = bmw_util::HashsetConfig::default();
		let mut error: Option<String> = None;
		let mut max_entries_specified = false;
		let mut max_load_factor_specified = false;
		let mut slab_size_specified = false;
		let mut slab_count_specified = false;

		// compiler sees macro as not used if it's not used in one part of the code
		// these lines make the warnings go away
		if config.max_entries == 0 {
			config.max_entries = 0;
		}
		if max_entries_specified {
			max_entries_specified = false;
		}
		if max_load_factor_specified {
			max_load_factor_specified = false;
		}
		if max_load_factor_specified {}
		if max_entries_specified {}
		if error.is_some() {
			error = None;
		}

		for config_item in $config_list.iter() {
			match config_item {
				bmw_util::ConfigOption::MaxEntries(max_entries) => {
					config.max_entries = max_entries;

					if max_entries_specified {
						error = Some("MaxEntries was specified more than once!".to_string());
					}

					max_entries_specified = true;
					if max_entries_specified {}
				}
				bmw_util::ConfigOption::MaxLoadFactor(max_load_factor) => {
					config.max_load_factor = max_load_factor;

					if max_load_factor_specified {
						error = Some("MaxLoadFactor was specified more than once!".to_string());
					}

					max_load_factor_specified = true;
					if max_load_factor_specified {}
				}
				bmw_util::ConfigOption::SlabSize(slab_size) => {
					slab_config.slab_size = slab_size;

					if slab_size_specified {
						error = Some("SlabSize was specified more than once!".to_string());
					}
					slab_size_specified = true;
					if slab_size_specified {}
				}
				bmw_util::ConfigOption::SlabCount(slab_count) => {
					slab_config.slab_count = slab_count;

					if slab_count_specified {
						error = Some("SlabCount was specified more than once!".to_string());
					}

					slab_count_specified = true;
					if slab_count_specified {}
				}
				_ => {
					error = Some(format!("'{:?}' is not allowed for hashset", config_item));
				}
			}
		}

		match error {
			Some(error) => (Some(error), None, None),
			None => (None, Some(config), Some(slab_config)),
		}
	}};
}

#[macro_export]
macro_rules! hashtable {
	( $( $config:expr ),* ) => {{
                use bmw_util::List;

                let mut slabs: Option<std::rc::Rc<std::cell::RefCell<dyn bmw_util::SlabAllocator>>> = None;
                if slabs.is_some() {
                    slabs = None;
                }
                $(
                    match $config {
                        bmw_util::ConfigOption::Slabs(s) => {
                            slabs = Some(s);
                        }
                        _ => {
                        }
                    }
                )*

                let mut config_list = bmw_util::Builder::build_list(
                        bmw_util::ListConfig::default(),
                        slabs.clone()
                )?;

                // delete_head is called to remove compiler warning about not needing to be mutable.
                // It does need to be mutable.
                config_list.delete_head()?;


                $(
                    match $config {
                        bmw_util::ConfigOption::Slabs(_) => {
                        }
                        _ => {
                            config_list.push($config)?;
                        }
                    }
                )*

		let (error, config) = bmw_util::hashtable_config!(config_list);
		match error {
			Some(error) => Err(bmw_err::err!(bmw_err::ErrKind::Configuration, error)),
			None => bmw_util::Builder::build_hashtable(config.unwrap(), slabs),
		}
	}};
}

#[macro_export]
macro_rules! hashtable_box {
        ( $( $config:expr ),* ) => {{
                use bmw_util::List;

                let mut slabs: Option<std::rc::Rc<std::cell::RefCell<dyn bmw_util::SlabAllocator>>> = None;
                if slabs.is_some() {
                    slabs = None;
                }
                $(
                    match $config {
                        bmw_util::ConfigOption::Slabs(s) => {
                            slabs = Some(s);
                        }
                        _ => {
                        }
                    }
                )*

                let mut config_list = bmw_util::Builder::build_list(
                        bmw_util::ListConfig::default(),
                        slabs.clone()
                )?;

                // delete_head is called to remove compiler warning about not needing to be mutable.
                // It does need to be mutable.
                config_list.delete_head()?;


                $(
                    match $config {
                        bmw_util::ConfigOption::Slabs(_) => {
                        }
                        _ => {
                            config_list.push($config)?;
                        }
                    }
                )*

                let (error, config) = bmw_util::hashtable_config!(config_list);
                match error {
                        Some(error) => Err(bmw_err::err!(bmw_err::ErrKind::Configuration, error)),
                        None => bmw_util::Builder::build_hashtable_box(config.unwrap(), slabs),
                }
        }};
}

#[macro_export]
macro_rules! hashtable_sync {
        ( $( $config:expr ),* ) => {{
                use bmw_util::List;

                $(
                    match $config {
                        bmw_util::ConfigOption::Slabs(_) => {
                            return Err(bmw_err::err!(
                                    bmw_err::ErrKind::Configuration,
                                    "Slabs cannot be specified with hashtable_sync"
                            ));
                        }
                        _ => {
                        }
                    }
                )*

                let mut config_list = bmw_util::Builder::build_list(
                        bmw_util::ListConfig::default(),
                        None
                )?;

                // delete_head is called to remove compiler warning about not needing to be mutable.
                // It does need to be mutable.
                config_list.delete_head()?;

                $(
                    match $config {
                        bmw_util::ConfigOption::Slabs(_) => {
                        }
                        _ => {
                            config_list.push($config)?;
                        }
                    }
                )*

                let (error, config, slab_config) = bmw_util::hashtable_sync_config!(config_list);
                match error {
                        Some(error) => Err(bmw_err::err!(bmw_err::ErrKind::Configuration, error)),
                        None => bmw_util::Builder::build_hashtable_sync(config.unwrap(), slab_config.unwrap()),
                }
        }};
}

#[macro_export]
macro_rules! hashtable_sync_box {
        ( $( $config:expr ),* ) => {{
                use bmw_util::List;

                $(
                    match $config {
                        bmw_util::ConfigOption::Slabs(_) => {
                            return Err(bmw_err::err!(
                                    bmw_err::ErrKind::Configuration,
                                    "Slabs cannot be specified with hashtable_sync"
                            ));
                        }
                        _ => {
                        }
                    }
                )*

                let mut config_list = bmw_util::Builder::build_list(
                        bmw_util::ListConfig::default(),
                        None
                )?;

                // delete_head is called to remove compiler warning about not needing to be mutable.
                // It does need to be mutable.
                config_list.delete_head()?;

                $(
                    match $config {
                        bmw_util::ConfigOption::Slabs(_) => {
                        }
                        _ => {
                            config_list.push($config)?;
                        }
                    }
                )*

                let (error, config, slab_config) = bmw_util::hashtable_sync_config!(config_list);
                match error {
                        Some(error) => Err(bmw_err::err!(bmw_err::ErrKind::Configuration, error)),
                        None => bmw_util::Builder::build_hashtable_sync_box(config.unwrap(), slab_config.unwrap()),
                }
        }};
}

#[macro_export]
macro_rules! hashset {
        ( $( $config:expr ),* ) => {{
                use bmw_util::List;

                let mut slabs: Option<std::rc::Rc<std::cell::RefCell<dyn bmw_util::SlabAllocator>>> = None;
                if slabs.is_some() {
                    slabs = None;
                }
                $(
                    match $config {
                        bmw_util::ConfigOption::Slabs(s) => {
                            slabs = Some(s);
                        }
                        _ => {
                        }
                    }
                )*

                let mut config_list = bmw_util::Builder::build_list(
                        bmw_util::ListConfig::default(),
                        slabs.clone()
                )?;

                // delete_head is called to remove compiler warning about not needing to be mutable.
                // It does need to be mutable.
                config_list.delete_head()?;


                $(
                    match $config {
                        bmw_util::ConfigOption::Slabs(_) => {
                        }
                        _ => {
                            config_list.push($config)?;
                        }
                    }
                )*

                let (error, config) = bmw_util::hashset_config!(config_list);
                match error {
                        Some(error) => Err(bmw_err::err!(bmw_err::ErrKind::Configuration, error)),
                        None => bmw_util::Builder::build_hashset(config.unwrap(), slabs),
                }
        }};
}

#[macro_export]
macro_rules! hashset_box {
        ( $( $config:expr ),* ) => {{
                use bmw_util::List;

                let mut slabs: Option<std::rc::Rc<std::cell::RefCell<dyn bmw_util::SlabAllocator>>> = None;
                if slabs.is_some() {
                    slabs = None;
                }
                $(
                    match $config {
                        bmw_util::ConfigOption::Slabs(s) => {
                            slabs = Some(s);
                        }
                        _ => {
                        }
                    }
                )*

                let mut config_list = bmw_util::Builder::build_list(
                        bmw_util::ListConfig::default(),
                        slabs.clone()
                )?;

                // delete_head is called to remove compiler warning about not needing to be mutable.
                // It does need to be mutable.
                config_list.delete_head()?;


                $(
                    match $config {
                        bmw_util::ConfigOption::Slabs(_) => {
                        }
                        _ => {
                            config_list.push($config)?;
                        }
                    }
                )*

                let (error, config) = bmw_util::hashset_config!(config_list);
                match error {
                        Some(error) => Err(bmw_err::err!(bmw_err::ErrKind::Configuration, error)),
                        None => bmw_util::Builder::build_hashset_box(config.unwrap(), slabs),
                }
        }};
}

#[macro_export]
macro_rules! hashset_sync {
        ( $( $config:expr ),* ) => {{
                use bmw_util::List;

                $(
                    match $config {
                        bmw_util::ConfigOption::Slabs(_) => {
                            return Err(bmw_err::err!(
                                    bmw_err::ErrKind::Configuration,
                                    "Slabs cannot be specified with hashset_sync"
                            ));
                        }
                        _ => {
                        }
                    }
                )*

                let mut config_list = bmw_util::Builder::build_list(
                        bmw_util::ListConfig::default(),
                        None
                )?;

                // delete_head is called to remove compiler warning about not needing to be mutable.
                // It does need to be mutable.
                config_list.delete_head()?;

                $(
                    match $config {
                        bmw_util::ConfigOption::Slabs(_) => {
                        }
                        _ => {
                            config_list.push($config)?;
                        }
                    }
                )*

                let (error, config, slab_config) = bmw_util::hashset_sync_config!(config_list);
                match error {
                        Some(error) => Err(bmw_err::err!(bmw_err::ErrKind::Configuration, error)),
                        None => bmw_util::Builder::build_hashset_sync(config.unwrap(), slab_config.unwrap()),
                }
        }};
}

#[macro_export]
macro_rules! hashset_sync_box {
        ( $( $config:expr ),* ) => {{
                use bmw_util::List;

                $(
                    match $config {
                        bmw_util::ConfigOption::Slabs(_) => {
                            return Err(bmw_err::err!(
                                    bmw_err::ErrKind::Configuration,
                                    "Slabs cannot be specified with hashset_sync"
                            ));
                        }
                        _ => {
                        }
                    }
                )*

                let mut config_list = bmw_util::Builder::build_list(
                        bmw_util::ListConfig::default(),
                        None
                )?;

                // delete_head is called to remove compiler warning about not needing to be mutable.
                // It does need to be mutable.
                config_list.delete_head()?;

                $(
                    match $config {
                        bmw_util::ConfigOption::Slabs(_) => {
                        }
                        _ => {
                            config_list.push($config)?;
                        }
                    }
                )*

                let (error, config, slab_config) = bmw_util::hashset_sync_config!(config_list);
                match error {
                        Some(error) => Err(bmw_err::err!(bmw_err::ErrKind::Configuration, error)),
                        None => bmw_util::Builder::build_hashset_sync_box(config.unwrap(), slab_config.unwrap()),
                }
        }};
}

#[macro_export]
macro_rules! list {
    ( $( $x:expr ),* ) => {
        {
            use bmw_util::List;
            let mut temp_list = bmw_util::Builder::build_list(bmw_util::ListConfig::default(), None)?;
            $(
                temp_list.push($x)?;
            )*
            temp_list
        }
    };
}

#[macro_export]
macro_rules! list_box {
    ( $( $x:expr ),* ) => {
        {
            use bmw_util::List;
            let mut temp_list = bmw_util::Builder::build_list_box(bmw_util::ListConfig::default(), None)?;
            $(
                temp_list.push($x)?;
            )*
            temp_list
        }
    };
}

#[macro_export]
macro_rules! list_sync {
    ( $( $x:expr ),* ) => {
        {
            use bmw_util::List;
            let mut temp_list = bmw_util::Builder::build_list_sync(bmw_util::ListConfig::default(), None)?;
            $(
                temp_list.push($x)?;
            )*
            temp_list
        }
    };
}

#[macro_export]
macro_rules! list_sync_box {
    ( $( $x:expr ),* ) => {
        {
            use bmw_util::List;
            let mut temp_list = bmw_util::Builder::build_list_sync_box(bmw_util::ListConfig::default(), None)?;
            $(
                temp_list.push($x)?;
            )*
            temp_list
        }
    };
}

#[macro_export]
macro_rules! array {
	( $size:expr ) => {{
		bmw_util::Builder::build_array($size)
	}};
}

#[macro_export]
macro_rules! array_list {
	( $size:expr ) => {{
		bmw_util::Builder::build_array_list($size)
	}};
}

#[macro_export]
macro_rules! array_list_box {
	( $size:expr ) => {{
		bmw_util::Builder::build_array_list_box($size)
	}};
}

#[macro_export]
macro_rules! queue {
	( $size:expr ) => {{
		bmw_util::Builder::build_queue($size)
	}};
}

#[macro_export]
macro_rules! queue_box {
	( $size:expr ) => {{
		bmw_util::Builder::build_queue_box($size)
	}};
}

#[macro_export]
macro_rules! stack {
	( $size:expr ) => {{
		bmw_util::Builder::build_stack($size)
	}};
}

#[macro_export]
macro_rules! stack_box {
	( $size:expr ) => {{
		bmw_util::Builder::build_stack_box($size)
	}};
}

#[macro_export]
macro_rules! list_append {
	($list1:expr, $list2:expr) => {{
		for x in $list2.iter() {
			$list1.push(x)?;
		}
	}};
}

#[macro_export]
macro_rules! list_eq {
	($list1:expr, $list2:expr) => {{
		let list1 = &$list1;
		let list2 = &$list2;
		let list1_size = list1.size();
		if list1_size != list2.size() {
			false
		} else {
			let mut ret = true;
			{
				let mut itt1 = list1.iter();
				let mut itt2 = list2.iter();
				for _ in 0..list1_size {
					if itt1.next() != itt2.next() {
						ret = false;
					}
				}
			}
			ret
		}
	}};
}

/// Macro used to configure/build a thread pool. See [`crate::ThreadPool`] for working examples.
#[macro_export]
macro_rules! thread_pool {
	() => {{
                let config = bmw_util::ThreadPoolConfig::default();
                match bmw_util::Builder::build_thread_pool(config) {
                        Ok(mut ret) => {
                                ret.start()?;
                                Ok(ret)
                        }
                        Err(e) => {
                            Err(
                                    bmw_err::err!(
                                            bmw_err::ErrKind::Misc,
                                            format!("threadpoolbuilder buld error: {}", e)
                                    )
                            )
                        }
                }
        }};
	( $( $config:expr ),* ) => {{
                let mut config = bmw_util::ThreadPoolConfig::default();
		let mut error: Option<String> = None;
		let mut min_size_specified = false;
                let mut max_size_specified = false;
                let mut sync_channel_size_specified = false;

                $(
                match $config {
                    bmw_util::ConfigOption::MinSize(min_size) => {
                        config.min_size = min_size;
                         if min_size_specified {
                            error = Some("MinSize was specified more than once!".to_string());
                        }

                        min_size_specified = true;
                        if min_size_specified {}
                    },
                    bmw_util::ConfigOption::MaxSize(max_size) => {
                        config.max_size = max_size;
                        if max_size_specified {
                            error = Some("MaxSize was specified more than once!".to_string());
                        }

                        max_size_specified = true;
                        if max_size_specified {}
                    },
                    bmw_util::ConfigOption::SyncChannelSize(sync_channel_size) => {
                        config.sync_channel_size = sync_channel_size;
                        if sync_channel_size_specified {
                             error = Some("SyncChannelSize was specified more than once!".to_string());
                        }

                        sync_channel_size_specified = true;
                        if sync_channel_size_specified {}
                    }
                    _ => {
                        error = Some(
                            format!(
                                "Invalid configuration {:?} is not allowed for thread_pool!",
                                $config
                            )
                        );
                    }
                }
                )*

                match error {
                    Some(error) => Err(bmw_err::err!(bmw_err::ErrKind::Configuration, error)),
                    None => {
                            let mut ret = bmw_util::Builder::build_thread_pool(config)?;
                            ret.start()?;
                            Ok(ret)
                    },
                }
        }};
}

/// Macro used to execute tasks in a thread pool. See [`crate::ThreadPool`] for working examples.
#[macro_export]
macro_rules! execute {
	($thread_pool:expr, $program:expr) => {{
		$thread_pool.execute(async move { $program })
	}};
}

/// Macro used to block until a thread pool has completed the task. See [`crate::ThreadPool`] for working examples.
#[macro_export]
macro_rules! block_on {
	($res:expr) => {{
		match $res.recv() {
			Ok(res) => res,
			Err(e) => bmw_util::PoolResult::Err(bmw_err::err!(
				bmw_err::ErrKind::ThreadPanic,
				format!("thread pool panic: {}", e)
			)),
		}
	}};
}

#[cfg(test)]
mod test {
	use crate as bmw_util;
	use crate::{thread_pool, Hashset, Hashtable, List, PoolResult, SortableList, ThreadPool};
	use bmw_err::{err, ErrKind, Error};
	use bmw_log::*;
	use bmw_util::ConfigOption::*;
	use std::cell::RefMut;
	use std::sync::mpsc::Receiver;
	use std::thread::sleep;
	use std::time::Duration;

	info!();

	struct TestHashsetHolder {
		h1: Option<Box<dyn Hashset<u32>>>,
		h2: Option<Box<dyn LockBox<Box<dyn Hashset<u32> + Send + Sync>>>>,
	}

	#[test]
	fn test_hashset_macros() -> Result<(), Error> {
		let slabs = slab_allocator!(SlabSize(128), SlabCount(1))?;
		{
			let mut hashset = hashset!(Slabs(slabs.clone()))?;
			hashset.insert(&1)?;
			assert!(hashset.contains(&1)?);
			assert!(!hashset.contains(&2)?);
			assert!(hashset.insert(&2).is_err());
		}

		{
			let mut hashset = hashset_box!(Slabs(slabs.clone()))?;
			hashset.insert(&1)?;
			assert!(hashset.contains(&1)?);
			assert!(!hashset.contains(&2)?);
			assert!(hashset.insert(&2).is_err());

			let mut thh = TestHashsetHolder {
				h2: None,
				h1: Some(hashset),
			};

			{
				let hashset = thh.h1.as_mut().unwrap();
				assert_eq!(hashset.size(), 1);
			}
		}

		{
			let mut hashset = hashset_sync!(SlabSize(128), SlabCount(1))?;
			hashset.insert(&1)?;
			assert!(hashset.contains(&1)?);
			assert!(!hashset.contains(&2)?);
			assert!(hashset.insert(&2).is_err());
		}

		{
			let hashset = hashset_sync_box!(SlabSize(128), SlabCount(1))?;
			let mut hashset = lock_box!(hashset)?;

			{
				let mut hashset = hashset.wlock()?;
				(**hashset.guard()).insert(&1)?;
				assert!((**hashset.guard()).contains(&1)?);
				assert!(!(**hashset.guard()).contains(&2)?);
				assert!((**hashset.guard()).insert(&2).is_err());
			}

			let mut thh = TestHashsetHolder {
				h1: None,
				h2: Some(hashset),
			};

			{
				let mut hashset = thh.h2.as_mut().unwrap().wlock()?;
				assert_eq!((**hashset.guard()).size(), 1);
			}
		}

		Ok(())
	}

	#[test]
	fn test_slabs_in_hashtable_macro() -> Result<(), Error> {
		let slabs = slab_allocator!(SlabSize(128), SlabCount(1))?;
		let mut hashtable = hashtable!(Slabs(slabs.clone()))?;
		hashtable.insert(&1, &2)?;

		assert_eq!(hashtable.get(&1).unwrap(), Some(2));

		assert!(hashtable.insert(&2, &3).is_err());

		Ok(())
	}

	#[test]
	fn test_hashtable_box_macro() -> Result<(), Error> {
		let slabs = slab_allocator!(SlabSize(128), SlabCount(1))?;
		let mut hashtable = hashtable_box!(Slabs(slabs.clone()))?;
		hashtable.insert(&1, &2)?;

		assert_eq!(hashtable.get(&1).unwrap(), Some(2));

		assert!(hashtable.insert(&2, &3).is_err());

		Ok(())
	}

	struct TestHashtableSyncBox {
		h: Box<dyn LockBox<Box<dyn Hashtable<u32, u32> + Send + Sync>>>,
	}

	#[test]
	fn test_hashtable_sync_box_macro() -> Result<(), Error> {
		let hashtable = hashtable_sync_box!(SlabSize(128), SlabCount(1), MaxEntries(10))?;
		let mut hashtable = lock_box!(hashtable)?;

		{
			let mut hashtable = hashtable.wlock()?;
			(**hashtable.guard()).insert(&1, &2)?;
			assert_eq!((**hashtable.guard()).get(&1).unwrap(), Some(2));
			assert!((**hashtable.guard()).insert(&2, &3).is_err());
		}

		let thsb = TestHashtableSyncBox { h: hashtable };

		{
			let h = thsb.h.rlock()?;
			assert!((**h.guard()).get(&1)?.is_some());
			assert!((**h.guard()).get(&2)?.is_none());
		}

		Ok(())
	}

	#[test]
	fn test_hashtable_sync_macro() -> Result<(), Error> {
		let hashtable = hashtable_sync_box!(SlabSize(128), SlabCount(1), MaxEntries(10))?;
		let mut hashtable = lock!(hashtable)?;

		{
			let mut hashtable = hashtable.wlock()?;
			(**hashtable.guard()).insert(&1, &2)?;
			assert_eq!((**hashtable.guard()).get(&1).unwrap(), Some(2));
			assert!((**hashtable.guard()).insert(&2, &3).is_err());
		}

		Ok(())
	}

	#[test]
	fn test_slab_allocator_macro() -> Result<(), bmw_err::Error> {
		let slabs = slab_allocator!()?;
		let slabs2 = slab_allocator!(SlabSize(128), SlabCount(1))?;

		let mut slabs: RefMut<_> = slabs.borrow_mut();
		let mut slabs2: RefMut<_> = slabs2.borrow_mut();

		let slab = slabs.allocate()?;
		assert_eq!(
			slab.get().len(),
			bmw_util::SlabAllocatorConfig::default().slab_size
		);
		let slab = slabs2.allocate()?;
		assert_eq!(slab.get().len(), 128);
		assert!(slabs2.allocate().is_err());
		assert!(slabs.allocate().is_ok());

		assert!(slab_allocator!(SlabSize(128), SlabSize(64)).is_err());
		assert!(slab_allocator!(SlabCount(128), SlabCount(64)).is_err());
		assert!(slab_allocator!(MaxEntries(128)).is_err());
		assert!(slab_allocator!(MaxLoadFactor(128.0)).is_err());
		Ok(())
	}

	#[test]
	fn test_hashtable_macro() -> Result<(), bmw_err::Error> {
		let mut hashtable = hashtable!()?;
		hashtable.insert(&1, &2)?;
		assert_eq!(hashtable.get(&1).unwrap().unwrap(), 2);
		let mut hashtable = hashtable!(MaxEntries(100), MaxLoadFactor(0.9))?;
		hashtable.insert(&"test".to_string(), &1)?;
		assert_eq!(hashtable.size(), 1);
		hashtable.insert(&"something".to_string(), &2)?;
		info!("hashtable={:?}", hashtable)?;
		Ok(())
	}

	#[test]
	fn test_hashset_macro() -> Result<(), bmw_err::Error> {
		let mut hashset = hashset!()?;
		hashset.insert(&1)?;
		assert_eq!(hashset.contains(&1).unwrap(), true);
		assert_eq!(hashset.contains(&2).unwrap(), false);
		let mut hashset = hashset!(MaxEntries(100), MaxLoadFactor(0.9))?;
		hashset.insert(&"test".to_string())?;
		assert_eq!(hashset.size(), 1);
		assert!(hashset.contains(&"test".to_string())?);
		info!("hashset={:?}", hashset)?;
		hashset.insert(&"another item".to_string())?;
		hashset.insert(&"third item".to_string())?;
		info!("hashset={:?}", hashset)?;
		Ok(())
	}

	#[test]
	fn test_list_macro() -> Result<(), bmw_err::Error> {
		let mut list1 = list!['1', '2', '3'];
		list_append!(list1, list!['a', 'b', 'c']);
		let list2 = list!['1', '2', '3', 'a', 'b', 'c'];
		assert!(list_eq!(list1, list2));
		let list2 = list!['a', 'b', 'c', '1', '2'];
		assert!(!list_eq!(list1, list2));

		let list3 = list![1, 2, 3, 4, 5];
		info!("list={:?}", list3)?;
		Ok(())
	}

	#[test]
	fn test_thread_pool_macro() -> Result<(), bmw_err::Error> {
		let tp = thread_pool!()?;
		let resp = execute!(tp, {
			info!("in thread pool")?;
			Ok(123)
		})?;
		assert_eq!(block_on!(resp), PoolResult::Ok(123));

		let tp = thread_pool!(MinSize(3))?;
		let resp: Receiver<PoolResult<u32, Error>> = execute!(tp, {
			info!("thread pool2")?;
			Err(err!(ErrKind::Test, "test err"))
		})?;
		assert_eq!(
			block_on!(resp),
			PoolResult::Err(err!(ErrKind::Test, "test err"))
		);

		let tp = thread_pool!(MinSize(3))?;
		let resp: Receiver<PoolResult<u32, Error>> = execute!(tp, {
			info!("thread pool panic")?;
			let x: Option<u32> = None;
			Ok(x.unwrap())
		})?;
		assert_eq!(
			block_on!(resp),
			PoolResult::Err(err!(
				ErrKind::ThreadPanic,
				"thread pool panic: receiving on a closed channel"
			))
		);
		Ok(())
	}

	#[test]
	fn test_thread_pool_options() -> Result<(), Error> {
		let tp = thread_pool!(MinSize(4), MaxSize(5), SyncChannelSize(10))?;

		assert_eq!(tp.size()?, 4);
		sleep(Duration::from_millis(2_000));
		let resp = execute!(tp, {
			info!("thread pool")?;
			Ok(0)
		})?;
		assert_eq!(block_on!(resp), PoolResult::Ok(0));
		assert_eq!(tp.size()?, 4);

		for _ in 0..10 {
			execute!(tp, {
				info!("thread pool")?;
				sleep(Duration::from_millis(5_000));
				Ok(0)
			})?;
		}
		sleep(Duration::from_millis(2_000));
		assert_eq!(tp.size()?, 5);
		Ok(())
	}

	#[test]
	fn test_list_eq() -> Result<(), Error> {
		let list1 = list![1, 2, 3];
		let eq = list_eq!(list1, list![1, 2, 3]);
		let mut list2 = list![4, 5, 6];
		list_append!(list2, list![5, 5, 5]);
		assert!(eq);
		assert!(list_eq!(list2, list![4, 5, 6, 5, 5, 5]));
		list2.sort_unstable()?;
		assert!(list_eq!(list2, list![4, 5, 5, 5, 5, 6]));
		assert!(!list_eq!(list2, list![1, 2, 3]));
		Ok(())
	}

	#[test]
	fn test_array_macro() -> Result<(), Error> {
		let mut array = array!(10)?;
		array[1] = 2;
		assert_eq!(array[1], 2);
		Ok(())
	}
}
