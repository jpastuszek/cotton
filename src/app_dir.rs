use std::{path::PathBuf, fmt::{self, Display}, error::Error};
use directories::{ProjectDirs, BaseDirs, UserDirs};

pub struct AppInfo {
    pub name: &'static str,
    pub author: &'static str,
}

//TODO: default to ARGV[0]?

//BUG: this will alwyas be "cotton" instead of client package
pub const APP_INFO: AppInfo = AppInfo {
    name: env!("CARGO_PKG_NAME"),
    author: env!("CARGO_PKG_AUTHORS"),
};

#[derive(Debug)]
pub enum AppDirError {
    NoProjectDir,
    NoBaseDir,
    NoUserDir,
}

impl Display for AppDirError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppDirError::NoProjectDir => write!(f, "getting project directories"),
            AppDirError::NoBaseDir  => write!(f, "getting base directories"),
            AppDirError::NoUserDir => write!(f, "getting user directories"),
        }
    }
}

impl Error for AppDirError {
}

pub fn project_dirs() -> Result<ProjectDirs, AppDirError> {
    ProjectDirs::from("", APP_INFO.author, APP_INFO.name).ok_or(AppDirError::NoProjectDir)
}

pub fn base_dirs() -> Result<BaseDirs, AppDirError> {
    BaseDirs::new().ok_or(AppDirError::NoBaseDir)
}

pub fn user_dirs() -> Result<UserDirs, AppDirError> {
    UserDirs::new().ok_or(AppDirError::NoUserDir)
}

/// Gets and creates if necessary application specific data directory.
///
/// If subdir is given then additional sub directory is crated.
pub fn app_data<'i>(subdir: impl Into<Option<&'i str>>) -> Result<PathBuf, AppDirError> {
    let data_dir = project_dirs()?;
    let data_dir = data_dir.data_dir();
    Ok(if let Some(subdir) = subdir.into() {
        data_dir.join(subdir)
    } else {
        data_dir.to_owned()
    })
}

/// Gets and creates if necessary application specific cache directory.
///
/// If subdir is given then additional sub directory is crated.
pub fn app_cache<'i>(subdir: impl Into<Option<&'i str>>) -> Result<PathBuf, AppDirError> {
    let cache_dir = project_dirs()?;
    let cache_dir = cache_dir.cache_dir();
    Ok(if let Some(subdir) = subdir.into() {
        cache_dir.join(subdir)
    } else {
        cache_dir.to_owned()
    })
}
