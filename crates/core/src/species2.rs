//! Second species: two minims against each cantus firmus note.

use crate::interval::direction;
use crate::rules::{self, Line, Placement, Vertical};
use crate::{InputError, Interval, Key, Pitch, Position, Violation, Voice};

pub struct SecondSpecies {
    pub key: Key,
    pub placement: Placement,
    pub cantus_firmus: Vec<Pitch>,
    /// One list per bar, two minims or one semibreve, with None for the opening minim rest.
    pub counterpoint: Vec<Vec<Option<Pitch>>>,
}

impl SecondSpecies {
    /// Checks every rule that applies to second species, sorted by position.
    pub fn check(&self) -> Result<Vec<Violation>, InputError> {
        let cf = &self.cantus_firmus;
        if cf.len() < 2 {
            return Err(InputError(
                "The cantus firmus needs at least two notes.".into(),
            ));
        }
        if self.counterpoint.len() != cf.len() {
            return Err(InputError(format!(
                "Second species needs one bar of counterpoint per cantus firmus note: got {} against {}.",
                self.counterpoint.len(),
                cf.len()
            )));
        }

        let mut out = Vec::new();
        let last = cf.len() - 1;
        let mut verticals: Vec<Vertical> = Vec::new();
        let mut bars: Vec<Vec<Vertical>> = Vec::new();
        for (i, bar) in self.counterpoint.iter().enumerate() {
            if let Some(message) = rhythm_problem(i, last, bar) {
                out.push(Violation::error(
                    "s2-rhythm",
                    Position {
                        bar: i + 1,
                        beat: 1,
                    },
                    Some(Voice::Counterpoint),
                    message.to_string(),
                ));
            }
            let in_bar: Vec<Vertical> = bar
                .iter()
                .enumerate()
                .filter_map(|(j, &cp)| {
                    Some(Vertical {
                        cf: cf[i],
                        cp: cp?,
                        at: Position {
                            bar: i + 1,
                            beat: (j + 1) as u8,
                        },
                    })
                })
                .collect();
            verticals.extend(&in_bar);
            bars.push(in_bar);
        }
        if verticals.is_empty() {
            return Err(InputError("The counterpoint has no notes.".into()));
        }

        let cf_at: Vec<Position> = (1..=cf.len())
            .map(|bar| Position { bar, beat: 1 })
            .collect();
        let cf_line = Line {
            voice: Voice::CantusFirmus,
            notes: cf,
            at: &cf_at,
        };
        let (cp, cp_at): (Vec<Pitch>, Vec<Position>) =
            verticals.iter().map(|v| (v.cp, v.at)).unzip();
        let cp_line = Line {
            voice: Voice::Counterpoint,
            notes: &cp,
            at: &cp_at,
        };

        rules::cf_tonic(self.key, &cf_line, &mut out);
        rules::key_rules(self.key, &cf_line, &mut out);
        rules::key_rules(self.key, &cp_line, &mut out);
        rules::melodic_rules(&cp_line, &mut out);
        let after_rest = self.counterpoint[0].first() == Some(&None);
        rules::start_perfect(&verticals[0], self.placement, after_rest, &mut out);
        rules::end_perfect(verticals.last().unwrap(), &mut out);
        rules::cadence_step(&cf_line, &cp_line, &mut out);
        for v in &verticals {
            rules::voice_crossing(v, self.placement, &mut out);
        }
        self.beat_rules(&verticals, last, &mut out);
        self.motion_rules(&bars, &mut out);
        self.cadence(&bars[last - 1], last, &mut out);

        out.sort_by_key(|v| v.at);
        Ok(out)
    }

