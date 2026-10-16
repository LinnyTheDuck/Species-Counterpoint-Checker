use std::fmt;
use std::str::FromStr;

use crate::pitch::write_alter;
use crate::{Letter, ParseError, Pitch};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Major,
    Minor,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Key {
    pub tonic: Letter,
    pub alter: i8,
    pub mode: Mode,
}

impl Key {
    pub fn new(tonic: Letter, alter: i8, mode: Mode) -> Key {
        Key { tonic, alter, mode }
    }

    /// Returns the scale degree of a pitch, 1 to 7.
    pub fn degree(self, p: Pitch) -> i32 {
        (p.letter.index() - self.tonic.index()).rem_euclid(7) + 1
    }

    /// Returns how many semitones a pitch sits above its degree in the major or natural minor scale.
    pub fn alteration(self, p: Pitch) -> i32 {
        let scale = match self.mode {
            Mode::Major => [0, 2, 4, 5, 7, 9, 11],
            Mode::Minor => [0, 2, 3, 5, 7, 8, 10],
        };
        let tonic = self.tonic.semitones() + self.alter as i32;
        let actual = (p.semitones() - tonic).rem_euclid(12);
        let diff = actual - scale[(self.degree(p) - 1) as usize];
        // Wraps into -6..=5 so a flat reads as -1, not 11.
        (diff + 6).rem_euclid(12) - 6
    }

    /// Returns true for a raised 6th or 7th in a minor key.
    pub fn is_raised(self, p: Pitch) -> bool {
        self.mode == Mode::Minor && matches!(self.degree(p), 6 | 7) && self.alteration(p) == 1
    }

    /// Returns true for a natural 6th or 7th in a minor key.
    pub fn is_natural_minor_6_7(self, p: Pitch) -> bool {
        self.mode == Mode::Minor && matches!(self.degree(p), 6 | 7) && self.alteration(p) == 0
    }

    /// Returns true when a pitch belongs to the key, counting raised 6th and 7th in minor.
    pub fn is_diatonic(self, p: Pitch) -> bool {
        self.alteration(p) == 0 || self.is_raised(p)
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.tonic)?;
        write_alter(f, self.alter)?;
        let mode = match self.mode {
            Mode::Major => "major",
            Mode::Minor => "minor",
        };
        write!(f, " {mode}")
    }
}

impl FromStr for Key {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Key, ParseError> {
        let err = || {
            ParseError(format!(
                "invalid key '{s}', expected a form like 'C major' or 'F# minor'"
            ))
        };
        let (tonic, mode) = s.trim().split_once(' ').ok_or_else(err)?;
        let mode = match mode.trim().to_ascii_lowercase().as_str() {
            "major" => Mode::Major,
            "minor" => Mode::Minor,
            _ => return Err(err()),
        };
        let mut chars = tonic.chars();
        let letter = chars.next().and_then(Letter::from_char).ok_or_else(err)?;
        let alter = match chars.as_str() {
            "" => 0,
            "#" => 1,
            "b" => -1,
            _ => return Err(err()),
        };
        Ok(Key::new(letter, alter, mode))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(s: &str) -> Pitch {
        s.parse().unwrap()
    }

    #[test]
    fn parses_and_displays() {
        for s in ["C major", "F# minor", "Bb major"] {
            assert_eq!(s.parse::<Key>().unwrap().to_string(), s);
        }
        assert!("H major".parse::<Key>().is_err());
        assert!("C dorian".parse::<Key>().is_err());
    }

    #[test]
    fn knows_scale_notes() {
        let eb: Key = "Eb major".parse().unwrap();
        assert!(eb.is_diatonic(p("Ab3")));
        assert!(!eb.is_diatonic(p("A3")));
        assert_eq!(eb.degree(p("D4")), 7);
    }

    #[test]
    fn allows_raised_6_7_in_minor() {
        let a: Key = "A minor".parse().unwrap();
        assert!(a.is_diatonic(p("G4")));
        assert!(a.is_raised(p("G#4")));
        assert!(a.is_raised(p("F#4")));
        assert!(a.is_natural_minor_6_7(p("F4")));
        assert!(!a.is_diatonic(p("C#4")));
    }
}
