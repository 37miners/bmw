initSidebarItems({"enum":[["ErrKind","The names of ErrorKinds in this crate. This enum is used to map to error names using the [`crate::err`] and [`crate::map_err`] macros."],["ErrorKind","Kinds of errors that can occur."]],"macro":[["err","Build the specified [`crate::ErrorKind`] and convert it into an [`crate::Error`]. The desired [`crate::ErrorKind`] is specified using the [`crate::ErrKind`] name enum."],["map_err","Map the specified error into the [`crate::ErrKind`] enum name from this crate. Optionally specify an additional message to be included in the error."],["try_into","Macro to map the try_from error into an appropriate error."]],"struct":[["Error","Base Error struct which is used throughout bmw."]]});