//! First species: one counterpoint note against each cantus firmus note.

use crate::rules::{self, Line, Placement, Vertical};
use crate::{InputError, Key, Pitch, Position, Quality, Violation, Voice};

/// A first species exercise.
pub struct FirstSpecies {
    pub key: Key,
    pub placement: Placement,
    pub cantus_firmus: Vec<Pitch>,
    pub counterpoint: Vec<Pitch>,
}

impl FirstSpecies {
    /// Checks every rule that applies to first species, sorted by position.
    pub fn check(&self) -> Result<Vec<Violation>, InputError> {
        let (cf, cp) = (&self.cantus_firmus, &self.counterpoint);
        if cf.len() < 2 {
            return Err(InputError(
                "The cantus firmus needs at least two notes.".into(),
            ));
        }
        if cp.len() != cf.len() {
            return Err(InputError(format!(
                "First species needs one counterpoint note per cantus firmus note: got {} against {}.",
                cp.len(),
                cf.len()
            )));
        }

        let at: Vec<Position> = (1..=cf.len())
            .map(|bar| Position { bar, beat: 1 })
            .collect();
        let cf_line = Line {
            voice: Voice::CantusFirmus,
            notes: cf,
            at: &at,
        };
        let cp_line = Line {
            voice: Voice::Counterpoint,
            notes: cp,
            at: &at,
        };
        let verticals: Vec<Vertical> = cf
            .iter()
            .zip(cp)
            .zip(&at)
            .map(|((&cf, &cp), &at)| Vertical { cf, cp, at })
            .collect();

        let mut out = Vec::new();
        rules::key_rules(self.key, &cf_line, &mut out);
        rules::key_rules(self.key, &cp_line, &mut out);
        rules::melodic_rules(&cp_line, &mut out);
        rules::start_perfect(&verticals[0], self.placement, &mut out);
        rules::end_perfect(verticals.last().unwrap(), &mut out);
        rules::cadence_step(&cf_line, &cp_line, &mut out);
        for v in &verticals {
            rules::spacing(v, self.placement, &mut out);
        }
        for w in verticals.windows(2) {
            rules::parallels(&w[0], &w[1], self.placement, &mut out);
        }
        self.species_rules(&verticals, &mut out);

        out.sort_by_key(|v| v.at);
        Ok(out)
    }

