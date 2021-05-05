use problem::*;
use std::path::PathBuf;
use directories::{ProjectDirs, BaseDirs, UserDirs};

pub struct AppInfo {
    pub name: &'static str,
    pub author: &'static str,
}

//BUG: this will alwyas be "cotton" instead of client package
pub const APP_INFO: AppInfo = AppInfo {
    name: env!("CARGO_PKG_NAME"),
    author: env!("CARGO_PKG_AUTHORS"),
};

pub fn project_dirs() -> Result<ProjectDirs, Problem> {
    ProjectDirs::from("", APP_INFO.author, APP_INFO.name).ok_or_problem("getting project directories")
}

pub fn base_dirs() -> Result<BaseDirs, Problem> {
    BaseDirs::new().ok_or_problem("getting base directories")
}

pub fn user_dirs() -> Result<UserDirs, Problem> {
    UserDirs::new().ok_or_problem("getting user directories")
}

/// Gets and creates if necessary application specific data directory.
///
/// If subdir is given then additional sub directory is crated.
pub fn app_data<'i>(subdir: impl Into<Option<&'i str>>) -> Result<PathBuf, Problem> {
    in_context_of("getting application data directory path", || {
        let data_dir = project_dirs()?;
        let data_dir = data_dir.data_dir();
        Ok(if let Some(subdir) = subdir.into() {
            data_dir.join(subdir)
        } else {
            data_dir.to_owned()
        })
    })
}

/// Gets and creates if necessary application specific cache directory.
///
/// If subdir is given then additional sub directory is crated.
pub fn app_cache<'i>(subdir: impl Into<Option<&'i str>>) -> Result<PathBuf, Problem> {
    in_context_of("getting application data directory path", || {
        let cache_dir = project_dirs()?;
        let cache_dir = cache_dir.cache_dir();
        Ok(if let Some(subdir) = subdir.into() {
            cache_dir.join(subdir)
        } else {
            cache_dir.to_owned()
        })
    })
}
