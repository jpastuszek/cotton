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
//TODO: put some features behind feature flags (all enabled by default): hashing, shell/cmd, scopeguard, signals/uninterruptible, time/duration, app_dir
//TODO: consider duct replacement?
//TODO: add progress bar crate under "term" feature

#[cfg(feature = "app")]
mod app_dir;
#[cfg(feature = "hashing")]
mod hashing;
#[cfg(feature = "time")]
mod time;

// All used crates available for direct usage

// Extensions
pub use itertools;
pub use linked_hash_map;
pub use linked_hash_set;
pub use boolinator;
pub use tap;

// File
#[cfg(feature = "filetime")]
pub use filetime;
#[cfg(all(target_family = "unix", feature = "file-owner"))]
pub use file_owner;
#[cfg(feature = "file-mode")]
pub use file_mode;

// Error handling
#[cfg(feature = "problem")]
pub use problem;
#[cfg(feature = "error-context")]
pub use error_context;
#[cfg(feature = "scopeguard")]
pub use scopeguard;
#[cfg(feature = "assert_matches")]
pub use assert_matches;

// Time/Date
#[cfg(feature = "chrono")]
pub use chrono;

// Terminal
#[cfg(feature = "ansi_term")]
pub use ansi_term;
#[cfg(feature = "atty")]
pub use atty;

// Argparse
#[cfg(feature = "clap")]
pub use clap;

// Logging
#[cfg(feature = "log")]
pub use log;
#[cfg(feature = "stderr")]
pub use stderrlog;

// Hashing
#[cfg(feature = "sha2")]
pub use sha2;

// Shellout/processes
//TODO: use cradle
//TODO: use mkargs
pub use shellwords;

// Strings
#[cfg(feature = "hex")]
pub use hex;
pub use maybe_string;

// UNIX signals
#[cfg(all(target_family = "unix", feature = "signal-hook"))]
pub use signal_hook;
#[cfg(all(target_family = "unix", feature = "uninterruptible"))]
pub use uninterruptible;

// Application environment
#[cfg(feature = "directories")]
pub use directories;

pub mod prelude {
    // Often used I/O
    pub use std::fs::{
        canonicalize, copy, create_dir, create_dir_all, hard_link, metadata, read, read_dir,
        read_link, read_to_string, remove_dir, remove_dir_all, remove_file, rename,
        set_permissions, symlink_metadata, write, DirBuilder, DirEntry, File, Metadata,
        OpenOptions, Permissions, ReadDir
    };
    pub use std::io::{
        self, stdin, stdout, BufRead, BufReader, BufWriter, Read, Write, Cursor,
        Seek, SeekFrom
    };

    pub use std::path::{Path, PathBuf};

    // filesystem
    #[cfg(feature = "file-mode")]
    pub use file_mode::{ModeParseError, Mode as FileMode, User, FileType, ProtectionBit, Protection, SpecialBit, Special, set_umask};
    #[cfg(all(target_family = "unix", feature = "file-mode"))]
    pub use file_mode::{ModeError, ModePath, ModeFile, SetMode};
    #[cfg(all(target_family = "unix", feature = "file-owner"))]
    pub use file_owner::{FileOwnerError, PathExt, group, owner, owner_group, set_group, set_owner, set_owner_group, Group as FileGroup, Owner as FileOwner};

    // Extra traits and stuff
    pub use std::hash::Hash;
    pub use std::marker::PhantomData;

    // Timestamps for files
    #[cfg(feature = "filetime")]
    pub use filetime::{set_file_atime, set_file_handle_times, set_file_mtime, set_file_times,
        set_symlink_file_times, FileTime};

    // Often used data structures
    pub use std::borrow::Cow;
    pub use std::collections::HashMap;
    pub use std::collections::HashSet;

    // String helpers
    pub use maybe_string::{MaybeString, MaybeStr};

    // Ordered HashMap/Set
    pub use linked_hash_map::LinkedHashMap;
    pub use linked_hash_set::LinkedHashSet;

