//! Rules from the "All species" table in RULES.md.

use crate::interval::direction;
use crate::{Interval, Key, Pitch, Position, Quality, Violation, Voice};

/// Whether the counterpoint sits above or below the cantus firmus.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Placement {
    Above,
    Below,
}

/// One voice's notes, each with the position it starts at.
pub struct Line<'a> {
    pub voice: Voice,
    pub notes: &'a [Pitch],
    pub at: &'a [Position],
}

/// A cantus firmus note and the counterpoint note sounding with it.
#[derive(Clone, Copy, Debug)]
pub struct Vertical {
    pub cf: Pitch,
    pub cp: Pitch,
    pub at: Position,
}

impl Vertical {
    /// Returns the interval between the two voices.
    pub fn interval(&self) -> Interval {
        Interval::between(self.cf, self.cp)
    }
}

/// Checks `diatonic`, `minor-6-7` and `cadence-leading-note` on one line.
pub fn key_rules(key: Key, line: &Line, out: &mut Vec<Violation>) {
    let voice = Some(line.voice);
    for (i, &p) in line.notes.iter().enumerate() {
        if !key.is_diatonic(p) {
            out.push(Violation::error(
                "diatonic",
                line.at[i],
                voice,
                format!("{p} is not in {key}."),
            ));
        }
    }

    for (i, w) in line.notes.windows(2).enumerate() {
        let (from, to) = (w[0], w[1]);
        if !Interval::between(from, to).is_step() {
            continue;
        }
        let dir = direction(from, to);
        if key.is_raised(from) && dir < 0 {
            out.push(Violation::warning(
                "minor-6-7",
                line.at[i],
                voice,
                format!("Raised {from} should rise, not fall to {to}."),
            ));
        } else if key.is_natural_minor_6_7(from) && dir > 0 {
            out.push(Violation::warning(
                "minor-6-7",
                line.at[i],
                voice,
                format!("Natural {from} should fall, not rise to {to}. Raise it to rise."),
            ));
        }
    }

    if let [.., pen, last] = line.notes {
        let rising_7_8 =
            key.degree(*pen) == 7 && key.degree(*last) == 1 && direction(*pen, *last) > 0;
        if rising_7_8 && last.semitones() - pen.semitones() != 1 {
            out.push(Violation::error(
                "cadence-leading-note",
                line.at[line.notes.len() - 2],
                voice,
                format!("{pen} should be the leading note, a semitone below {last}."),
            ));
        }
    }
}

/// Checks the melodic rules that apply to the counterpoint line.
pub fn melodic_rules(line: &Line, out: &mut Vec<Violation>) {
    let voice = Some(line.voice);
    let notes = line.notes;
    let moves: Vec<Interval> = notes
        .windows(2)
        .map(|w| Interval::between(w[0], w[1]))
        .collect();
    let dirs: Vec<i32> = notes.windows(2).map(|w| direction(w[0], w[1])).collect();

    for (i, m) in moves.iter().enumerate() {
        let (from, to, at) = (notes[i], notes[i + 1], line.at[i + 1]);
        let tritone = matches!(
            (m.steps, m.quality()),
            (3, Quality::Augmented(1)) | (4, Quality::Diminished(1))
        );
        if tritone || m.steps == 6 || m.steps == 8 {
            out.push(Violation::error(
                "melodic-forbidden",
                at,
                voice,
                format!("Melodic {m} from {from} to {to}."),
            ));
        } else if m.steps > 7 || (m.steps == 7 && m.semitones > 12) {
            out.push(Violation::error(
                "melodic-over-octave",
                at,
                voice,
                format!("Leap of a {m} from {from} to {to} is larger than an octave."),
            ));
        } else if m.is_altered() {
            out.push(Violation::error(
                "melodic-augmented",
                at,
                voice,
                format!("Melodic {m} from {from} to {to}."),
            ));
        } else if m.steps == 5 && !(dirs[i] > 0 && m.quality() == Quality::Minor) {
            out.push(Violation::warning(
                "melodic-sixth",
                at,
                voice,
                format!("Melodic {m} from {from} to {to}. Only a rising minor 6th is allowed."),
            ));
        }

        if m.steps >= 4 {
            let recovered = moves
                .get(i + 1)
                .is_none_or(|next| next.is_step() && dirs[i + 1] == -dirs[i]);
            if !recovered {
                out.push(Violation::warning(
                    "leap-recovery",
                    line.at[i + 1],
                    voice,
                    format!("Leap of a {m} to {to} should be followed by a step the other way."),
                ));
            }
        }
    }

    leap_runs(line, &moves, &dirs, out);
    tritone_outline(line, &dirs, out);

    if let Some(top) = notes.iter().map(|p| p.semitones()).max() {
        if let Some((i, p)) = notes
            .iter()
            .enumerate()
            .filter(|(_, p)| p.semitones() == top)
            .nth(1)
        {
            out.push(Violation::warning(
                "single-climax",
                line.at[i],
                voice,
                format!("The highest note {p} appears more than once."),
            ));
        }
        let low = notes.iter().min_by_key(|p| p.semitones()).unwrap();
        let high = notes.iter().max_by_key(|p| p.semitones()).unwrap();
        let range = Interval::between(*low, *high);
        if range.steps > 9 {
            out.push(Violation::warning(
                "range",
                line.at[0],
                voice,
                format!("Range {low} to {high} is a {range}, more than a 10th."),
            ));
        }
    }
}

