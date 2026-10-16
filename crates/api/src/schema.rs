use async_graphql::{
    EmptyMutation, EmptySubscription, Enum, Error, InputObject, Object, Result, Schema,
    SimpleObject,
};
use counterpoint_core as core;

/// Longest line the API accepts, to keep requests cheap.
const MAX_NOTES: usize = 64;
/// Most notes a bar may hold; a wrong count under this is reported as `s2-rhythm`.
const MAX_NOTES_PER_BAR: usize = 4;

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

/// A second species exercise, with null in the counterpoint for the opening minim rest.
#[derive(InputObject)]
pub struct SecondSpeciesInput {
    /// Key such as "C major" or "F# minor".
    key: String,
    placement: Placement,
    cantus_firmus: Vec<String>,
    /// One list per bar: two minims, or one semibreve in the last two bars.
    counterpoint: Vec<Vec<Option<String>>>,
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
        rules(core::FIRST_SPECIES_RULES)
    }

    /// Rules second species checks, in display order.
    async fn second_species_rules(&self) -> Vec<Rule> {
        rules(core::SECOND_SPECIES_RULES)
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
        log_result(check_first(input))
    }

    /// Checks a second species exercise against the rules in RULES.md.
    async fn check_second_species(&self, input: SecondSpeciesInput) -> Result<CheckResult> {
        let bars: Vec<String> = input
            .counterpoint
            .iter()
            .map(|bar| {
                let notes: Vec<&str> = bar.iter().map(|p| p.as_deref().unwrap_or("rest")).collect();
                notes.join(" ")
            })
            .collect();
        eprintln!(
            "checkSecondSpecies key=\"{}\" placement={:?}\n  cf: {}\n  cp: {}",
            input.key,
            input.placement,
            input.cantus_firmus.join(" "),
            bars.join(" | ")
        );
        log_result(check_second(input))
    }
}

/// Logs the violations or input error for a check, then passes the result on.
fn log_result(result: Result<CheckResult>) -> Result<CheckResult> {
    match &result {
        Ok(r) => {
            for v in &r.violations {
                eprintln!(
                    "  bar {} beat {} {:?} {}: {}",
                    v.bar, v.beat, v.severity, v.rule, v.message
                );
            }
            eprintln!("  -> valid={}", r.valid);
        }
        Err(e) => eprintln!("  -> input error: {}", e.message),
    }
    result
}

fn rules(list: &[core::RuleInfo]) -> Vec<Rule> {
    list.iter()
        .map(|r| Rule {
            id: r.id,
            group: r.group,
            severity: r.severity.into(),
            summary: r.summary,
        })
        .collect()
}

fn parse_key(key: &str) -> Result<core::Key> {
    key.parse().map_err(|e: core::ParseError| Error::new(e.0))
}

/// Turns engine output into the API result, or an engine input error into a GraphQL error.
fn result(
    violations: std::result::Result<Vec<core::Violation>, core::InputError>,
) -> Result<CheckResult> {
    let violations: Vec<Violation> = violations
        .map_err(|e| Error::new(e.0))?
        .into_iter()
        .map(Violation::from)
        .collect();
    Ok(CheckResult {
        valid: violations.iter().all(|v| v.severity != Severity::Error),
        violations,
    })
}

fn check_first(input: FirstSpeciesInput) -> Result<CheckResult> {
    let exercise = core::FirstSpecies {
        key: parse_key(&input.key)?,
        placement: input.placement.into(),
        cantus_firmus: parse_line("cantusFirmus", &input.cantus_firmus)?,
        counterpoint: parse_line("counterpoint", &input.counterpoint)?,
        ties: input.ties,
    };
    result(exercise.check())
}

fn check_second(input: SecondSpeciesInput) -> Result<CheckResult> {
    if input.counterpoint.len() > MAX_NOTES {
        return Err(Error::new(format!(
            "counterpoint has more than {MAX_NOTES} bars."
        )));
    }
    let counterpoint = input
        .counterpoint
        .iter()
        .enumerate()
        .map(|(i, bar)| {
            if bar.len() > MAX_NOTES_PER_BAR {
                return Err(Error::new(format!(
                    "counterpoint[{i}] has more than {MAX_NOTES_PER_BAR} notes."
                )));
            }
            bar.iter()
                .enumerate()
                .map(|(j, p)| {
                    p.as_deref()
                        .map(|p| parse_pitch(&format!("counterpoint[{i}]"), j, p))
                        .transpose()
                })
                .collect()
        })
        .collect::<Result<_>>()?;
    let exercise = core::SecondSpecies {
        key: parse_key(&input.key)?,
        placement: input.placement.into(),
        cantus_firmus: parse_line("cantusFirmus", &input.cantus_firmus)?,
        counterpoint,
    };
    result(exercise.check())
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
        .map(|(i, s)| parse_pitch(field, i, s))
        .collect()
}

fn parse_pitch(field: &str, index: usize, s: &str) -> Result<core::Pitch> {
    s.parse()
        .map_err(|e: core::ParseError| Error::new(format!("{field}[{index}]: {}", e.0)))
}

impl From<Placement> for core::Placement {
    fn from(p: Placement) -> core::Placement {
        match p {
            Placement::Above => core::Placement::Above,
            Placement::Below => core::Placement::Below,
        }
    }
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
    async fn second_species_assignment_passes() {
        // The page's second species example, as checker.js sends it.
        let res = run(r#"{ checkSecondSpecies(input: {
            key: "D minor", placement: ABOVE,
            cantusFirmus: ["D3","F3","E3","G3","A3","G3","F3","E3","C#3","D3"],
            counterpoint: [[null,"F4"],["A4","Bb4"],["C5","G4"],["E4","G4"],["F4","A4"],
                           ["B4","C#5"],["D5","A4"],["G4","F4"],["E4"],["D4"]]
        }) { valid violations { rule } } }"#)
        .await;
        assert!(res.errors.is_empty(), "{:?}", res.errors);
        assert_eq!(
            res.data,
            value!({ "checkSecondSpecies": { "valid": true, "violations": [] } })
        );
    }

    #[tokio::test]
    async fn second_species_reports_beat() {
        let res = run(r#"{ checkSecondSpecies(input: {
            key: "C major", placement: ABOVE,
            cantusFirmus: ["C4","E4","D4","C4"],
            counterpoint: [["C5","D5"],["C5","G4"],["A4","B4"],["C5"]]
        }) { violations { rule bar beat } } }"#)
        .await;
        assert!(res.errors.is_empty(), "{:?}", res.errors);
        let json = res.data.into_json().unwrap();
        let passing = json["checkSecondSpecies"]["violations"]
            .as_array()
            .unwrap()
            .iter()
            .find(|v| v["rule"] == "s2-passing-dissonance")
            .expect("s2-passing-dissonance");
        assert_eq!(
            (passing["bar"].as_u64(), passing["beat"].as_u64()),
            (Some(1), Some(2))
        );
    }

    #[tokio::test]
    async fn second_species_bad_pitch_names_the_bar() {
        let res = run(r#"{ checkSecondSpecies(input: {
            key: "C major", placement: ABOVE,
            cantusFirmus: ["C4","C4"], counterpoint: [["C5","H4"],["C5"]]
        }) { valid } }"#)
        .await;
        assert!(
            res.errors[0]
                .message
                .starts_with("counterpoint[0][1]: invalid pitch 'H4'")
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
