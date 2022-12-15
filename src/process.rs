use maybe_string::MaybeStr;
use std::process::ExitStatus;
use std::fmt::{self, Display};
use std::error::Error;
#[cfg(all(target_family = "unix", feature = "exec"))]
use std::{path::{Path, PathBuf}, ffi::OsStr, convert::Infallible};
#[cfg(feature = "cradle")]
use cradle::output::{Status, Stderr, StdoutTrimmed};

#[derive(Debug)]
pub struct StatusError {
    code: Option<i32>,
    #[cfg(target_family = "unix")]
    signal: Option<i32>,
    output: Vec<u8>,
}

impl Display for StatusError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.code, self.signal) {
            (Some(code), _) => write!(f, "Process exited with status code: {}; errors:\n{}", code, MaybeStr::from_bytes(&self.output)),
            #[cfg(target_family = "unix")]
            (_, Some(signal)) => write!(f, "Process aborted on signal: {}; errors:\n{}", signal, MaybeStr::from_bytes(&self.output)),
            _ => write!(f, "Process was aborted; errors:\n{}", MaybeStr::from_bytes(&self.output)),
        }
    }
}

impl Error for StatusError {
}

pub trait ExitStatusExt {
    /// Formats error message with status information and given error message.
    fn format_status_error(&self, stderr: Vec<u8>) -> StatusError;
    /// Returns [StatusError] if exit status code was not zero.
    fn success_or_err(&self, stderr: Vec<u8>) -> Result<(), StatusError>;
}

impl ExitStatusExt for ExitStatus {
    fn format_status_error(&self, stderr: Vec<u8>) -> StatusError {
        #[cfg(target_family = "unix")]
        use std::os::unix::process::ExitStatusExt;
        StatusError {
            code: self.code(), 
            #[cfg(target_family = "unix")]
            signal: self.signal(),
            output: stderr,
        }
    }

    fn success_or_err(&self, stderr: Vec<u8>) -> Result<(), StatusError> {
        if !self.success() {
            Err(self.format_status_error(stderr))
        } else {
            Ok(())
        }
    }
}

/// Returns [StatusError] if process status code was not zero.
#[cfg(feature = "cradle")]
pub fn check_status(Status(status): Status) -> Result<(), StatusError> {
    status.success_or_err(b"check output above for errors".to_vec())
}

/// Captures programs stderr output and provides it as error message if run was not successfull.
#[cfg(feature = "cradle")]
pub fn collect_errors((Status(status), Stderr(stderr)): (Status, Stderr)) -> Result<(), StatusError> {
    status.success_or_err(stderr.into_bytes())
}

#[cfg(feature = "cradle")]
/// Captures programs stdout and stderr outputs and provides stdout on success and stderr as part of error message on failure.
pub fn collect_output_and_errors(
    (Status(status), StdoutTrimmed(stdout), Stderr(stderr)): (Status, StdoutTrimmed, Stderr)
) -> Result<String, StatusError> {
    status.success_or_err(stderr.into_bytes()).map(|_| stdout)
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
