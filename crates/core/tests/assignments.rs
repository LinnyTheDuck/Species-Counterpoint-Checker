//! Graded assignments, also the page's examples: each must check with no errors or warnings.

use counterpoint_core::{FirstSpecies, Pitch, Placement, SecondSpecies};

fn line(s: &str) -> Vec<Pitch> {
    s.split_whitespace().map(|p| p.parse().unwrap()).collect()
}

fn assert_clean(key: &str, placement: Placement, cf: &str, cp: &str, ties: &[usize]) {
    let exercise = FirstSpecies {
        key: key.parse().unwrap(),
        placement,
        cantus_firmus: line(cf),
        counterpoint: line(cp),
        ties: ties.to_vec(),
    };
    let violations = exercise.check().unwrap();
    assert!(violations.is_empty(), "{violations:#?}");
}

#[test]
fn first_species_a_minor_below() {
    assert_clean(
        "A minor",
        Placement::Below,
        "A4 B4 C5 B4 D5 C5 D5 E5 B4 A4",
        "A2 G#2 A2 B2 B2 E3 F3 E3 G#2 A2",
        &[4],
    );
}

#[test]
fn second_species_d_minor_above() {
    // With a correction since the original leapt an augmented 5th, F4 to C#5 in bar 5.
    let bars = "_ F4 | A4 Bb4 | C5 G4 | E4 G4 | F4 A4 | B4 C#5 | D5 A4 | G4 F4 | E4 | D4";
    let exercise = SecondSpecies {
        key: "D minor".parse().unwrap(),
        placement: Placement::Above,
        cantus_firmus: line("D3 F3 E3 G3 A3 G3 F3 E3 C#3 D3"),
        counterpoint: bars
            .split('|')
            .map(|bar| {
                bar.split_whitespace()
                    .map(|p| (p != "_").then(|| p.parse().unwrap()))
                    .collect()
            })
            .collect(),
    };
    let violations = exercise.check().unwrap();
    assert!(violations.is_empty(), "{violations:#?}");
}
