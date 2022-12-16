use std::{path::{PathBuf, Path}, fmt::{self, Display}, error::Error};
use directories::{ProjectDirs, BaseDirs, UserDirs};
use std::sync::Mutex;

struct AppInfo {
    pub name: String,
    pub author: String,
}

static APP_INFO: Mutex<Option<AppInfo>> = Mutex::new(None);

/// Initializes application name and author with CARGO_PKG_NAME and CARGO_PKG_AUTHORS.
#[macro_export]
macro_rules! init_app_info {
    () => {
        init_app_info_with(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_AUTHORS"))
    };
}

pub use init_app_info;

/// Initializes application name and author with given values.
pub fn init_app_info_with(name: impl Into<String>, author: impl Into<String>) {
    let mut app_info = APP_INFO.lock().unwrap();
    if app_info.is_none() {
        app_info.replace(AppInfo { name: name.into(), author: author.into() });
    }
}

/// Initializes application name and author guessing from environment.
pub fn init_app_info_guess() {
    let mut app_info = APP_INFO.lock().unwrap();
    if app_info.is_none() {
        let name = std::env::args().next().and_then(|a| Path::new(&a).file_name().and_then(|n| n.to_str().map(ToOwned::to_owned)));
        app_info.replace(AppInfo { name: name.unwrap_or("cotton".to_owned()), author: "Anonymous".to_owned() });
    }
}

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
    init_app_info_guess();
    let app_info = APP_INFO.lock().unwrap();
    let app_info = app_info.as_ref().unwrap();

    ProjectDirs::from("", &app_info.author, &app_info.name).ok_or(AppDirError::NoProjectDir)
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