/// Checks `consecutive-leaps`: at most two leaps one way, and two must outline a triad.
fn leap_runs(line: &Line, moves: &[Interval], dirs: &[i32], out: &mut Vec<Violation>) {
    let notes = line.notes;
    let same_way_leaps =
        |i: usize| moves[i].is_leap() && moves[i + 1].is_leap() && dirs[i] == dirs[i + 1];
    for i in 0..moves.len().saturating_sub(1) {
        if !same_way_leaps(i) || (i > 0 && same_way_leaps(i - 1)) {
            continue;
        }
        if i + 2 < moves.len() && same_way_leaps(i + 1) {
            out.push(Violation::warning(
                "consecutive-leaps",
                line.at[i + 3],
                Some(line.voice),
                format!(
                    "Three leaps in a row the same way, {} to {}.",
                    notes[i],
                    notes[i + 3]
                ),
            ));
        } else if !is_triad(&notes[i..i + 3]) {
            out.push(Violation::warning(
                "consecutive-leaps",
                line.at[i + 2],
                Some(line.voice),
                format!(
                    "Leaps {}, {}, {} do not outline a major or minor triad.",
                    notes[i],
                    notes[i + 1],
                    notes[i + 2]
                ),
            ));
        }
    }
}

/// Returns true when three pitches form a major or minor triad in any position.
fn is_triad(notes: &[Pitch]) -> bool {
    let pcs: Vec<i32> = notes.iter().map(|p| p.semitones().rem_euclid(12)).collect();
    let has = |pc: i32| pcs.contains(&pc.rem_euclid(12));
    pcs.iter()
        .any(|&r| has(r + 7) && (has(r + 3) || has(r + 4)))
}

/// Checks `tritone-outline` between each pair of turning points in the line.
fn tritone_outline(line: &Line, dirs: &[i32], out: &mut Vec<Violation>) {
    let mut turns = vec![0];
    let mut last_dir = 0;
    for (i, &d) in dirs.iter().enumerate() {
        if d == 0 {
            continue;
        }
        if last_dir != 0 && d != last_dir {
            turns.push(i);
        }
        last_dir = d;
    }
    turns.push(line.notes.len() - 1);

    for w in turns.windows(2) {
        let (a, b) = (w[0], w[1]);
        let span = Interval::between(line.notes[a], line.notes[b]);
        let tritone = matches!(
            (span.steps % 7, span.quality()),
            (3, Quality::Augmented(1)) | (4, Quality::Diminished(1))
        );
        if tritone && b - a > 1 {
            out.push(Violation::warning(
                "tritone-outline",
                line.at[b],
                Some(line.voice),
                format!(
                    "The line from {} to {} outlines a {span}.",
                    line.notes[a], line.notes[b]
                ),
            ));
        }
    }
}

/// Checks `start-perfect` on the first vertical.
pub fn start_perfect(first: &Vertical, placement: Placement, out: &mut Vec<Violation>) {
    let iv = first.interval();
    let ok = iv.is_octave_class() || (placement == Placement::Above && iv.is_fifth_class());
    if !ok {
        let allowed = match placement {
            Placement::Above => "a unison, 5th or octave",
            Placement::Below => "a unison or octave",
        };
        out.push(Violation::error(
            "start-perfect",
            first.at,
            None,
            format!("Starts on a {iv}, should be {allowed}."),
        ));
    }
}

