pub mod prelude {
    pub use std::io::{stdin, stdout, Read, Write, BufReader, BufRead, BufWriter};
    pub use std::fs::File;
    pub use problem::prelude::*;
    pub use structopt::StructOpt;
    pub use std::fmt::{self, Display, Debug};
    pub use log::{self, trace, debug, info, warn, error, log_enabled}; 
    pub use boolinator::Boolinator;
    pub use itertools::*;
    pub use tap::*;

    #[derive(Debug, StructOpt)]
    pub struct LoggingOpt {
        /// Verbose mode (-v for Debug, -vv for Trace, -vvv Trace all modules)
        #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
        pub verbose: u8,

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

    pub fn init_logger(args: &LoggingOpt, module_paths: impl IntoIterator<Item = impl Into<String>>) {
        use log::Level;
        use loggerv::{Logger, Output};

        let mut logger = Logger::new()
            .base_level(Level::Info)
            .verbosity(args.verbose as u64)
            .output(&Level::Info, Output::Stderr)
            .output(&Level::Debug, Output::Stderr)
            .output(&Level::Trace, Output::Stderr)
            .module_path(false);

        if args.verbose <= 2 {
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
