initSidebarItems({"enum":[["LogConfigOption","This enum is used to get/set log settings after [`Log::init`] is called. The only setting that cannot be set after initialization is the [`LogConfigOption::FilePath`] setting. It is read only. Trying to write to it will result in an error. The function used to get these values is [`Log::get_config_option`] and the function used to set these values is [`Log::set_config_option`]."],["LogConfigOptionName","This enum contains the names of the configuration options. It is used in the [`Log::get_config_option`] function. See [`Log::get_config_option`] for further details."],["LogLevel","Standard 6 log levels."]],"macro":[["debug",""],["debug_all",""],["debug_plain",""],["do_log",""],["error",""],["error_all",""],["error_plain",""],["fatal",""],["fatal_all",""],["fatal_plain",""],["get_log_option",""],["info",""],["info_all",""],["info_plain",""],["lockr","A macro that is used to lock a rwlock in read mode and return the appropriate error if the lock is poisoned."],["lockw","A macro that is used to lock a rwlock in write mode and return the appropriate error if the lock is poisoned."],["log",""],["log_all",""],["log_init",""],["log_plain",""],["log_rotate",""],["need_rotate",""],["set_log_option",""],["trace",""],["trace_all",""],["trace_plain",""],["warn",""],["warn_all",""],["warn_plain",""]],"struct":[["LogBuilder","The publicly accessible builder struct. This is the only way a log can be created outside of this crate. See [`LogBuilder::build`]."],["LogConfig","The log configuration struct. Logs can only be built through the [`crate::LogBuilder::build`] function. This is the only parameter to that function. An example configuration with all parameters explicitly specified might look like this:"]],"trait":[["Log",""]]});