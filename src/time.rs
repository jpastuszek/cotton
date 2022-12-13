pub use std::num::ParseFloatError;
pub use std::time::Duration;

pub trait DurationExt {
    /// Constructs Duration from &str parsed as f64 representing seconds.
    ///
    /// This is useful with `structopt` to get `Duration`: `parse(try_from_str = Duration::from_secs_str)`
    fn from_secs_str(val: &str) -> Result<Duration, ParseFloatError>;

    /// Constructs Duration from &str parsed as f64 representing milliseconds.
    ///
    /// This is useful with `structopt` to get `Duration`: `parse(try_from_str = Duration::from_millis_str)`
    fn from_millis_str(val: &str) -> Result<Duration, ParseFloatError>;
}

impl DurationExt for Duration {
    fn from_secs_str(val: &str) -> Result<Duration, ParseFloatError> {
        Ok(Duration::from_secs_f64(val.parse::<f64>()?))
    }

    fn from_millis_str(val: &str) -> Result<Duration, ParseFloatError> {
        Ok(Duration::from_secs_f64(val.parse::<f64>()? / 1000.0))
    }
}

pub use chrono::prelude::*;

/// Returns string in format YYYYMMDD (e.g. 20201009) based on UTC time zone
pub fn today_utc() -> String {
    Utc::now().date_naive().format("%Y%m%d").to_string()
}

/// Returns string in format YYYYMMDD (e.g. 20201009) based on local time zone
pub fn today() -> String {
    Local::now().date_naive().format("%Y%m%d").to_string()
}

pub trait ChoronoDurationExt {
    /// Constructs Duration from &str parsed as f64 representing seconds.
    ///
    /// This is useful with `structopt` to get `chrono::Duration`: `parse(try_from_str = chrono::Duration::from_secs_str)`
    fn from_secs_str(val: &str) -> Result<chrono::Duration, ParseFloatError>;

    /// Constructs Duration from &str parsed as f64 representing milliseconds.
    ///
    /// This is useful with `structopt` to get `chrono::Duration`: `parse(try_from_str = chrono::Duration::from_millis_str)`
    fn from_millis_str(val: &str) -> Result<chrono::Duration, ParseFloatError>;
}

impl ChoronoDurationExt for chrono::Duration {
    fn from_secs_str(val: &str) -> Result<chrono::Duration, ParseFloatError> {
        Ok(chrono::Duration::from_std(Duration::from_secs_str(val)?).unwrap())
    }

    fn from_millis_str(val: &str) -> Result<chrono::Duration, ParseFloatError> {
        Ok(chrono::Duration::from_std(Duration::from_millis_str(val)?).unwrap())
    }
}

/// Sleeps for duration.
pub fn sleep(duration: Duration) {
    std::thread::sleep(duration)
}

/// Sleeps for given number of seconds.
pub fn sleep_sec(seconds: f64) {
    std::thread::sleep(Duration::from_secs_f64(seconds))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duration_from_str() {
        let d = Duration::from_secs_str("1.5").unwrap();

        assert_eq!(d.as_millis(), 1500);

        assert_eq!(d, Duration::from_millis_str("1500").unwrap());
    }

    #[test]
    fn chrono_duration_from_str() {
        let d = chrono::Duration::from_secs_str("1.5").unwrap();

        assert_eq!(d.num_milliseconds(), 1500);

        assert_eq!(d, chrono::Duration::from_millis_str("1500").unwrap());
    }
}
