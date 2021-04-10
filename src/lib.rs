//! # Error context
//!
//! Generally libraries should not add context to the errors as it may be considered sensitive for
//! some uses.
//! In this library context (like file paths) will be provided by default.
//!
//! # Static error types
//!
//! When you need proper error handling (e.g. on the internal modules or when you need to act on
//! the errors specifically) use standard way of doing this.
//!
//! Use enums with `Debug`, `Display` and `Error` trait implementations.
//! Add additional `From` implementations to make `?` operator to work.
//!
//! If you need to add context to an error you can use [error_context](https://docs.rs/error-context) crate that is included in the prelude.
//!
//! ## Example custom static error type implementation
//!
//! ```rust
//! use cotton::prelude::*;
//!
//! #[derive(Debug)]
//! enum FileResourceError {
//!         FileDigestError(PathBuf, FileDigestError),
//!         NotAFileError(PathBuf),
//! }
//!
//! impl Display for FileResourceError {
//!     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//!         match self {
//!             // Do not include chained error message in the message; let the client handle this (e.g. with Problem type)
//!             FileResourceError::FileDigestError(path, _) => write!(f, "digest of a file {:?} could not be calculated", path),
//!             FileResourceError::NotAFileError(path) => write!(f, "path {:?} is not a file", path),
//!         }
//!     }
//! }
//!
//! impl Error for FileResourceError {
//!     fn source(&self) -> Option<&(dyn Error + 'static)> {
//!         match self {
//!             // Chain the internal error
//!             FileResourceError::FileDigestError(_, err) => Some(err),
//!             FileResourceError::NotAFileError(_) => None,
//!         }
//!     }
//! }
//!
//! // This allows for calls like `foo().wrap_error_while_with(|| self.path.clone())?` to add extra `PathBuf` context to the error
//! impl From<ErrorContext<FileDigestError, PathBuf>> for FileResourceError {
//!     fn from(err: ErrorContext<FileDigestError, PathBuf>) -> FileResourceError {
//!         FileResourceError::FileDigestError(err.context, err.error)
//!     }
//! }
//! ```
//!

//TODO: don't use Problme for error type of the functions in this crate as it makes it more diffuclt to work with Error trait based errors in the client.
//TODO: use https://crates.io/crates/camino for Path? If so also add support in file-mode crate.

mod app_dir;
mod cmd;
mod digest;
mod time;

// needed for derive to work
pub use structopt;

// Export crates to give access to unexported types
pub use filetime;
pub use boolinator;
pub use chrono;
pub use itertools;
pub use linked_hash_map;
pub use linked_hash_set;
pub use log;
pub use problem;
pub use error_context;
pub use shaman;
pub use tap;
pub mod loggerv;
pub use duct;
pub use file_mode;
#[cfg(target_family = "unix")]
pub use file_owner;

pub mod prelude {
    // Often used I/O
    pub use std::fs::{
        canonicalize, copy, create_dir, create_dir_all, hard_link, metadata, read, read_dir,
        read_link, read_to_string, remove_dir, remove_dir_all, remove_file, rename,
        set_permissions, symlink_metadata, write, DirBuilder, DirEntry, File, Metadata,
        OpenOptions, Permissions, ReadDir,
    };
    pub use std::io::{self, stdin, stdout, BufRead, BufReader, BufWriter, Read, Write, Cursor};

    pub use std::path::{Path, PathBuf};

    // filesystem
    pub use file_mode::{ModeParseError, Mode, User, FileType, ProtectionBit, Protection, SpecialBit, Special};
    #[cfg(target_family = "unix")]
    pub use file_mode::{ModeError, ModePath, ModeFile, SetMode};
    #[cfg(target_family = "unix")]
    pub use file_owner::{PathExt, group, owner, owner_group, set_group, set_owner, set_owner_group};

    // Extra traits and stuff
    pub use std::hash::Hash;
    pub use std::marker::PhantomData;

    // Timestamps for files
    pub use filetime::{set_file_atime, set_file_handle_times, set_file_mtime, set_file_times,
        set_symlink_file_times, FileTime};

    // Often used data structures
    pub use std::borrow::Cow;
    pub use std::collections::HashMap;
    pub use std::collections::HashSet;

    // Ordered HashMap/Set
    pub use linked_hash_map::LinkedHashMap;
    pub use linked_hash_set::LinkedHashSet;

    // New std traits
    pub use std::convert::Infallible;
    pub use std::convert::TryFrom;
    pub use std::convert::TryInto; // As we wait for "!"

    // Logging and messaging
    pub use log::{debug, error, info, log_enabled, trace, warn};
    pub use std::fmt::Write as FmtWrite; // allow write! to &mut String
    pub use std::fmt::{self, Display, Debug};

    // Arguments
    pub use structopt::StructOpt;

    // Error handling
    pub use std::error::Error;
    pub use assert_matches::assert_matches;
    pub use ::problem::prelude::{problem, in_context_of, in_context_of_with, FailedTo, FailedToIter, Fatal, FatalProblem,
        MapProblem, MapProblemOr, OkOrProblem, Problem, ProblemWhile, OkOrLog, OkOrLogIter};
    pub use ::problem::result::FinalResult;
    pub use ::problem::result::Result as PResult;
    pub use ::error_context::{
        in_context_of as in_error_context_of, in_context_of_with as in_error_context_of_with, wrap_in_context_of,
        wrap_in_context_of_with, ErrorContext, ErrorNoContext, MapErrorNoContext, ResultErrorWhile,
        ResultErrorWhileWrap, ToErrorNoContext, WithContext, WrapContext};

