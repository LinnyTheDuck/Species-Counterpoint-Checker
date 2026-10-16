#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

/// Which line a violation belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Voice {
    CantusFirmus,
    Counterpoint,
}

/// A place in the score, bar and beat both counted from 1.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Position {
    pub bar: usize,
    pub beat: u8,
}

/// A broken rule, with the rule id from RULES.md.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Violation {
    pub rule: &'static str,
    pub severity: Severity,
    pub at: Position,
    /// The line at fault, or None when the problem is between the voices.
    pub voice: Option<Voice>,
    pub message: String,
}

impl Violation {
    pub fn error(
        rule: &'static str,
        at: Position,
        voice: Option<Voice>,
        message: String,
    ) -> Violation {
        Violation {
            rule,
            severity: Severity::Error,
            at,
            voice,
            message,
        }
    }

    pub fn warning(
        rule: &'static str,
        at: Position,
        voice: Option<Voice>,
        message: String,
    ) -> Violation {
        Violation {
            rule,
            severity: Severity::Warning,
            at,
            voice,
            message,
        }
    }
}
