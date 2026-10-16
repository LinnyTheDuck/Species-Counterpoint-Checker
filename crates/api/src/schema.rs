use async_graphql::{
    EmptyMutation, EmptySubscription, Enum, Error, InputObject, Object, Result, Schema,
    SimpleObject,
};
use counterpoint_core as core;

/// Longest line the API accepts, to keep requests cheap.
const MAX_NOTES: usize = 64;

pub type CounterpointSchema = Schema<Query, EmptyMutation, EmptySubscription>;

pub fn build() -> CounterpointSchema {
    Schema::build(Query, EmptyMutation, EmptySubscription).finish()
}

/// Whether the counterpoint sits above or below the cantus firmus.
#[derive(Enum, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Placement {
    Above,
    Below,
}

/// How serious a broken rule is.
#[derive(Enum, Clone, Copy, PartialEq, Eq, Debug)]
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

/// A first species exercise, with pitches written like C4, F#3 or Bb4 and middle C as C4.
#[derive(InputObject)]
pub struct FirstSpeciesInput {
    /// Key such as "C major" or "F# minor".
    key: String,
    placement: Placement,
    cantus_firmus: Vec<String>,
    counterpoint: Vec<String>,
    /// Bars, counted from 1, whose counterpoint note is tied into the next bar.
    #[graphql(default)]
    ties: Vec<usize>,
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

/// A rule the checker applies; `id` matches the violations.
#[derive(SimpleObject)]
pub struct Rule {
    id: &'static str,
    group: &'static str,
    severity: Severity,
    summary: &'static str,
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

    /// Rules first species checks, in display order.
    async fn first_species_rules(&self) -> Vec<Rule> {
        core::FIRST_SPECIES_RULES
            .iter()
            .map(|r| Rule {
                id: r.id,
                group: r.group,
                severity: r.severity.into(),
                summary: r.summary,
            })
            .collect()
    }

    /// Checks a first species exercise against the rules in RULES.md.
    async fn check_first_species(&self, input: FirstSpeciesInput) -> Result<CheckResult> {
        eprintln!(
            "checkFirstSpecies key=\"{}\" placement={:?}\n  cf: {}\n  cp: {}\n  ties: {:?}",
            input.key,
            input.placement,
            input.cantus_firmus.join(" "),
            input.counterpoint.join(" "),
            input.ties
        );
        let result = check(input);
        match &result {
            Ok(r) => {
                for v in &r.violations {
                    eprintln!("  bar {} {:?} {}: {}", v.bar, v.severity, v.rule, v.message);
                }
                eprintln!("  -> valid={}", r.valid);
            }
            Err(e) => eprintln!("  -> input error: {}", e.message),
        }
        result
    }
}

fn check(input: FirstSpeciesInput) -> Result<CheckResult> {
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
        ties: input.ties,
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

impl From<core::Severity> for Severity {
    fn from(s: core::Severity) -> Severity {
        match s {
            core::Severity::Error => Severity::Error,
            core::Severity::Warning => Severity::Warning,
        }
    }
}

impl From<core::Violation> for Violation {
    fn from(v: core::Violation) -> Violation {
        Violation {
            rule: v.rule.to_string(),
            severity: v.severity.into(),
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
    async fn accepts_ties() {
        let res = run(r#"{ checkFirstSpecies(input: {
            key: "A minor", placement: BELOW,
            cantusFirmus: ["A4","B4","C5","B4","D5","C5","D5","E5","B4","A4"],
            counterpoint: ["A2","G#2","A2","B2","B2","E3","F3","E3","G#2","A2"],
            ties: [4]
        }) { valid violations { rule } } }"#)
        .await;
        assert!(res.errors.is_empty(), "{:?}", res.errors);
        assert_eq!(
            res.data,
            value!({ "checkFirstSpecies": { "valid": true, "violations": [] } })
        );
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
