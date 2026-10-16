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
const SECOND: &str = "Second species";

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

const DIATONIC: RuleInfo = rule(
    "diatonic",
    KEY,
    Error,
    "Both voices use only notes of the key, plus the raised 6th and 7th in minor.",
);
const CF_TONIC: RuleInfo = rule(
    "cf-tonic",
    KEY,
    Warning,
    "The cantus firmus starts and ends on the tonic. If it does not, the key is probably wrong, for example G major for an E minor exercise.",
);
const MINOR_6_7: RuleInfo = rule(
    "minor-6-7",
    KEY,
    Warning,
    "In minor, a natural 7th does not step up to the tonic, and a raised 6th does not step down to the 5th.",
);
const CADENCE_LEADING_NOTE: RuleInfo = rule(
    "cadence-leading-note",
    KEY,
    Error,
    "A voice that ends 7 to 1 uses the leading note, a semitone below the tonic. In minor this is the raised 7th.",
);
const MELODIC_FORBIDDEN: RuleInfo = rule(
    "melodic-forbidden",
    MELODY,
    Error,
    "No melodic augmented 4ths, diminished 5ths, 7ths or 9ths.",
);
const MELODIC_AUGMENTED: RuleInfo = rule(
    "melodic-augmented",
    MELODY,
    Error,
    "No other augmented or diminished melodic intervals.",
);
const MELODIC_OVER_OCTAVE: RuleInfo = rule(
    "melodic-over-octave",
    MELODY,
    Error,
    "No leap larger than an octave.",
);
const MELODIC_SIXTH: RuleInfo = rule(
    "melodic-sixth",
    MELODY,
    Warning,
    "A melodic 6th is a minor 6th, rising or falling.",
);
const LEAP_RECOVERY: RuleInfo = rule(
    "leap-recovery",
    MELODY,
    Warning,
    "A leap larger than a 4th is followed by a step the other way.",
);
const CONSECUTIVE_LEAPS: RuleInfo = rule(
    "consecutive-leaps",
    MELODY,
    Warning,
    "At most two leaps in a row the same way, and two must outline a major or minor triad.",
);
const TRITONE_OUTLINE: RuleInfo = rule(
    "tritone-outline",
    MELODY,
    Warning,
    "The line does not outline a tritone between its high and low points.",
);
const SINGLE_CLIMAX: RuleInfo = rule(
    "single-climax",
    MELODY,
    Warning,
    "The highest note is used only once.",
);
const RANGE: RuleInfo = rule(
    "range",
    MELODY,
    Warning,
    "The counterpoint stays within a 10th.",
);
const START_PERFECT: RuleInfo = rule(
    "start-perfect",
    VOICES,
    Error,
    "The first interval is a unison, 5th or octave with the counterpoint above, or a unison or octave with it below. After an opening rest, a 3rd above is fine too.",
);
const END_PERFECT: RuleInfo = rule(
    "end-perfect",
    VOICES,
    Error,
    "The last interval is a unison or octave.",
);
const CADENCE_STEP: RuleInfo = rule(
    "cadence-step",
    VOICES,
    Error,
    "Both voices reach the last note by step, in contrary motion.",
);
const PARALLEL_PERFECT: RuleInfo = rule(
    "parallel-perfect",
    VOICES,
    Error,
    "No parallel 5ths, octaves or unisons.",
);
const DIRECT_PERFECT: RuleInfo = rule(
    "direct-perfect",
    VOICES,
    Error,
    "No direct 5ths or octaves, where both voices move the same way into a perfect interval. A warning instead of an error when the upper voice moves by step.",
);
const VOICE_CROSSING: RuleInfo = rule(
    "voice-crossing",
    VOICES,
    Warning,
    "The counterpoint does not cross the cantus firmus.",
);

const S1_CONSONANT: RuleInfo = rule(
    "s1-consonant",
    FIRST,
    Error,
    "Every interval is a unison, 3rd, 5th, 6th or octave, or a compound of one. No 2nds, 4ths, 7ths, or augmented or diminished intervals.",
);
const S1_UNISON_MIDDLE: RuleInfo = rule(
    "s1-unison-middle",
    FIRST,
    Error,
    "A unison only on the first or last note.",
);
const S1_CADENCE: RuleInfo = rule(
    "s1-cadence",
    FIRST,
    Error,
    "The second last interval is a major 6th when the upper voice has the leading note, or a minor 3rd when the lower voice has it. Compounds count.",
);
const S1_IMPERFECT_RUN: RuleInfo = rule(
    "s1-imperfect-run",
    FIRST,
    Warning,
    "No more than three 3rds, or three 6ths, in a row.",
);
const S1_REPEATED_NOTE: RuleInfo = rule(
    "s1-repeated-note",
    FIRST,
    Warning,
    "The counterpoint does not repeat a note, unless the two are tied.",
);

