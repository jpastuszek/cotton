mod app_dir;
mod cmd;
mod digest;
pub mod loggerv;
mod time;

// needed for derive to work
pub use structopt;

// Export crates to give access to unexported types
pub use boolinator;
pub use chrono;
pub use itertools;
pub use linked_hash_map;
pub use linked_hash_set;
pub use log;
pub use problem;
pub use shaman;
pub use tap;

pub mod prelude {
    // Often used I/O
    pub use std::fs::{
        canonicalize, copy, create_dir, create_dir_all, hard_link, metadata, read, read_dir,
        read_link, read_to_string, remove_dir, remove_dir_all, remove_file, rename,
        set_permissions, symlink_metadata, write, DirBuilder, DirEntry, File, FileType, Metadata,
        OpenOptions, Permissions, ReadDir,
    };
    pub use std::io::{stdin, stdout, BufRead, BufReader, BufWriter, Read, Write};

    pub use std::path::{Path, PathBuf};

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
    pub use std::fmt::{self, Debug, Display};

    // Arguments
    pub use structopt::StructOpt;

    // Error handling
    pub use assert_matches::assert_matches;
    pub use problem::prelude::*;
    pub use problem::result::FinalResult;
    pub use problem::result::Result as PResult;

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
    pub use tap::prelude::*;

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