    /// Checks consonance, passing notes, unisons and repeats note by note.
    fn beat_rules(&self, verticals: &[Vertical], last: usize, out: &mut Vec<Violation>) {
        for (k, v) in verticals.iter().enumerate() {
            let iv = v.interval();
            let strong = v.at.beat == 1;
            if strong && !iv.is_consonant() {
                out.push(Violation::error(
                    "s2-strong-consonant",
                    v.at,
                    None,
                    format!(
                        "{iv} between {} and {} on the downbeat is dissonant.",
                        v.cf, v.cp
                    ),
                ));
            }
            if !strong && !iv.is_consonant() && !is_passing(verticals, k) {
                out.push(Violation::error(
                    "s2-passing-dissonance",
                    v.at,
                    None,
                    format!(
                        "{iv} between {} and {} is dissonant, so it must be a passing note.",
                        v.cf, v.cp
                    ),
                ));
            }
            if strong && iv.steps == 0 && v.at.bar != 1 && v.at.bar != last + 1 {
                out.push(Violation::error(
                    "s2-unison-strong",
                    v.at,
                    None,
                    format!(
                        "Unison on {} is only allowed on the downbeat of the first or last bar.",
                        v.cp
                    ),
                ));
            }
            if k > 0 && verticals[k - 1].cp == v.cp {
                out.push(Violation::error(
                    "s2-repeated-note",
                    v.at,
                    Some(Voice::Counterpoint),
                    format!("{} is repeated.", v.cp),
                ));
            }
        }
    }

    /// Checks parallel and direct motion between downbeats, and from the last note of a bar to the next downbeat.
    fn motion_rules(&self, bars: &[Vec<Vertical>], out: &mut Vec<Violation>) {
        for w in bars.windows(2) {
            let Some(next) = w[1].first().filter(|v| v.at.beat == 1) else {
                continue;
            };
            let down = w[0].first().filter(|v| v.at.beat == 1);
            let strong = down
                .and_then(|d| rules::into_perfect(d, next, self.placement))
                .filter(|m| m.parallel);
            if let Some(m) = strong {
                out.push(Violation::error(
                    "s2-parallels-strong",
                    next.at,
                    None,
                    format!(
                        "Parallel {} on consecutive downbeats: {} to {}.",
                        m.name, m.from, m.to
                    ),
                ));
            }

            // A semibreve's last note is its downbeat, whose parallels were checked above.
            let Some(prev) = w[0].last() else {
                continue;
            };
            let Some(m) = rules::into_perfect(prev, next, self.placement) else {
                continue;
            };
            let to = m.to;
            if m.parallel && prev.at.beat == 1 {
                continue;
            }
            if m.parallel {
                out.push(Violation::error(
                    "s2-parallels-weak-strong",
                    next.at,
                    None,
                    format!(
                        "Parallel {} from beat 2 to the downbeat: {} to {to}.",
                        m.name, m.from
                    ),
                ));
            } else if to.is_octave_class() {
                out.push(Violation::error(
                    "s2-direct-octave",
                    next.at,
                    None,
                    format!(
                        "Direct {}: both voices move the same way into a {to}.",
                        m.name
                    ),
                ));
            } else {
                out.push(Violation::warning(
                    "s2-direct-fifth",
                    next.at,
                    None,
                    format!("Direct 5ths: both voices move the same way into a {to}."),
                ));
            }
        }
    }

    /// Checks `s2-cadence` on the second last bar, numbered from 1.
    fn cadence(&self, bar: &[Vertical], number: usize, out: &mut Vec<Violation>) {
        let cf_leads = self.key.degree(self.cantus_firmus[number - 1]) == 7;
        let (into_final, name) = rules::into_final(self.placement, cf_leads);
        let needs_fifth = bar.len() == 2 && !cf_leads;
        let intervals: Vec<Interval> = bar.iter().map(Vertical::interval).collect();
        let ok = intervals
            .last()
            .is_some_and(|iv| (iv.simple_number(), iv.quality()) == into_final)
            && (!needs_fifth || intervals[0].is_fifth_class());
        if !ok {
            let actual: Vec<String> = intervals.iter().map(Interval::to_string).collect();
            let expected = if needs_fifth {
                format!("a 5th then {name}")
            } else {
                name.to_string()
            };
            out.push(Violation::warning(
                "s2-cadence",
                Position {
                    bar: number,
                    beat: 1,
                },
                None,
                format!(
                    "The second last bar is {}, should be {expected} (or compound).",
                    actual.join(" to ")
                ),
            ));
        }
    }
}

