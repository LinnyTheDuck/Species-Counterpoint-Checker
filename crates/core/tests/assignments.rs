//! Full-marks assignments: each must check with no errors.

use counterpoint_core::{FirstSpecies, Pitch, Placement, Severity};

fn line(s: &str) -> Vec<Pitch> {
    s.split_whitespace().map(|p| p.parse().unwrap()).collect()
}

fn assert_no_errors(key: &str, placement: Placement, cf: &str, cp: &str, ties: &[usize]) {
    let exercise = FirstSpecies {
        key: key.parse().unwrap(),
        placement,
        cantus_firmus: line(cf),
        counterpoint: line(cp),
        ties: ties.to_vec(),
    };
    let errors: Vec<_> = exercise
        .check()
        .unwrap()
        .into_iter()
        .filter(|v| v.severity == Severity::Error)
        .collect();
    assert!(errors.is_empty(), "{errors:#?}");
}

#[test]
fn first_species_a_minor_below() {
    assert_no_errors(
        "A minor",
        Placement::Below,
        "A4 B4 C5 B4 D5 C5 D5 E5 B4 A4",
        "A2 G#2 A2 B2 B2 E3 F3 E3 G#2 A2",
        &[4],
    );
}
