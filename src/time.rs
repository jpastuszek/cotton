pub use std::time::Duration;
pub use std::num::ParseFloatError;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duration_from_str() {
        let d = Duration::from_secs_str("1.5").unwrap();

        assert_eq!(d.as_millis(), 1500);

        assert_eq!(d, Duration::from_millis_str("1500").unwrap());
    }
}
