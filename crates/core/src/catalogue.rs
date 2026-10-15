//! Plain descriptions of the rules for people using the checker.

use crate::Severity;

/// A rule as shown to users; `id` matches RULES.md and the violations.
#[derive(Clone, Copy, Debug)]
pub struct RuleInfo {
    pub id: &'static str,
    pub group: &'static str,
    pub severity: Severity,
    pub summary: &'static str,
}

const KEY: &str = "Key";
const MELODY: &str = "Counterpoint melody";
const VOICES: &str = "Between the voices";
const FIRST: &str = "First species";

const fn rule(
    id: &'static str,
    group: &'static str,
    severity: Severity,
    summary: &'static str,
) -> RuleInfo {
    RuleInfo {
        id,
        group,
        severity,
        summary,
    }
}

use Severity::{Error, Warning};

/// Every rule first species checks, in display order.
pub const FIRST_SPECIES_RULES: &[RuleInfo] = &[
    rule(
        "diatonic",
        KEY,
        Error,
        "Both voices use only notes of the key, plus the raised 6th and 7th in minor.",
    ),
    rule(
        "minor-6-7",
        KEY,
        Warning,
        "In minor, a raised 6th or 7th moving by step goes up, and a natural 6th or 7th moving by step goes down.",
    ),
    rule(
        "cadence-leading-note",
        KEY,
        Error,
        "A voice that ends 7 to 1 uses the leading note, a semitone below the tonic. In minor this is the raised 7th.",
    ),
    rule(
        "melodic-forbidden",
        MELODY,
        Error,
        "No melodic augmented 4ths, diminished 5ths, 7ths or 9ths.",
    ),
    rule(
        "melodic-augmented",
        MELODY,
        Error,
        "No other augmented or diminished melodic intervals.",
    ),
    rule(
        "melodic-over-octave",
        MELODY,
        Error,
        "No leap larger than an octave.",
    ),
    rule(
        "melodic-sixth",
        MELODY,
        Warning,
        "A melodic 6th is a minor 6th, rising or falling.",
    ),
    rule(
        "leap-recovery",
        MELODY,
        Warning,
        "A leap larger than a 4th is followed by a step the other way.",
    ),
    rule(
        "consecutive-leaps",
        MELODY,
        Warning,
        "At most two leaps in a row the same way, and two must outline a major or minor triad.",
    ),
    rule(
        "tritone-outline",
        MELODY,
        Warning,
        "The line does not outline a tritone between its high and low points.",
    ),
    rule(
        "single-climax",
        MELODY,
        Warning,
        "The highest note is used only once.",
    ),
    rule(
        "range",
        MELODY,
        Warning,
        "The counterpoint stays within a 10th.",
    ),
    rule(
        "start-perfect",
        VOICES,
        Error,
        "The first interval is a unison, 5th or octave with the counterpoint above, or a unison or octave with it below.",
    ),
    rule(
        "end-perfect",
        VOICES,
        Error,
        "The last interval is a unison or octave.",
    ),
    rule(
        "cadence-step",
        VOICES,
        Error,
        "Both voices reach the last note by step, in contrary motion.",
    ),
    rule(
        "parallel-perfect",
        VOICES,
        Error,
        "No parallel 5ths, octaves or unisons.",
    ),
    rule(
        "direct-perfect",
        VOICES,
        Error,
        "No direct 5ths or octaves, where both voices move the same way into a perfect interval. A warning instead of an error when the upper voice moves by step.",
    ),
    rule(
        "voice-crossing",
        VOICES,
        Warning,
        "The counterpoint does not cross the cantus firmus.",
    ),
    rule(
        "s1-consonant",
        FIRST,
        Error,
        "Every interval is a unison, 3rd, 5th, 6th or octave, or a compound of one. No 2nds, 4ths, 7ths, or augmented or diminished intervals.",
    ),
    rule(
        "s1-unison-middle",
        FIRST,
        Error,
        "A unison only on the first or last note.",
    ),
    rule(
        "s1-cadence",
        FIRST,
        Error,
        "The second last interval is a major 6th with the counterpoint above, or a minor 3rd with it below. Compounds count.",
    ),
    rule(
        "s1-imperfect-run",
        FIRST,
        Warning,
        "No more than three 3rds, or three 6ths, in a row.",
    ),
    rule(
        "s1-repeated-note",
        FIRST,
        Warning,
        "The counterpoint does not repeat a note, unless the two are tied.",
    ),
];

#[cfg(test)]
mod tests {
    use super::*;

    /// Returns the rule ids passed to `Violation::error` and `Violation::warning` in a source file.
    fn reported_ids(source: &str) -> Vec<&str> {
        let mut ids = Vec::new();
        for call in ["Violation::error(", "Violation::warning("] {
            for (start, _) in source.match_indices(call) {
                let rest = &source[start + call.len()..];
                if let Some(open) = rest.find('"') {
                    let id = &rest[open + 1..];
                    ids.push(&id[..id.find('"').unwrap()]);
                }
            }
        }
        ids
    }

    #[test]
    fn ids_are_unique() {
        for (i, a) in FIRST_SPECIES_RULES.iter().enumerate() {
            assert!(
                FIRST_SPECIES_RULES[i + 1..].iter().all(|b| b.id != a.id),
                "{}",
                a.id
            );
        }
    }

    #[test]
    fn every_rule_is_in_rules_md() {
        let spec = include_str!("../../../RULES.md");
        for r in FIRST_SPECIES_RULES {
            assert!(
                spec.contains(&format!("`{}`", r.id)),
                "{} is not in RULES.md",
                r.id
            );
        }
    }

    #[test]
    fn every_reported_rule_is_listed() {
        let sources = [include_str!("rules.rs"), include_str!("species1.rs")];
        let ids: Vec<&str> = sources.iter().flat_map(|s| reported_ids(s)).collect();
        assert!(ids.len() > 20, "found only {} rule ids", ids.len());
        for id in ids {
            assert!(
                FIRST_SPECIES_RULES.iter().any(|r| r.id == id),
                "{id} is reported but not listed"
            );
        }
    }
}