    /// Checks the rules in the 1st species table of RULES.md.
    fn species_rules(&self, verticals: &[Vertical], out: &mut Vec<Violation>) {
        let last = verticals.len() - 1;
        let mut run = 0;
        let mut run_number = 0;

        for (i, v) in verticals.iter().enumerate() {
            let iv = v.interval();
            if !iv.is_consonant() {
                out.push(Violation::error(
                    "s1-consonant",
                    v.at,
                    None,
                    format!("{iv} between {} and {} is dissonant.", v.cf, v.cp),
                ));
            }
            if iv.steps == 0 && i != 0 && i != last {
                out.push(Violation::error(
                    "s1-unison-middle",
                    v.at,
                    None,
                    format!(
                        "Unison on {} is only allowed on the first or last note.",
                        v.cp
                    ),
                ));
            }

            let number = iv.simple_number();
            if matches!(number, 3 | 6) && iv.is_consonant() {
                run = if number == run_number { run + 1 } else { 1 };
                run_number = number;
                if run == 4 {
                    let name = if number == 3 { "3rds" } else { "6ths" };
                    out.push(Violation::warning(
                        "s1-imperfect-run",
                        v.at,
                        None,
                        format!("More than three {name} in a row."),
                    ));
                }
            } else {
                run = 0;
                run_number = 0;
            }

            if i > 0 && v.cp == verticals[i - 1].cp {
                out.push(Violation::warning(
                    "s1-repeated-note",
                    v.at,
                    Some(Voice::Counterpoint),
                    format!("{} is repeated.", v.cp),
                ));
            }
        }

        let pen = &verticals[last - 1];
        let iv = pen.interval();
        let (number, quality, expected) = match self.placement {
            Placement::Above => (6, Quality::Major, "a major 6th"),
            Placement::Below => (3, Quality::Minor, "a minor 3rd"),
        };
        if iv.simple_number() != number || iv.quality() != quality {
            out.push(Violation::error(
                "s1-cadence",
                pen.at,
                None,
                format!("The second last interval is a {iv}, should be {expected} (or compound)."),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(s: &str) -> Vec<Pitch> {
        s.split_whitespace().map(|p| p.parse().unwrap()).collect()
    }

    fn check(key: &str, placement: Placement, cf: &str, cp: &str) -> Vec<Violation> {
        FirstSpecies {
            key: key.parse().unwrap(),
            placement,
            cantus_firmus: line(cf),
            counterpoint: line(cp),
        }
        .check()
        .unwrap()
    }

    fn rules(v: &[Violation]) -> Vec<&'static str> {
        v.iter().map(|v| v.rule).collect()
    }

    const CF: &str = "C4 D4 F4 E4 G4 A4 F4 E4 D4 C4";

    #[test]
    fn valid_above_has_no_violations() {
        let v = check(
            "C major",
            Placement::Above,
            CF,
            "C5 B4 A4 C5 B4 C5 D5 C5 B4 C5",
        );
        assert!(v.is_empty(), "{v:#?}");
    }

    #[test]
    fn valid_below_has_no_violations() {
        let v = check(
            "C major",
            Placement::Below,
            CF,
            "C4 B3 A3 C4 B3 C4 D4 C4 B3 C4",
        );
        assert!(v.is_empty(), "{v:#?}");
    }

    #[test]
    fn valid_minor_has_no_violations() {
        let v = check(
            "A minor",
            Placement::Above,
            "A3 C4 B3 D4 C4 B3 A3",
            "A4 E5 D5 B4 A4 G#4 A4",
        );
        assert!(v.is_empty(), "{v:#?}");
    }

    #[test]
    fn rejects_mismatched_lengths() {
        let ex = FirstSpecies {
            key: "C major".parse().unwrap(),
            placement: Placement::Above,
            cantus_firmus: line("C4 D4 C4"),
            counterpoint: line("C5 B4"),
        };
        assert!(ex.check().is_err());
    }

    #[test]
    fn flags_dissonance() {
        let v = check(
            "C major",
            Placement::Above,
            CF,
            "C5 B4 A4 C5 B4 C5 D5 D5 B4 C5",
        );
        assert!(rules(&v).contains(&"s1-consonant"), "{v:#?}");
    }

    #[test]
    fn flags_parallel_fifths() {
        let v = check(
            "C major",
            Placement::Above,
            "C4 D4 E4 D4 C4",
            "G4 A4 G4 B4 C5",
        );
        assert_eq!(
            v.iter()
                .find(|v| v.rule == "parallel-perfect")
                .map(|v| v.at.bar),
            Some(2),
            "{v:#?}"
        );
    }

    #[test]
    fn flags_direct_octave_by_leap_as_error() {
        let v = check("C major", Placement::Above, "C4 E4 D4 C4", "G4 E5 B4 C5");
        let direct = v
            .iter()
            .find(|v| v.rule == "direct-perfect")
            .expect("direct-perfect");
        assert_eq!(
            (direct.at.bar, direct.severity),
            (2, crate::Severity::Error)
        );
    }

    #[test]
    fn flags_direct_fifth_by_step_as_warning() {
        let v = check("C major", Placement::Above, "C4 A3 C4", "E4 F4 G4");
        let direct = v
            .iter()
            .find(|v| v.rule == "direct-perfect")
            .expect("direct-perfect");
        assert_eq!(
            (direct.at.bar, direct.severity),
            (3, crate::Severity::Warning),
            "{v:#?}"
        );
    }

    #[test]
    fn oblique_motion_is_not_direct() {
        let v = check(
            "C major",
            Placement::Above,
            "C4 E4 F4 D4 C4",
            "C5 C5 C5 B4 C5",
        );
        assert!(!rules(&v).contains(&"direct-perfect"), "{v:#?}");
    }

    #[test]
    fn flags_bad_start_and_end() {
        let v = check(
            "C major",
            Placement::Above,
            CF,
            "E5 B4 A4 C5 B4 C5 D5 C5 B4 E5",
        );
        assert!(rules(&v).contains(&"start-perfect"));
        assert!(rules(&v).contains(&"end-perfect"));
    }

    #[test]
    fn fifth_start_only_above() {
        let above = check(
            "C major",
            Placement::Above,
            CF,
            "G4 B4 A4 C5 B4 C5 D5 C5 B4 C5",
        );
        assert!(!rules(&above).contains(&"start-perfect"), "{above:#?}");
        let below = check(
            "C major",
            Placement::Below,
            CF,
            "F3 B2 D3 C3 E3 F3 D3 C3 B2 C3",
        );
        assert!(rules(&below).contains(&"start-perfect"));
    }

    #[test]
    fn flags_unraised_leading_note_in_minor() {
        let v = check(
            "A minor",
            Placement::Above,
            "A3 C4 B3 D4 C4 B3 A3",
            "A4 G4 G4 F4 E4 G4 A4",
        );
        assert!(rules(&v).contains(&"cadence-leading-note"));
        assert!(rules(&v).contains(&"s1-cadence"));
    }

    #[test]
    fn flags_note_outside_key() {
        let v = check(
            "C major",
            Placement::Above,
            CF,
            "C5 B4 A4 C#5 B4 C5 D5 C5 B4 C5",
        );
        assert!(rules(&v).contains(&"diatonic"));
    }

    #[test]
    fn flags_melodic_tritone() {
        let v = check(
            "C major",
            Placement::Above,
            "C4 D4 F4 D4 C4",
            "C5 F4 B4 B4 C5",
        );
        assert!(rules(&v).contains(&"melodic-forbidden"), "{v:#?}");
    }

    #[test]
    fn flags_unison_in_middle() {
        let v = check(
            "C major",
            Placement::Above,
            "C4 D4 E4 D4 C4",
            "C5 F4 E4 B4 C5",
        );
        assert!(rules(&v).contains(&"s1-unison-middle"));
    }

    #[test]
    fn flags_run_of_thirds() {
        let v = check(
            "C major",
            Placement::Above,
            "C4 D4 E4 F4 G4 F4 E4 D4 C4",
            "C5 F4 G4 A4 B4 A4 G4 B4 C5",
        );
        assert!(rules(&v).contains(&"s1-imperfect-run"), "{v:#?}");
    }

    #[test]
    fn flags_unrecovered_leap() {
        let v = check(
            "C major",
            Placement::Above,
            CF,
            "C5 F4 A4 G4 B4 C5 D5 C5 B4 C5",
        );
        assert!(rules(&v).contains(&"leap-recovery"), "{v:#?}");
    }
}
