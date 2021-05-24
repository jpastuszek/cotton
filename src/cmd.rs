use problem::*;

pub use duct::cmd;
pub use std::convert::Infallible;
pub use std::ffi::{OsStr, OsString};
pub use std::path::Path;
pub use std::fmt::{self, Display};

/// Execute program with given path by replacing current program image.
///
/// Program name is taken from file stem of program path.
#[cfg(target_family = "unix")]
pub fn exec<S>(program: &Path, args: &[S]) -> Result<Infallible, Problem>
where
    S: AsRef<OsStr>,
{
    let name = program.file_stem().ok_or_problem_with(|| {
        format!(
            "Program path has no file stem and no program name given: {}",
            program.display()
        )
    })?;

    exec_with_name(program, name, args)
}

/// Execute program with given path by replacing current program image and using given name for
/// argument 0 of executed program.
#[cfg(target_family = "unix")]
pub fn exec_with_name<N, S>(program: &Path, name: N, args: &[S]) -> Result<Infallible, Problem>
where
    N: AsRef<OsStr>,
    S: AsRef<OsStr>,
{
    // arg[0] value for program
    let name: &OsStr = name.as_ref();
    let args = args.iter().map(|a| a.as_ref());

    let err = exec::execvp(program, Some(name).into_iter().chain(args));
    Err(err).problem_while(format!("executing program: {}", program.display()))
}

pub trait ExpressionExt {
    /// Runs command capturing stderr and stdout.
    ///
    /// Returns problem message containing stderr and stdout (unless it is not UTF-8).
    fn silent(&self) -> Result<(), Problem>;

    /// Runs command letting stderr and stdout to pass through.
    ///
    /// If the command finished with not successfull exit code this program will exit with that code.
    fn exec(&self) -> Result<(), Problem>;

    /// Runs command returning its exit code.
    ///
    /// If the command finishes without exit code (e.g. via signal) an "aborted" error is returned.
    fn run_with_status(&self) -> Result<i32, Problem>;

    /// Runs command capturing stdout and exit code.
    ///
    /// If the command finishes without exit code (e.g. via signal) an "aborted" error is returned.
    /// It will also fail if captured output can't be converted to UTF-8 string.
    fn read_with_status(&self) -> Result<(String, i32), Problem>;

    /// Runs command capturing stdout and exit code.
    ///
    /// If the command finishes without exit code (e.g. via signal) an "aborted" error is returned.
    fn read_with_status_bytes(&self) -> Result<(Vec<u8>, i32), Problem>;
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
            let code = out
                .status
                .code()
                .map(|code| code.to_string())
                .unwrap_or("unknown".to_owned());
            let output = String::from_utf8(out.stdout).unwrap_or("<non-UTF-8 output>".to_owned());
            return Err(Problem::from_error(format!(
                "command {:?} failed with status code {}:\n{}",
                expr, code, output
            )));
        }

        Ok(())
    }

    fn exec(&self) -> Result<(), Problem> {
        let expr = self.clone();
        let out = self
            .unchecked()
            .run()
            .problem_while_with(|| format!("while executing command {:?}", expr))?;

        if !out.status.success() {
            std::process::exit(out.status.code().unwrap())
        }

        Ok(())
    }

    fn run_with_status(&self) -> Result<i32, Problem> {
        let expr = self.clone();
        let out = self
            .unchecked()
            .run()
            .problem_while_with(|| format!("while executing command {:?}", expr))?;

        Ok(out.status.code().ok_or_problem("aborted")?)
    }

    fn read_with_status(&self) -> Result<(String, i32), Problem> {
        let (out, code) = self.read_with_status_bytes()?;

        Ok((String::from_utf8(out)?, code))
    }

    fn read_with_status_bytes(&self) -> Result<(Vec<u8>, i32), Problem> {
        let expr = self.clone();
        let out = self
            .stdout_capture()
            .unchecked()
            .run()
            .problem_while_with(|| format!("while executing command {:?}", expr))?;

        Ok((out.stdout, out.status.code().ok_or_problem("aborted")?))
    }
}

#[derive(Debug, Default, Clone)]
pub struct CmdArgs(Vec<OsString>);

impl CmdArgs {
    pub fn new() -> CmdArgs {
        Default::default()
    }

    pub fn with(mut self, arg: impl Into<OsString>) -> Self {
        self.push(arg);
        self
    }

    pub fn with_append(mut self, args: CmdArgs) -> Self {
        self.append(args);
        self
    }

    pub fn push(&mut self, arg: impl Into<OsString>) {
        self.0.push(arg.into())
    }

    pub fn append(&mut self, args: CmdArgs) {
        self.0.extend(args.0.into_iter());
    }

	pub fn into_expression(self, command: impl Into<OsString> + duct::IntoExecutablePath) -> duct::Expression {
        cmd(command, self.0)
	}

    pub fn as_expression(&self, command: impl Into<OsString> + duct::IntoExecutablePath) -> duct::Expression {
        cmd(command, self.0.as_slice())
    }

    pub fn into_inner(self) -> Vec<OsString> {
        self.0
    }
}

impl Display for CmdArgs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}