/// Returns what is wrong with the rhythm of a bar, by index, or None when it is fine.
fn rhythm_problem(i: usize, last: usize, bar: &[Option<Pitch>]) -> Option<&'static str> {
    let rests = bar.iter().filter(|p| p.is_none()).count();
    if i == last {
        return (bar.len() != 1 || rests > 0).then_some("The last bar should be one semibreve.");
    }
    if bar.len() != 2 && !(bar.len() == 1 && i + 1 == last) {
        return Some("Each bar should be two minims; only the second last may be a semibreve.");
    }
    let rest_ok = i == 0 && bar.len() == 2 && bar[0].is_none() && bar[1].is_some();
    (rests > 0 && !rest_ok).then_some("Only the first minim of the piece may be a rest.")
}

/// Returns true when the note at an index is reached and left by step in the same direction.
fn is_passing(verticals: &[Vertical], k: usize) -> bool {
    if k == 0 || k + 1 >= verticals.len() {
        return false;
    }
    let (a, b, c) = (verticals[k - 1].cp, verticals[k].cp, verticals[k + 1].cp);
    Interval::between(a, b).is_step()
        && Interval::between(b, c).is_step()
        && direction(a, b) != 0
        && direction(a, b) == direction(b, c)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Severity;

    fn line(s: &str) -> Vec<Pitch> {
        s.split_whitespace().map(|p| p.parse().unwrap()).collect()
    }

    /// Reads bars split by `|`, with `_` for a rest.
    fn bars(s: &str) -> Vec<Vec<Option<Pitch>>> {
        s.split('|')
            .map(|bar| {
                bar.split_whitespace()
                    .map(|p| (p != "_").then(|| p.parse().unwrap()))
                    .collect()
            })
            .collect()
    }

    fn exercise(key: &str, placement: Placement, cf: &str, cp: &str) -> SecondSpecies {
        SecondSpecies {
            key: key.parse().unwrap(),
            placement,
            cantus_firmus: line(cf),
            counterpoint: bars(cp),
        }
    }

    fn check(key: &str, placement: Placement, cf: &str, cp: &str) -> Vec<Violation> {
        exercise(key, placement, cf, cp).check().unwrap()
    }

    /// Returns the bar and beat of each violation of one rule.
    fn found(v: &[Violation], rule: &str) -> Vec<(usize, u8)> {
        v.iter()
            .filter(|v| v.rule == rule)
            .map(|v| (v.at.bar, v.at.beat))
            .collect()
    }

    const CF: &str = "C4 D4 F4 E4 G4 A4 F4 E4 D4 C4";

    #[test]
    fn valid_above_with_rest_has_no_violations() {
        let v = check(
            "C major",
            Placement::Above,
            CF,
            "_ G4 | F4 G4 | A4 B4 | C5 D5 | E5 D5 | C5 B4 | A4 B4 | C5 G4 | A4 B4 | C5",
        );
        assert!(v.is_empty(), "{v:#?}");
    }

    #[test]
    fn rejects_bar_count_mismatch() {
        let ex = exercise("C major", Placement::Above, "C4 D4 C4", "C5 G4 | B4");
        assert!(ex.check().is_err());
    }

    #[test]
    fn flags_wrong_rhythm() {
        let v = check(
            "C major",
            Placement::Above,
            "C4 D4 E4 C4",
            "C5 G4 | A4 | G4 B4 | C5 E5",
        );
        assert_eq!(found(&v, "s2-rhythm"), [(2, 1), (4, 1)], "{v:#?}");
    }

    #[test]
    fn second_last_bar_may_be_a_semibreve() {
        let v = check(
            "C major",
            Placement::Above,
            "C4 D4 E4 D4 C4",
            "C5 G4 | F4 A4 | G4 C5 | B4 | C5",
        );
        assert!(found(&v, "s2-rhythm").is_empty(), "{v:#?}");
        assert!(found(&v, "s2-repeated-note").is_empty(), "{v:#?}");
    }

    #[test]
    fn only_the_opening_minim_may_be_a_rest() {
        let v = check(
            "C major",
            Placement::Above,
            "C4 D4 E4 C4",
            "_ G4 | F4 _ | G4 B4 | C5",
        );
        assert_eq!(found(&v, "s2-rhythm"), [(2, 1)], "{v:#?}");
    }

    #[test]
    fn leading_rest_moves_the_start_to_beat_2() {
        let v = check("C major", Placement::Above, "C4 D4 C4", "_ A4 | F4 B4 | C5");
        assert_eq!(found(&v, "start-perfect"), [(1, 2)], "{v:#?}");
    }

    #[test]
    fn third_above_is_allowed_after_a_rest() {
        let rest = check("C major", Placement::Above, "C4 D4 C4", "_ E4 | F4 B4 | C5");
        assert!(found(&rest, "start-perfect").is_empty(), "{rest:#?}");
        let no_rest = check(
            "C major",
            Placement::Above,
            "C4 D4 C4",
            "E4 G4 | F4 B4 | C5",
        );
        assert_eq!(found(&no_rest, "start-perfect"), [(1, 1)], "{no_rest:#?}");
        let below = check("C major", Placement::Below, "C4 D4 C4", "_ A3 | G3 B3 | C4");
        assert_eq!(found(&below, "start-perfect"), [(1, 2)], "{below:#?}");
    }

    #[test]
    fn flags_dissonant_downbeat() {
        let v = check(
            "C major",
            Placement::Above,
            "C4 D4 C4",
            "C5 G4 | E4 G4 | C5",
        );
        assert_eq!(found(&v, "s2-strong-consonant"), [(2, 1)], "{v:#?}");
    }

    #[test]
    fn passing_dissonance_is_allowed() {
        let v = check(
            "C major",
            Placement::Above,
            "C4 E4 D4 C4",
            "C5 D5 | E5 C5 | A4 B4 | C5",
        );
        assert!(
            !found(&v, "s2-passing-dissonance").contains(&(1, 2)),
            "{v:#?}"
        );
    }

    #[test]
    fn flags_neighbour_and_leapt_dissonance() {
        let neighbour = check(
            "C major",
            Placement::Above,
            "C4 E4 D4 C4",
            "C5 D5 | C5 G4 | A4 B4 | C5",
        );
        assert!(
            found(&neighbour, "s2-passing-dissonance").contains(&(1, 2)),
            "{neighbour:#?}"
        );
        let leapt = check(
            "C major",
            Placement::Above,
            "C4 E4 D4 C4",
            "C5 F4 | G4 C5 | A4 B4 | C5",
        );
        assert!(
            found(&leapt, "s2-passing-dissonance").contains(&(1, 2)),
            "{leapt:#?}"
        );
    }

    #[test]
    fn flags_parallel_fifths_on_downbeats() {
        let v = check(
            "C major",
            Placement::Above,
            "C4 D4 C4",
            "G4 E4 | A4 F4 | C5",
        );
        assert_eq!(found(&v, "s2-parallels-strong"), [(2, 1)], "{v:#?}");
    }

    #[test]
    fn flags_parallel_octaves_from_beat_2() {
        let v = check(
            "C major",
            Placement::Above,
            "C4 D4 C4",
            "E5 C5 | D5 B4 | C5",
        );
        assert_eq!(found(&v, "s2-parallels-weak-strong"), [(2, 1)], "{v:#?}");
    }

    #[test]
    fn flags_direct_octave_by_step_as_error() {
        let v = check(
            "C major",
            Placement::Above,
            "C4 G4 D4 C4",
            "C5 G4 | B4 E5 | D5 B4 | C5",
        );
        let direct: Vec<_> = v.iter().filter(|v| v.rule == "s2-direct-octave").collect();
        assert_eq!(direct.len(), 1, "{v:#?}");
        assert_eq!(
            (direct[0].at.bar, direct[0].at.beat, direct[0].severity),
            (3, 1, Severity::Error)
        );
    }

    #[test]
    fn direct_fifth_from_beat_2_is_a_warning() {
        let v = check(
            "C major",
            Placement::Above,
            "C4 D4 C4",
            "C5 E4 | A4 B4 | C5",
        );
        let direct: Vec<_> = v.iter().filter(|v| v.rule == "s2-direct-fifth").collect();
        assert_eq!(direct.len(), 1, "{v:#?}");
        assert_eq!(
            (direct[0].at.bar, direct[0].at.beat, direct[0].severity),
            (2, 1, Severity::Warning)
        );
    }

    #[test]
    fn general_parallel_rules_are_replaced() {
        let v = check(
            "C major",
            Placement::Above,
            "C4 D4 C4",
            "C5 E4 | A4 B4 | C5",
        );
        assert!(found(&v, "parallel-perfect").is_empty(), "{v:#?}");
        assert!(found(&v, "direct-perfect").is_empty(), "{v:#?}");
    }

    #[test]
    fn unison_only_off_the_downbeat() {
        let v = check(
            "C major",
            Placement::Above,
            "C4 E4 D4 C4",
            "G4 C4 | E4 G4 | F4 B4 | C5",
        );
        assert_eq!(found(&v, "s2-unison-strong"), [(2, 1)], "{v:#?}");
    }

    #[test]
    fn flags_repeated_notes_in_and_across_bars() {
        let within = check(
            "C major",
            Placement::Above,
            "C4 D4 C4",
            "C5 C5 | A4 B4 | C5",
        );
        assert_eq!(found(&within, "s2-repeated-note"), [(1, 2)], "{within:#?}");
        let across = check(
            "C major",
            Placement::Above,
            "C4 E4 D4 C4",
            "E5 C5 | C5 G4 | A4 B4 | C5",
        );
        assert_eq!(found(&across, "s2-repeated-note"), [(2, 1)], "{across:#?}");
    }

    #[test]
    fn cadence_follows_the_cantus_leading_note() {
        // The end of a D minor assignment: the CF has C#-D, so the CP above ends on a m10.
        let v = check(
            "D minor",
            Placement::Above,
            "F3 E3 C#3 D3",
            "A4 G4 | G4 F4 | E4 | D4",
        );
        assert!(found(&v, "s2-cadence").is_empty(), "{v:#?}");
        let wrong = check(
            "D minor",
            Placement::Above,
            "F3 E3 C#3 D3",
            "A4 G4 | G4 F4 | C#4 | D4",
        );
        assert_eq!(found(&wrong, "s2-cadence"), [(3, 1)], "{wrong:#?}");
    }

    #[test]
    fn cadence_is_5_6_above_and_5_3_below() {
        let above = check(
            "C major",
            Placement::Above,
            "C4 D4 C4",
            "C5 G4 | A4 B4 | C5",
        );
        assert!(found(&above, "s2-cadence").is_empty(), "{above:#?}");
        let below = check(
            "C major",
            Placement::Below,
            "C4 D4 C4",
            "C3 A3 | G3 B3 | C4",
        );
        assert!(found(&below, "s2-cadence").is_empty(), "{below:#?}");
        let semibreve = check("C major", Placement::Above, "C4 D4 C4", "C5 G4 | B4 | C5");
        assert!(found(&semibreve, "s2-cadence").is_empty(), "{semibreve:#?}");
        let wrong = check(
            "C major",
            Placement::Above,
            "C4 D4 C4",
            "C5 G4 | F4 B4 | C5",
        );
        let cadence: Vec<_> = wrong.iter().filter(|v| v.rule == "s2-cadence").collect();
        assert_eq!(cadence.len(), 1, "{wrong:#?}");
        assert_eq!(
            (cadence[0].at.bar, cadence[0].severity),
            (2, Severity::Warning)
        );
    }
}
