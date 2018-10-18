extern crate problem;
#[macro_use]
extern crate structopt;

pub extern crate log;
extern crate loggerv;

pub mod prelude {
    pub use std::io::{stdin, stdout, Read, Write, BufReader, BufRead, BufWriter};
    pub use std::fs::File;
    pub use problem::*;
    pub use structopt::StructOpt;

    #[derive(Debug, StructOpt)]
    pub struct LoggingArgs {
        /// Verbose mode (-v for Debug, -vv for Trace, -vvv Trace all modules)
        #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
        pub verbose: u8,

        /// Force colorizing the logger output
        #[structopt(long = "force-colors")]
        pub force_colors: bool,
    }

    pub fn read_stdin() -> String {
        let mut buffer = String::new();
        stdin().read_to_string(&mut buffer).or_failed_to("read UTF-8 string from stdin");
        buffer
    }

    pub fn init_logger(args: &LoggingArgs, module_path: impl Into<String>) {
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
                .add_module_path_filter(module_path)
                .add_module_path_filter("problem");
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

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