    // New std traits
    pub use std::convert::Infallible;
    pub use std::convert::TryFrom;
    pub use std::convert::TryInto; // As we wait for "!"

    // Formatting
    pub use std::fmt::Write as FmtWrite; // allow write! to &mut String
    pub use std::fmt::{self, Display, Debug};

    // Arguments
    #[cfg(feature = "args")]
    pub use clap::{self /* needed for derive to work */, Parser, Args, ValueEnum, Subcommand};

    // Error handling
    pub use std::error::Error;
    #[cfg(feature = "errors")]
    pub use assert_matches::assert_matches;
    #[cfg(feature = "errors")]
    pub use ::problem::prelude::{problem, in_context_of, in_context_of_with, FailedTo, FailedToIter, Fatal, FatalProblem,
        MapProblem, MapProblemOr, OkOrProblem, Problem, ProblemWhile, OkOrLog, OkOrLogIter};
    #[cfg(feature = "errors")]
    pub use ::problem::result::{FinalResult, Result as PResult};
    #[cfg(feature = "errors")]
    pub use ::error_context::{
        in_context_of as in_error_context_of, in_context_of_with as in_error_context_of_with, wrap_in_context_of,
        wrap_in_context_of_with, ErrorContext, ErrorNoContext, MapErrorNoContext, ResultErrorWhile,
        ResultErrorWhileWrap, ToErrorNoContext, WithContext, WrapContext};
    #[cfg(feature = "errors")]
    pub use scopeguard::{defer, defer_on_success, defer_on_unwind, guard, guard_on_success, guard_on_unwind};

    // Running commands
    pub use ::shellwords::{escape as shell_escape, join as shell_join, split as shell_split};

    // Content hashing and crypto
    #[cfg(feature = "hashing")]
    pub use super::hashing::*;

    #[cfg(feature = "hex")]
    pub use hex::{encode as hex_encode, decode as hex_decode, FromHexError};
    #[cfg(feature = "hashing")]
    pub use sha2::digest::{self, generic_array::{self, GenericArray}};

    // Application environment
    #[cfg(feature = "app")]
    pub use super::app_dir::*;

    // Time and duration
    #[cfg(feature = "time")]
    pub use super::time::*;

    // Iterators
    pub use itertools::*;
    pub use std::iter::FromIterator;
    pub use std::iter::{empty, from_fn, once, once_with, repeat, repeat_with, successors};

    // Signals
    #[cfg(all(target_family = "unix", feature = "signals"))]
    pub use uninterruptible::Uninterruptible;

    // Handy extensions
    pub use boolinator::Boolinator;
    pub use tap::prelude::{Conv, Tap, TapFallible, TapOptional, TryConv};
    //TODO: pub use tap::prelude::{Pipe}; - colides with duct::Expression.pipe!

    // Terminal
    #[cfg(feature = "term")]
    pub use ansi_term::{Colour, Style, ANSIString, ANSIStrings, unstyle};

    /// Returns true if stdout is a TTY
    #[cfg(feature = "term")]
    pub fn stdout_is_tty() -> bool {
        atty::is(atty::Stream::Stdout)
    }

    /// Returns true if stderr is a TTY
    #[cfg(feature = "term")]
    pub fn stderr_is_tty() -> bool {
        atty::is(atty::Stream::Stdout)
    }

    // Logging
    #[cfg(feature = "logging")]
    pub use log::{debug, error, info, log_enabled, trace, warn};

    #[cfg(feature = "args")]
    #[derive(Debug, Args)]
    pub struct DryRunOpt {
        /// Just print what would have been done
        #[arg(long = "dry-run", short = 'd')]
        pub enabled: bool,
    }

