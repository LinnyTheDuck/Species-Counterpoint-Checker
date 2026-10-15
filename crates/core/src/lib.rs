//! Species counterpoint rule engine. Rule ids match RULES.md.

use std::fmt;

pub mod catalogue;
mod interval;
mod key;
mod pitch;
pub mod rules;
pub mod species1;
mod violation;

pub use catalogue::{FIRST_SPECIES_RULES, RuleInfo};
pub use interval::{Interval, Quality};
pub use key::{Key, Mode};
pub use pitch::{Letter, Pitch};
pub use rules::Placement;
pub use species1::FirstSpecies;
pub use violation::{Position, Severity, Violation, Voice};

/// A pitch or key that could not be parsed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseError(pub String);

/// An exercise that cannot be checked, such as voices of different lengths.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InputError(pub String);

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl fmt::Display for InputError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ParseError {}
impl std::error::Error for InputError {}

/// Returns the engine version.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_set() {
        assert_eq!(version(), "0.1.0");
    }
}
