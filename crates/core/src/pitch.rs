use std::fmt;
use std::str::FromStr;

use crate::ParseError;

/// Note letter names in scale order from C.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Letter {
    C,
    D,
    E,
    F,
    G,
    A,
    B,
}

impl Letter {
    /// Returns the position in the C major scale, 0 for C.
    pub fn index(self) -> i32 {
        self as i32
    }

    /// Returns the semitones above C of the natural note.
    pub fn semitones(self) -> i32 {
        [0, 2, 4, 5, 7, 9, 11][self as usize]
    }

    /// Parses a letter name, upper or lower case.
    pub fn from_char(c: char) -> Option<Letter> {
        match c.to_ascii_uppercase() {
            'C' => Some(Letter::C),
            'D' => Some(Letter::D),
            'E' => Some(Letter::E),
            'F' => Some(Letter::F),
            'G' => Some(Letter::G),
            'A' => Some(Letter::A),
            'B' => Some(Letter::B),
            _ => None,
        }
    }
}

impl fmt::Display for Letter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Writes an alteration as sharps or flats.
pub(crate) fn write_alter(f: &mut fmt::Formatter<'_>, alter: i8) -> fmt::Result {
    let sign = if alter > 0 { "#" } else { "b" };
    write!(f, "{}", sign.repeat(alter.unsigned_abs() as usize))
}

/// A spelled pitch, octave numbered so that middle C is C4.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pitch {
    pub letter: Letter,
    pub alter: i8,
    pub octave: i8,
}

impl Pitch {
    /// Creates a pitch from its letter, alteration in semitones and octave.
    pub fn new(letter: Letter, alter: i8, octave: i8) -> Pitch {
        Pitch {
            letter,
            alter,
            octave,
        }
    }

    /// Returns the number of diatonic steps above C0.
    pub fn diatonic(self) -> i32 {
        self.octave as i32 * 7 + self.letter.index()
    }

    /// Returns the number of semitones above C0.
    pub fn semitones(self) -> i32 {
        self.octave as i32 * 12 + self.letter.semitones() + self.alter as i32
    }
}

impl fmt::Display for Pitch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.letter)?;
        write_alter(f, self.alter)?;
        write!(f, "{}", self.octave)
    }
}

impl FromStr for Pitch {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Pitch, ParseError> {
        let err = || {
            ParseError(format!(
                "invalid pitch '{s}', expected a form like C4, F#3 or Bb4"
            ))
        };
        let mut chars = s.trim().chars().peekable();
        let letter = chars.next().and_then(Letter::from_char).ok_or_else(err)?;
        let mut alter: i8 = 0;
        while let Some(&c) = chars.peek() {
            match c {
                '#' => alter += 1,
                'b' => alter -= 1,
                _ => break,
            }
            chars.next();
        }
        let octave: i8 = chars.collect::<String>().parse().map_err(|_| err())?;
        if !(0..=9).contains(&octave) || alter.abs() > 2 {
            return Err(err());
        }
        Ok(Pitch::new(letter, alter, octave))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_displays() {
        for s in ["C4", "F#3", "Bb4", "Ebb2", "G##5"] {
            assert_eq!(s.parse::<Pitch>().unwrap().to_string(), s);
        }
    }

    #[test]
    fn rejects_bad_input() {
        for s in ["", "H4", "C", "C#", "C10", "Cx4"] {
            assert!(s.parse::<Pitch>().is_err(), "{s}");
        }
    }

    #[test]
    fn counts_steps_and_semitones() {
        let c4: Pitch = "C4".parse().unwrap();
        let b3: Pitch = "B3".parse().unwrap();
        assert_eq!(c4.semitones() - b3.semitones(), 1);
        assert_eq!(c4.diatonic() - b3.diatonic(), 1);
    }
}
