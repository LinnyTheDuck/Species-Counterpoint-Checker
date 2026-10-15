use std::fmt;

use crate::Pitch;

/// Semitones in each natural simple interval of the major scale.
const MAJOR_SEMITONES: [i32; 7] = [0, 2, 4, 5, 7, 9, 11];

/// Interval quality; augmented and diminished carry how many times.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Quality {
    Perfect,
    Major,
    Minor,
    Augmented(i32),
    Diminished(i32),
}

/// An undirected interval between two pitches.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Interval {
    /// Diatonic steps, 0 for a unison, 7 for an octave.
    pub steps: i32,
    /// Semitones between the two pitches.
    pub semitones: i32,
}

impl Interval {
    /// Returns the interval between two pitches in either order.
    pub fn between(a: Pitch, b: Pitch) -> Interval {
        let mut steps = b.diatonic() - a.diatonic();
        let mut semitones = b.semitones() - a.semitones();
        if steps < 0 || (steps == 0 && semitones < 0) {
            steps = -steps;
            semitones = -semitones;
        }
        Interval { steps, semitones }
    }

    /// Returns the interval number reduced to one octave, 1 to 7.
    pub fn simple_number(self) -> i32 {
        self.steps % 7 + 1
    }

    /// Returns the interval quality.
    pub fn quality(self) -> Quality {
        let simple = (self.steps % 7) as usize;
        let diff = self.semitones - 12 * (self.steps / 7) - MAJOR_SEMITONES[simple];
        if matches!(simple, 0 | 3 | 4) {
            match diff {
                0 => Quality::Perfect,
                d if d > 0 => Quality::Augmented(d),
                d => Quality::Diminished(-d),
            }
        } else {
            match diff {
                0 => Quality::Major,
                -1 => Quality::Minor,
                d if d > 0 => Quality::Augmented(d),
                d => Quality::Diminished(-d - 1),
            }
        }
    }

    /// Returns true for a perfect unison, 4th, 5th or octave, compounds included.
    pub fn is_perfect(self) -> bool {
        self.quality() == Quality::Perfect
    }

    /// Returns true for a perfect unison or octave, compounds included.
    pub fn is_octave_class(self) -> bool {
        self.is_perfect() && self.simple_number() == 1
    }

    /// Returns true for a perfect 5th, compounds included.
    pub fn is_fifth_class(self) -> bool {
        self.is_perfect() && self.simple_number() == 5
    }

    /// Returns true for a consonance between two voices: P1, 3rds, P5, 6ths, P8.
    pub fn is_consonant(self) -> bool {
        match self.simple_number() {
            1 | 5 => self.quality() == Quality::Perfect,
            3 | 6 => matches!(self.quality(), Quality::Major | Quality::Minor),
            _ => false,
        }
    }

    /// Returns true for an augmented or diminished interval.
    pub fn is_altered(self) -> bool {
        matches!(
            self.quality(),
            Quality::Augmented(_) | Quality::Diminished(_)
        )
    }

    /// Returns true for a melodic step, a 2nd.
    pub fn is_step(self) -> bool {
        self.steps == 1
    }

    /// Returns true for a melodic leap, a 3rd or more.
    pub fn is_leap(self) -> bool {
        self.steps >= 2
    }
}

impl fmt::Display for Interval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let prefix = match self.quality() {
            Quality::Perfect => "P".to_string(),
            Quality::Major => "M".to_string(),
            Quality::Minor => "m".to_string(),
            Quality::Augmented(n) => "A".repeat(n as usize),
            Quality::Diminished(n) => "d".repeat(n as usize),
        };
        write!(f, "{}{}", prefix, self.steps + 1)
    }
}

/// Direction of a melodic move: -1 down, 0 none, 1 up.
pub fn direction(from: Pitch, to: Pitch) -> i32 {
    (to.semitones() - from.semitones()).signum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn iv(a: &str, b: &str) -> Interval {
        Interval::between(a.parse().unwrap(), b.parse().unwrap())
    }

    #[test]
    fn names_intervals() {
        let cases = [
            ("C4", "C4", "P1"),
            ("C4", "E4", "M3"),
            ("E4", "C5", "m6"),
            ("F4", "B4", "A4"),
            ("B3", "F4", "d5"),
            ("C4", "G5", "P12"),
            ("D4", "B4", "M6"),
            ("C4", "C#4", "A1"),
            ("C4", "Bb4", "m7"),
            ("C#4", "Bb4", "d7"),
            ("G4", "C4", "P5"),
        ];
        for (a, b, name) in cases {
            assert_eq!(iv(a, b).to_string(), name, "{a}-{b}");
        }
    }

    #[test]
    fn classifies_consonance() {
        assert!(iv("C4", "G5").is_consonant());
        assert!(iv("C4", "A4").is_consonant());
        assert!(!iv("C4", "F4").is_consonant());
        assert!(!iv("F4", "B4").is_consonant());
        assert!(!iv("C4", "D5").is_consonant());
    }
}