    #[cfg(all(feature = "args", feature = "logging"))]
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
        Utf8Error(PathBuf, std::str::Utf8Error),
    }

    impl Display for FileIoError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                FileIoError::IoError(path, _) => write!(f, "I/O error while reading file {:?}", path),
                FileIoError::Utf8Error(path, _) => write!(f, "failed to decode content of file {:?} as UTF-8 encoded string", path),
            }
        }
    }

    impl Error for FileIoError {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            match self {
                FileIoError::IoError(_, err) => Some(err),
                FileIoError::Utf8Error(_, err) => Some(err),
            }
        }
    }

    pub fn read_stdin() -> String {
        let mut buffer = String::new();
        stdin()
            .read_to_string(&mut buffer)
            .map_err(|err| format!("Failed to read UTF-8 string from stdin due to: {}", err))
            .unwrap();
        buffer
    }

    pub fn read_stdin_bytes() -> Vec<u8> {
        let mut buffer = Vec::new();
        stdin()
            .read_to_end(&mut buffer)
            .map_err(|err| format!("Failed to read bytes from stdin due to: {}", err))
            .unwrap();
        buffer
    }

    pub fn read_stdin_lines() -> impl Iterator<Item = String> {
        BufReader::new(stdin())
            .lines()
            .map(|val| val.map_err(|err| format!("Failed to read UTF-8 lines from stdin due to: {}", err)).unwrap())
    }

    /// Read content of all files as string.
    pub fn read_all(paths: impl IntoIterator<Item = impl AsRef<Path>>) -> Result<String, FileIoError> {
        let mut string = String::new();

        for path in paths {
            let path = path.as_ref();
            let mut file = File::open(path).map_err(|err| FileIoError::IoError(path.into(), err))?;
            file.read_to_string(&mut string).map_err(|err| FileIoError::IoError(path.into(), err))?;
        }

        Ok(string)
    }

    /// Read content of all files as bytes.
    pub fn read_all_bytes(paths: impl IntoIterator<Item = impl AsRef<Path>>) -> Result<Vec<u8>, FileIoError> {
        let mut bytes = Vec::new();

        for path in paths {
            let path = path.as_ref();
            let mut file = File::open(path).map_err(|err| FileIoError::IoError(path.into(), err))?;
            file.read_to_end(&mut bytes).map_err(|err| FileIoError::IoError(path.into(), err))?;
        }

        Ok(bytes)
    }

    #[cfg(all(feature = "args", feature = "logging"))]
    #[derive(Args)]
    pub struct ArgsLogger {
        /// Verbose mode (-v for INFO, -vv for DEBUG)
        #[arg(short = 'v', long, action = clap::ArgAction::Count)]
        pub verbose: u8,

        /// Quiet mode (-s for no WARN, -ss for no ERROR)
        #[arg(short = 'q', long, action = clap::ArgAction::Count)]
        quiet: u8,

        /// Force colorizing the logger output
        #[arg(long = "force-colors")]
        pub force_colors: bool,
    }

    #[cfg(all(feature = "args", feature = "logging"))]
    pub fn setup_logger(opt: ArgsLogger, module_paths: impl IntoIterator<Item = impl Into<String>>) {
        let verbosity = (opt.verbose + 1) as i16 - opt.quiet as i16;
        _setup_logger(verbosity, opt.force_colors, module_paths)
    }

    #[cfg(all(not(feature = "args"), feature = "logging"))]
    pub fn setup_logger(verbosity: i16, force_colors: bool, module_paths: impl IntoIterator<Item = impl Into<String>>) {
        _setup_logger(verbosity, force_colors, module_paths)
    }

    #[cfg(feature = "logging")]
    pub fn _setup_logger(verbosity: i16, force_colors: bool, module_paths: impl IntoIterator<Item = impl Into<String>>) {
        let mut logger = stderrlog::new();

        logger
            .quiet(verbosity < 0)
            .verbosity(verbosity as usize)
            .color(if force_colors { stderrlog::ColorChoice::Always } else { stderrlog::ColorChoice::Auto })
            .timestamp(stderrlog::Timestamp::Microsecond)
            .module(module_path!())
            .module("cotton")
            .module("problem");

        for module in module_paths {
            logger.module(module);
        }

        logger
            .init()
            .unwrap();

        #[cfg(feature = "problem")]
        problem::format_panic_to_error_log();
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