const S2_RHYTHM: RuleInfo = rule(
    "s2-rhythm",
    SECOND,
    Error,
    "Two minims a bar. The second last bar may be a semibreve and the last bar is one. The first bar may start with a minim rest.",
);
const S2_STRONG_CONSONANT: RuleInfo = rule(
    "s2-strong-consonant",
    SECOND,
    Error,
    "Every downbeat is a unison, 3rd, 5th, 6th or octave, or a compound of one.",
);
const S2_PASSING_DISSONANCE: RuleInfo = rule(
    "s2-passing-dissonance",
    SECOND,
    Error,
    "A dissonant second minim is a passing note: reached and left by step, the same way.",
);
const S2_PARALLELS_STRONG: RuleInfo = rule(
    "s2-parallels-strong",
    SECOND,
    Error,
    "No parallel 5ths, octaves or unisons on consecutive downbeats.",
);
const S2_PARALLELS_WEAK_STRONG: RuleInfo = rule(
    "s2-parallels-weak-strong",
    SECOND,
    Error,
    "No parallel 5ths, octaves or unisons from the second minim to the next downbeat.",
);
const S2_DIRECT_OCTAVE: RuleInfo = rule(
    "s2-direct-octave",
    SECOND,
    Error,
    "No direct octaves or unisons into a downbeat, even when the upper voice steps.",
);
const S2_DIRECT_FIFTH: RuleInfo = rule(
    "s2-direct-fifth",
    SECOND,
    Warning,
    "Avoid direct 5ths into a downbeat.",
);
const S2_UNISON_STRONG: RuleInfo = rule(
    "s2-unison-strong",
    SECOND,
    Error,
    "No unison on a downbeat except in the first and last bar.",
);
const S2_REPEATED_NOTE: RuleInfo = rule(
    "s2-repeated-note",
    SECOND,
    Error,
    "The counterpoint does not repeat a note.",
);
const S2_CADENCE: RuleInfo = rule(
    "s2-cadence",
    SECOND,
    Warning,
    "The second last bar reaches the final octave from a major 6th when the upper voice has the leading note, or a minor 3rd when the lower voice has it. When the counterpoint has the leading note in two minims, the first is a 5th. Compounds count.",
);

/// Every rule first species checks, in display order.
pub const FIRST_SPECIES_RULES: &[RuleInfo] = &[
    CF_TONIC,
    DIATONIC,
    MINOR_6_7,
    CADENCE_LEADING_NOTE,
    MELODIC_FORBIDDEN,
    MELODIC_AUGMENTED,
    MELODIC_OVER_OCTAVE,
    MELODIC_SIXTH,
    LEAP_RECOVERY,
    CONSECUTIVE_LEAPS,
    TRITONE_OUTLINE,
    SINGLE_CLIMAX,
    RANGE,
    START_PERFECT,
    END_PERFECT,
    CADENCE_STEP,
    PARALLEL_PERFECT,
    DIRECT_PERFECT,
    VOICE_CROSSING,
    S1_CONSONANT,
    S1_UNISON_MIDDLE,
    S1_CADENCE,
    S1_IMPERFECT_RUN,
    S1_REPEATED_NOTE,
];

/// Every rule second species checks, in display order.
pub const SECOND_SPECIES_RULES: &[RuleInfo] = &[
    CF_TONIC,
    DIATONIC,
    MINOR_6_7,
    CADENCE_LEADING_NOTE,
    MELODIC_FORBIDDEN,
    MELODIC_AUGMENTED,
    MELODIC_OVER_OCTAVE,
    MELODIC_SIXTH,
    LEAP_RECOVERY,
    CONSECUTIVE_LEAPS,
    TRITONE_OUTLINE,
    SINGLE_CLIMAX,
    RANGE,
    START_PERFECT,
    END_PERFECT,
    CADENCE_STEP,
    VOICE_CROSSING,
    S2_RHYTHM,
    S2_STRONG_CONSONANT,
    S2_PASSING_DISSONANCE,
    S2_PARALLELS_STRONG,
    S2_PARALLELS_WEAK_STRONG,
    S2_DIRECT_OCTAVE,
    S2_DIRECT_FIFTH,
    S2_UNISON_STRONG,
    S2_REPEATED_NOTE,
    S2_CADENCE,
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
        for list in [FIRST_SPECIES_RULES, SECOND_SPECIES_RULES] {
            for (i, a) in list.iter().enumerate() {
                assert!(list[i + 1..].iter().all(|b| b.id != a.id), "{}", a.id);
            }
        }
    }

    #[test]
    fn every_rule_is_in_rules_md() {
        let spec = include_str!("../../../RULES.md");
        for r in FIRST_SPECIES_RULES.iter().chain(SECOND_SPECIES_RULES) {
            assert!(
                spec.contains(&format!("`{}`", r.id)),
                "{} is not in RULES.md",
                r.id
            );
        }
    }

    /// Fails when a source reports a rule id that is neither in the list nor skipped.
    fn assert_listed(sources: &[&str], list: &[RuleInfo], skipped: &[&str]) {
        let ids: Vec<&str> = sources.iter().flat_map(|s| reported_ids(s)).collect();
        assert!(ids.len() > 10, "found only {} rule ids", ids.len());
        for id in ids.into_iter().filter(|id| !skipped.contains(id)) {
            assert!(
                list.iter().any(|r| r.id == id),
                "{id} is reported but not listed"
            );
        }
    }

    #[test]
    fn every_reported_rule_is_listed() {
        let general = include_str!("rules.rs");
        assert_listed(
            &[general, include_str!("species1.rs")],
            FIRST_SPECIES_RULES,
            &[],
        );
        // Second species does not call rules::parallels().
        assert_listed(
            &[general, include_str!("species2.rs")],
            SECOND_SPECIES_RULES,
            &["parallel-perfect", "direct-perfect"],
        );
    }
}
