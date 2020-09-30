use problem::*;

pub use std::convert::Infallible;
pub use std::path::Path;
pub use duct::cmd;
pub use std::ffi::{OsString, OsStr};

/// Execute program with given path by replacing current program image.
///
/// Program name is taken from file stem of program path.
#[cfg(target_family = "unix")]
pub fn exec<S>(program: &Path, args: &[S]) -> Result<Infallible, Problem>
where
    S: AsRef<OsStr>,
{
    let name = program
        .file_stem().ok_or_problem_with(|| format!("Program path has no file stem and no program name given: {}", program.display()))?;

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
    /// Run command capturing stderr and stdout.
    ///
    /// Returns problem message containing stderr and stdout.
    fn silent(&self) -> Result<(), Problem>;

    /// Run command and exit with it's exit status code if not successful.
    fn exec(&self) -> Result<(), Problem>;
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

    fn exec(&self) -> Result<(), Problem> {
        let expr = self.clone();
        let out = self
            .unchecked()
            .run()
            .problem_while_with(|| format!("while executing command {:?}", expr))?;
        if !out.status.success() {
            std::process::exit(out.status.code().unwrap())
        }
        Ok(()) //TODO: !
    }
}
