use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Formatter;
use std::str::FromStr;
use thiserror::Error;

/// Error type for probability operations
#[derive(Debug, Error)]
pub enum ProbabilityError {
    /// Returned when a probability value is out of the valid range (0.0-1.0)
    #[error("{0} is not in the valid probability range of 0.0 to 1.0")]
    OutOfRange(f64),

    /// Returned when a string can't be parsed as a valid float
    #[error("'{0}' is not a valid number for a probability value")]
    ParseError(String),
}

/// Represents a probability value between 0.0 and 1.0.
///
/// This type ensures that probability values are always within
/// the valid range of 0.0 (impossible) to 1.0 (certain).
///
/// # Example
///
/// ```
/// let p = Probability::new(0.5).unwrap(); // 50% chance
/// assert_eq!(p.value(), 0.5);
///
/// assert!(Probability::new(1.5).is_err());
/// ```
#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct Probability(f64);

impl Probability {
    /// Creates a new probability value, ensuring it's between 0.0 and 1.0.
    ///
    /// # Arguments
    ///
    /// * `value` - The probability value between 0.0 and 1.0
    ///
    /// # Returns
    ///
    /// * `Result<Self, ProbabilityError>` - A Result containing the Probability
    ///   or an error if the value is outside the valid range
    pub fn new(value: f64) -> Result<Self, ProbabilityError> {
        if !(0.0..=1.0).contains(&value) {
            return Err(ProbabilityError::OutOfRange(value));
        }

        Ok(Probability(value))
    }

    /// Returns the underlying probability value.
    ///
    /// # Returns
    ///
    /// * `f64` - The probability value between 0.0 and 1.0
    pub fn value(&self) -> f64 {
        self.0
    }
}

impl FromStr for Probability {
    type Err = ProbabilityError;

    /// Converts a string to a probability value.
    ///
    /// # Arguments
    ///
    /// * `s` - The string to parse
    ///
    /// # Returns
    ///
    /// * `Result<Self, Self::Err>` - A Result containing the Probability
    ///   or an error if the string can't be parsed or the value is invalid
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value: f64 = s
            .parse()
            .map_err(|_| ProbabilityError::ParseError(s.to_string()))?;
        Probability::new(value)
    }
}

impl From<Probability> for f64 {
    /// Converts a Probability to an f64.
    ///
    /// # Arguments
    ///
    /// * `prob` - The Probability to convert
    ///
    /// # Returns
    ///
    /// * `f64` - The probability value
    fn from(prob: Probability) -> Self {
        prob.0
    }
}

impl Default for Probability {
    /// Creates a default probability of 0.0 (impossible).
    ///
    /// # Returns
    ///
    /// * `Self` - A Probability with value 0.0
    fn default() -> Self {
        Probability(0.0)
    }
}

impl fmt::Display for Probability {
    /// Formats the probability value for display.
    ///
    /// # Arguments
    ///
    /// * `f` - Formatter
    ///
    /// # Returns
    ///
    /// * `fmt::Result` - Result of the formatting operation
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_probability() {
        let p = Probability::new(0.5).unwrap();
        assert_eq!(p.value(), 0.5);
    }

    #[test]
    fn test_invalid_probability() {
        assert!(Probability::new(-0.1).is_err());
        assert!(Probability::new(1.1).is_err());
    }

    #[test]
    fn test_from_string() {
        let p = "0.75".parse::<Probability>().unwrap();
        assert_eq!(p.value(), 0.75);

        assert!("not_a_number".parse::<Probability>().is_err());
        assert!("1.5".parse::<Probability>().is_err());
    }

    #[test]
    fn test_default() {
        let p = Probability::default();
        assert_eq!(p.value(), 0.0);
    }

    #[test]
    fn test_display() {
        let p = Probability::new(0.25).unwrap();
        assert_eq!(format!("{}", p), "0.25");
    }
}
