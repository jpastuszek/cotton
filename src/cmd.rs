use problem::*;

pub use duct::cmd;
pub use std::ffi::{OsString, OsStr};

pub trait ExpressionExt {
    /// Run command capturing stderr and stdout.
    ///
    /// Returns problem message containing stderr and stdout.
    fn silent(&self) -> Result<(), Problem>;
}

impl ExpressionExt for duct::Expression {
    fn silent(&self) -> Result<(), Problem> {
        let expr = self.clone();
        let out = self
            .stderr_to_stdout()
            .stdout_capture()
            .unchecked()
            .run()
            .problem_while_with(|| format!("while executing command {:?}", expr))?;

        if !out.status.success() {
            let code = out.status.code().map(|code| code.to_string()).unwrap_or("unknown".to_owned());
            let output = String::from_utf8(out.stdout).unwrap_or("<non-UTF-8 output>".to_owned());
            return Err(Problem::from_error(format!("command {:?} failed with status code {}:\n{}", expr, code, output)))
        }

        Ok(())
    }
}