    // Running commands
    pub use super::cmd::*;

    // Content hashing
    pub use super::digest::*;

    // App directory
    pub use super::app_dir::*;

    // Time and duration
    pub use super::time::*;

    // Iterators
    pub use itertools::*;
    pub use std::iter::FromIterator;
    pub use std::iter::{empty, from_fn, once, once_with, repeat, repeat_with, successors};

    // Handy extensions
    pub use boolinator::Boolinator;
    pub use tap::prelude::{Conv, Tap, TapFallible, TapOptional, TryConv};
    //TODO: pub use tap::prelude::{Pipe}; - colides with duct::Expression.pipe!

    #[derive(Debug, StructOpt)]
    pub struct LoggingOpt {
        /// Verbose mode (-v for INFO, -vv for DEBUG, -vvv for TRACE, -vvvv TRACE all modules)
        #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
        pub verbose: u8,

        /// Only log errors
        #[structopt(long = "errors-only")]
        pub errors_only: bool,

        /// Force colorizing the logger output
        #[structopt(long = "force-colors")]
        pub force_colors: bool,
    }

    #[derive(Debug, StructOpt)]
    pub struct DryRunOpt {
        /// Just print what would have been done
        #[structopt(long = "dry-run", short = "-d")]
        pub enabled: bool,
    }

    impl DryRunOpt {
        pub fn run(&self, msg: impl Display, run: impl FnOnce() -> ()) -> () {
            if self.enabled {
                info!("[dry run]: {}", msg);
            } else {
                info!("{}", msg);
                run()
            }
        }
    }

    #[derive(Debug)]
    pub enum FileIoError {
        IoError(PathBuf, io::Error),
    }

    impl Display for FileIoError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                FileIoError::IoError(path, _) => write!(f, "I/O error while reading file {:?}", path),
            }
        }
    }

    impl Error for FileIoError {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            match self {
                FileIoError::IoError(_, err) => Some(err),
            }
        }
    }

    impl From<ErrorContext<io::Error, PathBuf>> for FileIoError {
        fn from(err: ErrorContext<io::Error, PathBuf>) -> FileIoError {
            FileIoError::IoError(err.context, err.error)
        }
    }

    pub fn read_stdin() -> String {
        let mut buffer = String::new();
        stdin()
            .read_to_string(&mut buffer)
            .or_failed_to("read UTF-8 string from stdin");
        buffer
    }

    pub fn read_stdin_bytes() -> Vec<u8> {
        let mut buffer = Vec::new();
        stdin()
            .read_to_end(&mut buffer)
            .or_failed_to("read bytes from stdin");
        buffer
    }

    pub fn read_stdin_lines() -> impl Iterator<Item = String> {
        BufReader::new(stdin())
            .lines()
            .or_failed_to("read UTF-8 lines from stdin")
    }

    /// Read content of all files as string.
    pub fn read_all(paths: impl IntoIterator<Item = impl AsRef<Path>>) -> Result<String, FileIoError> {
        let mut string = String::new();

        for path in paths {
            let path = path.as_ref();
            let mut file = File::open(path).wrap_error_while_with(|| path.into())?;
            file.read_to_string(&mut string).wrap_error_while_with(|| path.into())?;
        }

        Ok(string)
    }

    /// Read content of all files as bytes.
    pub fn read_all_bytes(paths: impl IntoIterator<Item = impl AsRef<Path>>) -> Result<Vec<u8>, FileIoError> {
        let mut bytes = Vec::new();

        for path in paths {
            let path = path.as_ref();
            let mut file = File::open(path).wrap_error_while_with(|| path.into())?;
            file.read_to_end(&mut bytes).wrap_error_while_with(|| path.into())?;
        }

        Ok(bytes)
    }

    pub fn init_logger(
        args: &LoggingOpt,
        module_paths: impl IntoIterator<Item = impl Into<String>>,
    ) {
        use crate::loggerv::{Logger, Output};
        use log::Level;

        let (base_level, verbose) = if args.errors_only {
            (Level::Error, 0)
        } else {
            (Level::Warn, args.verbose)
        };

        let mut logger = Logger::new()
            .base_level(base_level)
            .verbosity(verbose as u64)
            .output(&Level::Info, Output::Stderr)
            .output(&Level::Debug, Output::Stderr)
            .output(&Level::Trace, Output::Stderr)
            .module_path(false)
            .timestamp_format_default();

        if verbose <= 3 {
            logger = logger
                .add_module_path_filter("cotton")
                .add_module_path_filter("problem");

            logger = module_paths
                .into_iter()
                .fold(logger, |logger, module_path| {
                    logger.add_module_path_filter(module_path)
                });
        }

        if args.force_colors {
            logger = logger.force_colors()
        }

        logger.level(true).init().or_failed_to("init logger");

        ::problem::format_panic_to_error_log();
    }
}


#[cfg(test)]
mod tests {
    use super::prelude::*;

    #[test]
    #[should_panic(expected = "Failed to baz due to: while bar got error caused by: foo")]
    fn test_problem() {
        in_context_of("bar", || {
            problem!("foo")?;
            Ok(())
        }).or_failed_to("baz");
    }
}
