pub mod loggerv;
mod digest;
mod app_dir;
mod cmd;

// needed for derive to work
pub use structopt;

// there is some extra stuff in there to be used
pub use problem;

pub mod prelude {
    // Often used I/O
    pub use std::io::{stdin, stdout, Read, Write, BufReader, BufRead, BufWriter};
    pub use std::fs::{self, File};
    pub use std::path::{PathBuf, Path};

    // Logging and messaging
    pub use crate::loggerv;
    pub use std::fmt::{self, Display, Debug};
    pub use log::{self, trace, debug, info, warn, error, log_enabled};

    // Arguments
    pub use structopt::StructOpt;

    // Error handling
    pub use problem::prelude::*;
    pub use problem::result::*;
    pub use assert_matches::assert_matches;

    // Running commands
    pub use super::cmd::*;
    pub use exec::execvp as exec;

    // Content hashing
    pub use super::digest::*;

    // App directory
    pub use super::app_dir::*;

    // Handy extensions
    pub use boolinator::Boolinator;
    pub use itertools::*;
    pub use tap::*;

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
        #[structopt(long = "dry-run",short = "-d")]
        pub enabled: bool,
    }

    impl DryRunOpt {
        pub fn run(&self, msg: impl Display, run: impl FnOnce() -> ()) -> () {
            if self.enabled {
                info!("[dry run]: {}", msg);
            }  else {
                info!("{}", msg);
                run()
            }
        }
    }

    pub fn read_stdin() -> String {
        let mut buffer = String::new();
        stdin().read_to_string(&mut buffer).or_failed_to("read UTF-8 string from stdin");
        buffer
    }

    pub fn read_stdin_bytes() -> Vec<u8> {
        let mut buffer = Vec::new();
        stdin().read_to_end(&mut buffer).or_failed_to("read bytes from stdin");
        buffer
    }

    pub fn read_stdin_lines() -> impl Iterator<Item = String> {
        BufReader::new(stdin()).lines().or_failed_to("read UTF-8 lines from stdin")
    }

    pub fn init_logger(args: &LoggingOpt, module_paths: impl IntoIterator<Item = impl Into<String>>) {
        use log::Level;
        use loggerv::{Logger, Output};

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

            logger = module_paths.into_iter().fold(logger, |logger, module_path|
                logger.add_module_path_filter(module_path)
            );
        }

        if args.force_colors {
            logger = logger
                .force_colors()
        }

        logger.level(true)
            .init()
            .or_failed_to("init logger");

        ::problem::format_panic_to_error_log();
    }
}
