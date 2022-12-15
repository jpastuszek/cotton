use maybe_string::MaybeString;
use std::process::ExitStatus;
#[cfg(all(target_family = "unix", feature = "exec"))]
use std::{path::{Path, PathBuf}, fmt::{self, Display}, error::Error, ffi::OsStr, convert::Infallible};

pub trait ExitStatusExt {
    /// Formats error message with status information and given error message.
    fn format_status_error(&self, stderr: Vec<u8>) -> String;
}

impl ExitStatusExt for ExitStatus {
    #[cfg(target_family = "unix")]
    fn format_status_error(&self, stderr: Vec<u8>) -> String {
        use std::os::unix::process::ExitStatusExt;
        match (self.code(), self.signal()) {
            (Some(code), _) => format!("Process exited with status code: {}; errors:\n{}", code, MaybeString(stderr)),
            (_, Some(signal)) => format!("Process aborted on signal: {}; errors:\n{}", signal, MaybeString(stderr)),
            _ => format!("Process was aborted; errors:\n{}", MaybeString(stderr)),
        }
    }

    #[cfg(not(target_family = "unix"))]
    fn format_status_error(&self, stderr: Vec<u8>) -> String {
        match self.code() {
            Some(code) => format!("Process exited with status code: {}; errors:\n{}", code, MaybeString(stderr)),
            _ => format!("Process was aborted; errors:\n{}", MaybeString(stderr)),
        }
    }
}

#[cfg(all(target_family = "unix", feature = "exec"))]
#[derive(Debug)]
pub enum ExecError {
    RunError(PathBuf, exec::Error),
    ProgramStemError(PathBuf),
}

#[cfg(all(target_family = "unix", feature = "exec"))]
impl Display for ExecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExecError::RunError(path, _) => write!(f, "executing program : {}", path.display()),
            ExecError::ProgramStemError(path) => write!(f, 
                "Program path has no file stem and no program name given: {}", path.display()),
        }
    }
}

#[cfg(all(target_family = "unix", feature = "exec"))]
impl Error for ExecError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ExecError::RunError(_, err) => Some(err),
            ExecError::ProgramStemError(_) => None, 
        }
    }
}

/// Executes program with given path by replacing current program image.
///
/// Program name is taken from file stem of program path.
#[cfg(all(target_family = "unix", feature = "exec"))]
pub fn exec<S>(program: &Path, args: &[S]) -> Result<Infallible, ExecError>
where
    S: AsRef<OsStr>,
{
    let name = program.file_stem().ok_or_else(|| ExecError::ProgramStemError(program.to_owned()))?;

    exec_with_name(program, name, args)
}

/// Executes program with given path by replacing current program image and using given name for
/// argument 0 of executed program.
#[cfg(all(target_family = "unix", feature = "exec"))]
pub fn exec_with_name<N, S>(program: &Path, name: N, args: &[S]) -> Result<Infallible, ExecError>
where
    N: AsRef<OsStr>,
    S: AsRef<OsStr>,
{
    // arg[0] value for program
    let name: &OsStr = name.as_ref();
    let args = args.iter().map(|a| a.as_ref());

    let err = exec::execvp(program, Some(name).into_iter().chain(args));
    Err(ExecError::RunError(program.to_owned(), err))
}
