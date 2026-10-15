use async_graphql::{
    EmptyMutation, EmptySubscription, Enum, Error, InputObject, Object, Result, Schema,
    SimpleObject,
};
use counterpoint_core as core;

/// Longest line the API accepts, to keep requests cheap.
const MAX_NOTES: usize = 64;

pub type CounterpointSchema = Schema<Query, EmptyMutation, EmptySubscription>;

/// Builds the GraphQL schema.
pub fn build() -> CounterpointSchema {
    Schema::build(Query, EmptyMutation, EmptySubscription).finish()
}

/// Whether the counterpoint sits above or below the cantus firmus.
#[derive(Enum, Clone, Copy, PartialEq, Eq)]
pub enum Placement {
    Above,
    Below,
}

/// How serious a broken rule is.
#[derive(Enum, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

/// Which line a violation belongs to.
#[derive(Enum, Clone, Copy, PartialEq, Eq)]
pub enum Voice {
    CantusFirmus,
    Counterpoint,
}

/// A first species exercise. Pitches are written like C4, F#3 or Bb4, middle C is C4.
#[derive(InputObject)]
pub struct FirstSpeciesInput {
    /// Key such as "C major" or "F# minor".
    key: String,
    placement: Placement,
    cantus_firmus: Vec<String>,
    counterpoint: Vec<String>,
}

/// A broken rule; `rule` is the id from RULES.md.
#[derive(SimpleObject)]
pub struct Violation {
    rule: String,
    severity: Severity,
    bar: usize,
    beat: u8,
    /// The line at fault, or null when the problem is between the voices.
    voice: Option<Voice>,
    message: String,
}

/// The outcome of checking an exercise.
#[derive(SimpleObject)]
pub struct CheckResult {
    /// True when there are no errors; warnings are allowed.
    valid: bool,
    violations: Vec<Violation>,
}

pub struct Query;

#[Object]
impl Query {
    /// Engine version.
    async fn version(&self) -> &'static str {
        core::version()
    }

    /// Checks a first species exercise against the rules in RULES.md.
    async fn check_first_species(&self, input: FirstSpeciesInput) -> Result<CheckResult> {
        let exercise = core::FirstSpecies {
            key: input
                .key
                .parse()
                .map_err(|e: core::ParseError| Error::new(e.0))?,
            placement: match input.placement {
                Placement::Above => core::Placement::Above,
                Placement::Below => core::Placement::Below,
            },
            cantus_firmus: parse_line("cantusFirmus", &input.cantus_firmus)?,
            counterpoint: parse_line("counterpoint", &input.counterpoint)?,
        };
        let violations: Vec<Violation> = exercise
            .check()
            .map_err(|e| Error::new(e.0))?
            .into_iter()
            .map(Violation::from)
            .collect();
        Ok(CheckResult {
            valid: violations.iter().all(|v| v.severity != Severity::Error),
            violations,
        })
    }
}

/// Parses one voice, naming the field and note index on failure.
fn parse_line(field: &str, notes: &[String]) -> Result<Vec<core::Pitch>> {
    if notes.len() > MAX_NOTES {
        return Err(Error::new(format!(
            "{field} has more than {MAX_NOTES} notes."
        )));
    }
    notes
        .iter()
        .enumerate()
        .map(|(i, s)| {
            s.parse()
                .map_err(|e: core::ParseError| Error::new(format!("{field}[{i}]: {}", e.0)))
        })
        .collect()
}

impl From<core::Violation> for Violation {
    fn from(v: core::Violation) -> Violation {
        Violation {
            rule: v.rule.to_string(),
            severity: match v.severity {
                core::Severity::Error => Severity::Error,
                core::Severity::Warning => Severity::Warning,
            },
            bar: v.at.bar,
            beat: v.at.beat,
            voice: v.voice.map(|voice| match voice {
                core::Voice::CantusFirmus => Voice::CantusFirmus,
                core::Voice::Counterpoint => Voice::Counterpoint,
            }),
            message: v.message,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_graphql::value;

    async fn run(query: &str) -> async_graphql::Response {
        build().execute(query).await
    }

    #[tokio::test]
    async fn valid_exercise_passes() {
        let res = run(r#"{ checkFirstSpecies(input: {
            key: "C major", placement: ABOVE,
            cantusFirmus: ["C4","D4","F4","E4","G4","A4","F4","E4","D4","C4"],
            counterpoint: ["C5","B4","A4","C5","B4","C5","D5","C5","B4","C5"]
        }) { valid violations { rule } } }"#)
        .await;
        assert!(res.errors.is_empty(), "{:?}", res.errors);
        assert_eq!(
            res.data,
            value!({ "checkFirstSpecies": { "valid": true, "violations": [] } })
        );
    }

    #[tokio::test]
    async fn reports_violations() {
        let res = run(r#"{ checkFirstSpecies(input: {
            key: "C major", placement: ABOVE,
            cantusFirmus: ["C4","D4","E4","D4","C4"],
            counterpoint: ["G4","A4","G4","B4","C5"]
        }) { valid violations { rule severity bar beat voice } } }"#)
        .await;
        assert!(res.errors.is_empty(), "{:?}", res.errors);
        let json = res.data.into_json().unwrap();
        assert_eq!(json["checkFirstSpecies"]["valid"], false);
        let parallel = json["checkFirstSpecies"]["violations"]
            .as_array()
            .unwrap()
            .iter()
            .find(|v| v["rule"] == "parallel-perfect")
            .expect("parallel-perfect");
        assert_eq!(parallel["severity"], "ERROR");
        assert_eq!(parallel["bar"], 2);
        assert!(parallel["voice"].is_null());
    }

    #[tokio::test]
    async fn bad_pitch_names_the_note() {
        let res = run(r#"{ checkFirstSpecies(input: {
            key: "C major", placement: ABOVE,
            cantusFirmus: ["C4","H4"], counterpoint: ["C5","C5"]
        }) { valid } }"#)
        .await;
        assert!(
            res.errors[0]
                .message
                .starts_with("cantusFirmus[1]: invalid pitch 'H4'")
        );
    }

    #[tokio::test]
    async fn mismatched_lengths_is_an_error() {
        let res = run(r#"{ checkFirstSpecies(input: {
            key: "C major", placement: BELOW,
            cantusFirmus: ["C4","D4","C4"], counterpoint: ["C3","B2"]
        }) { valid } }"#)
        .await;
        assert_eq!(res.errors.len(), 1);
    }
}
