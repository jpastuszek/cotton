use problem::*;
use std::path::PathBuf;

pub use app_dirs::{AppDirsError, AppInfo};
pub const APP_INFO: AppInfo = AppInfo {
    name: env!("CARGO_PKG_NAME"),
    author: env!("CARGO_PKG_AUTHORS"),
};
use app_dirs::{app_dir, app_root, AppDataType};

/// Gets and creates if necessary application specific data directory.
///
/// If subdir is given then additional sub directory is crated.
pub fn app_data<'i>(subdir: impl Into<Option<&'i str>>) -> Result<PathBuf, Problem> {
    if let Some(subdir) = subdir.into() {
        app_dir(AppDataType::UserData, &APP_INFO, subdir)
    } else {
        app_root(AppDataType::UserData, &APP_INFO)
    }
    .problem_while("getting application data directory path")
}

/// Gets and creates if necessary application specific cache directory.
///
/// If subdir is given then additional sub directory is crated.
pub fn app_cache<'i>(subdir: impl Into<Option<&'i str>>) -> Result<PathBuf, Problem> {
    if let Some(subdir) = subdir.into() {
        app_dir(AppDataType::UserCache, &APP_INFO, subdir)
    } else {
        app_root(AppDataType::UserCache, &APP_INFO)
    }
    .problem_while("getting application data directory path")
}