/// Checks `end-perfect` on the last vertical.
pub fn end_perfect(last: &Vertical, out: &mut Vec<Violation>) {
    let iv = last.interval();
    if !iv.is_octave_class() {
        out.push(Violation::error(
            "end-perfect",
            last.at,
            None,
            format!("Ends on a {iv}, should be a unison or octave."),
        ));
    }
}

/// Checks `cadence-step`: both voices reach the last note by step in contrary motion.
pub fn cadence_step(cf: &Line, cp: &Line, out: &mut Vec<Violation>) {
    if cf.notes.len() < 2 || cp.notes.len() < 2 {
        return;
    }
    let (cf_last, cp_last) = (cf.notes.len() - 1, cp.notes.len() - 1);
    let (a, b) = (cf.notes[cf_last - 1], cf.notes[cf_last]);
    let (c, d) = (cp.notes[cp_last - 1], cp.notes[cp_last]);
    let steps = Interval::between(a, b).is_step() && Interval::between(c, d).is_step();
    if !steps || direction(a, b) != -direction(c, d) {
        out.push(Violation::error(
            "cadence-step",
            cp.at[cp_last],
            None,
            "Both voices should reach the last note by step, in contrary motion.".to_string(),
        ));
    }
}

/// Checks `spacing` and `voice-crossing` on one vertical.
pub fn spacing(v: &Vertical, placement: Placement, out: &mut Vec<Violation>) {
    let iv = v.interval();
    if iv.steps > 9 {
        out.push(Violation::warning(
            "spacing",
            v.at,
            None,
            format!("Voices are a {iv} apart, more than a 10th."),
        ));
    }
    let crossed = match placement {
        Placement::Above => v.cp.semitones() < v.cf.semitones(),
        Placement::Below => v.cp.semitones() > v.cf.semitones(),
    };
    if crossed {
        out.push(Violation::warning(
            "voice-crossing",
            v.at,
            Some(Voice::Counterpoint),
            format!("{} crosses the cantus firmus note {}.", v.cp, v.cf),
        ));
    }
}

/// Checks `parallel-perfect` and `direct-perfect` from one vertical to the next.
pub fn parallels(prev: &Vertical, next: &Vertical, placement: Placement, out: &mut Vec<Violation>) {
    let cf_dir = direction(prev.cf, next.cf);
    let cp_dir = direction(prev.cp, next.cp);
    let (from, to) = (prev.interval(), next.interval());
    if cf_dir == 0 || cf_dir != cp_dir || !(to.is_octave_class() || to.is_fifth_class()) {
        return;
    }

    let name = if to.is_fifth_class() {
        "5ths"
    } else if to.steps == 0 {
        "unisons"
    } else {
        "octaves"
    };
    let same_class = (from.is_octave_class() && to.is_octave_class())
        || (from.is_fifth_class() && to.is_fifth_class());
    if same_class {
        out.push(Violation::error(
            "parallel-perfect",
            next.at,
            None,
            format!("Parallel {name}: {from} to {to}."),
        ));
        return;
    }

    let (upper_from, upper_to) = match placement {
        Placement::Above => (prev.cp, next.cp),
        Placement::Below => (prev.cf, next.cf),
    };
    let message = format!("Direct {name}: both voices move the same way into a {to}.");
    if Interval::between(upper_from, upper_to).is_step() {
        out.push(Violation::warning("direct-perfect", next.at, None, message));
    } else {
        out.push(Violation::error("direct-perfect", next.at, None, message));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spots_triads() {
        let t = |s: &str| {
            is_triad(
                &s.split(' ')
                    .map(|p| p.parse().unwrap())
                    .collect::<Vec<Pitch>>(),
            )
        };
        assert!(t("C4 E4 G4"));
        assert!(t("A3 C4 E4"));
        assert!(t("E4 C4 G3"));
        assert!(t("C4 E4 A4"));
        assert!(t("C4 F4 A4"));
        assert!(!t("C4 E4 G#4"));
        assert!(!t("B3 D4 F4"));
    }
}
