//! Rules from the "All species" table in RULES.md.

use crate::interval::direction;
use crate::{Interval, Key, Mode, Pitch, Position, Quality, Violation, Voice};

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
    pub fn interval(&self) -> Interval {
        Interval::between(self.cf, self.cp)
    }
}

/// Checks `cf-tonic`: the cantus firmus starts and ends on the tonic, which catches a wrongly chosen key.
pub fn cf_tonic(key: Key, cf: &Line, out: &mut Vec<Violation>) {
    let (Some(&first), Some(&last)) = (cf.notes.first(), cf.notes.last()) else {
        return;
    };
    let is_tonic = |p: Pitch| key.degree(p) == 1 && key.alteration(p) == 0;
    // A CF on the relative key's tonic usually means the other mode with the same signature was meant.
    let (relative_degree, other_mode) = match key.mode {
        Mode::Major => (6, Mode::Minor),
        Mode::Minor => (3, Mode::Major),
    };
    let hint = if first.letter == last.letter
        && first.alter == last.alter
        && key.degree(first) == relative_degree
        && key.alteration(first) == 0
    {
        format!(
            " Is the key {}?",
            Key::new(first.letter, first.alter, other_mode)
        )
    } else {
        String::new()
    };
    let ends = [(0, first, "starts"), (cf.notes.len() - 1, last, "ends")];
    for (i, p, verb) in ends {
        if !is_tonic(p) {
            out.push(Violation::warning(
                "cf-tonic",
                cf.at[i],
                Some(cf.voice),
                format!("The cantus firmus {verb} on {p}, not the tonic of {key}.{hint}"),
            ));
        }
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
        // Other moves, such as a raised 7th dipping to the 6th and back, are left to the line.
        let (dir, degree) = (direction(from, to), key.degree(from));
        if key.is_raised(from) && degree == 6 && dir < 0 {
            out.push(Violation::warning(
                "minor-6-7",
                line.at[i],
                voice,
                format!("Raised {from} should rise, not fall to {to}."),
            ));
        } else if key.is_natural_minor_6_7(from) && degree == 7 && dir > 0 {
            out.push(Violation::warning(
                "minor-6-7",
                line.at[i],
                voice,
                format!("Natural {from} rises to the tonic {to}. Raise it to the leading note."),
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
        } else if m.steps == 5 && m.quality() == Quality::Major {
            out.push(Violation::warning(
                "melodic-sixth",
                at,
                voice,
                format!("Melodic {m} from {from} to {to}. Only minor 6ths are allowed."),
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

/// Checks `start-perfect` on the first vertical; after an opening rest a 3rd above is allowed too.
pub fn start_perfect(
    first: &Vertical,
    placement: Placement,
    after_rest: bool,
    out: &mut Vec<Violation>,
) {
    let iv = first.interval();
    let above = placement == Placement::Above;
    let third = above && after_rest && iv.simple_number() == 3 && iv.is_consonant();
    let ok = iv.is_octave_class() || (above && iv.is_fifth_class()) || third;
    if !ok {
        let allowed = match (placement, after_rest) {
            (Placement::Above, true) => "a unison, 3rd, 5th or octave",
            (Placement::Above, false) => "a unison, 5th or octave",
            (Placement::Below, _) => "a unison or octave",
        };
        out.push(Violation::error(
            "start-perfect",
            first.at,
            None,
            format!("Starts on a {iv}, should be {allowed}."),
        ));
    }
}

/// Returns the interval into the final octave, as (number, quality) and a name, given whether the CF has the leading note.
pub fn into_final(placement: Placement, cf_leads: bool) -> ((i32, Quality), &'static str) {
    // M6 when the upper voice has the leading note, m3 when the lower voice has it.
    if (placement == Placement::Above) != cf_leads {
        ((6, Quality::Major), "a major 6th")
    } else {
        ((3, Quality::Minor), "a minor 3rd")
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

/// Checks `voice-crossing` on one vertical.
pub fn voice_crossing(v: &Vertical, placement: Placement, out: &mut Vec<Violation>) {
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

/// Both voices moving the same way into a perfect 5th, octave or unison.
pub struct IntoPerfect {
    /// True when the interval before is the same perfect interval, false for direct motion.
    pub parallel: bool,
    /// "5ths", "octaves" or "unisons".
    pub name: &'static str,
    pub from: Interval,
    pub to: Interval,
    pub upper_steps: bool,
}

/// Returns how the voices move into a perfect interval, or None when they do not.
pub fn into_perfect(prev: &Vertical, next: &Vertical, placement: Placement) -> Option<IntoPerfect> {
    let cf_dir = direction(prev.cf, next.cf);
    let cp_dir = direction(prev.cp, next.cp);
    let (from, to) = (prev.interval(), next.interval());
    if cf_dir == 0 || cf_dir != cp_dir || !(to.is_octave_class() || to.is_fifth_class()) {
        return None;
    }

    let name = if to.is_fifth_class() {
        "5ths"
    } else if to.steps == 0 {
        "unisons"
    } else {
        "octaves"
    };
    let parallel = (from.is_octave_class() && to.is_octave_class())
        || (from.is_fifth_class() && to.is_fifth_class());
    let (upper_from, upper_to) = match placement {
        Placement::Above => (prev.cp, next.cp),
        Placement::Below => (prev.cf, next.cf),
    };
    Some(IntoPerfect {
        parallel,
        name,
        from,
        to,
        upper_steps: Interval::between(upper_from, upper_to).is_step(),
    })
}

/// Checks `parallel-perfect` and `direct-perfect` from one vertical to the next.
pub fn parallels(prev: &Vertical, next: &Vertical, placement: Placement, out: &mut Vec<Violation>) {
    let Some(m) = into_perfect(prev, next, placement) else {
        return;
    };
    let IntoPerfect { name, from, to, .. } = m;
    if m.parallel {
        out.push(Violation::error(
            "parallel-perfect",
            next.at,
            None,
            format!("Parallel {name}: {from} to {to}."),
        ));
        return;
    }

    let message = format!("Direct {name}: both voices move the same way into a {to}.");
    if m.upper_steps {
        out.push(Violation::warning("direct-perfect", next.at, None, message));
    } else {
        out.push(Violation::error("direct-perfect", next.at, None, message));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cf_tonic_suggests_the_relative_key() {
        let notes: Vec<Pitch> = ["E3", "B2", "F#3", "E3"]
            .iter()
            .map(|p| p.parse().unwrap())
            .collect();
        let at: Vec<Position> = (1..=4).map(|bar| Position { bar, beat: 1 }).collect();
        let cf = Line {
            voice: Voice::CantusFirmus,
            notes: &notes,
            at: &at,
        };
        let mut out = Vec::new();
        cf_tonic("G major".parse().unwrap(), &cf, &mut out);
        let found: Vec<_> = out.iter().map(|v| (v.at.bar, v.message.as_str())).collect();
        assert_eq!(
            found,
            [
                (
                    1,
                    "The cantus firmus starts on E3, not the tonic of G major. Is the key E minor?"
                ),
                (
                    4,
                    "The cantus firmus ends on E3, not the tonic of G major. Is the key E minor?"
                ),
            ]
        );
        out.clear();
        cf_tonic("E minor".parse().unwrap(), &cf, &mut out);
        assert!(out.is_empty(), "{out:#?}");
    }

    #[test]
    fn minor_6_7_guards_the_tonic_and_the_5th() {
        let key: Key = "D minor".parse().unwrap();
        let warned = |s: &str| {
            let notes: Vec<Pitch> = s.split(' ').map(|p| p.parse().unwrap()).collect();
            let at: Vec<Position> = (1..=notes.len())
                .map(|bar| Position { bar, beat: 1 })
                .collect();
            let line = Line {
                voice: Voice::Counterpoint,
                notes: &notes,
                at: &at,
            };
            let mut out = Vec::new();
            key_rules(key, &line, &mut out);
            out.iter()
                .filter(|v| v.rule == "minor-6-7")
                .map(|v| v.at.bar)
                .collect::<Vec<_>>()
        };
        assert_eq!(warned("Bb4 C5 G4"), Vec::<usize>::new());
        assert_eq!(warned("C#5 B4 C#5 D5"), Vec::<usize>::new());
        assert_eq!(warned("C5 D5"), [1]);
        assert_eq!(warned("C#5 B4 A4"), [2]);
    }

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
