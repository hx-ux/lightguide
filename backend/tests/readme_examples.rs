//! Sanity check that the README examples produce the comments they claim.
//! Run with: `cargo test --test readme_examples -- --nocapture`.

use harmonia::{
    Chord, Key, Note, PitchClass, Scale, detect_key, suggest_next_chords,
    suggest_scales_for_bracket,
};

#[test]
fn detect_key_example() {
    let progression: Vec<Chord> = ["C", "Am", "Dm", "G7"]
        .iter()
        .map(|s| s.parse().unwrap())
        .collect();

    let candidates = detect_key(&progression);
    let best = &candidates[0];
    let line = format!("{}: {}/{} chords fit", best.key, best.matched, best.total);
    assert_eq!(line, "C major: 4/4 chords fit");
}

#[test]
fn roman_analysis_example() {
    let c_major = Key::new(PitchClass::C);
    let lines: Vec<String> = ["Am", "F", "G7", "C7", "F#"]
        .iter()
        .map(|symbol| {
            let chord: Chord = symbol.parse().unwrap();
            let roman = c_major
                .roman_for(chord)
                .map(|r| r.to_string())
                .unwrap_or_else(|| "—".into());
            format!("{symbol:>4} → {roman}")
        })
        .collect();
    assert_eq!(
        lines,
        vec![
            "  Am → vi",
            "   F → IV",
            "  G7 → V7",
            "  C7 → I7",
            "  F# → —",
        ]
    );
}

#[test]
fn no_string() {
    let mut f = Scale::new(PitchClass::C, harmonia::ScaleKind::Ionian);
    // f.root = PitchClass::D;

    let notes: Option<[Note; 7]> = f.spelled();

    assert!(true);
    // let labels: Vec<String> = f.spelled().unwrap().iter().map(|n| n.to_string()).collect();

    // let hh = labels;
}

#[test]
fn scale_spelling_example() {
    let g_major: Scale = "G Ionian".parse().unwrap();
    let labels: Vec<String> = g_major
        .spelled()
        .unwrap()
        .iter()
        .map(|n| n.to_string())
        .collect();

    assert_eq!(labels, ["G", "A", "B", "C", "D", "E", "F♯"]);
}

#[test]
fn suggest_next_chords_example() {
    let history: Vec<Chord> = ["C", "Am", "F"]
        .iter()
        .map(|s| s.parse().unwrap())
        .collect();

    let result = suggest_next_chords(&history, None);
    let key = result.key.unwrap();
    assert_eq!(key.key.tonic, PitchClass::C);

    let lines: Vec<String> = result
        .suggestions
        .iter()
        .take(6)
        .map(|s| {
            let roman = s.roman.as_ref().map(|r| r.to_string()).unwrap_or_default();
            format!("  {} ({}) — {}", s.chord, roman, s.reason)
        })
        .collect();
    assert_eq!(
        lines,
        vec![
            "  C (I) — diatonic in C major",
            "  Dm (ii) — diatonic in C major",
            "  Em (iii) — diatonic in C major",
            "  F (IV) — diatonic in C major",
            "  G (V) — diatonic in C major",
            "  Am (vi) — diatonic in C major",
        ]
    );
}

#[test]
fn bracket_scales_example() {
    let g: Chord = "G".parse().unwrap();
    let am: Chord = "Am".parse().unwrap();

    let suggestions = suggest_scales_for_bracket(Some(g), Some(am));
    let lines: Vec<String> = suggestions
        .iter()
        .take(3)
        .map(|s| format!("{:<22} {}", s.scale.to_string(), s.reasoning))
        .collect();
    assert!(lines[0].starts_with("G Ionian (major)"));
    assert!(lines[1].starts_with("G Mixolydian"));
    assert!(lines[2].starts_with("A Dorian"));
    for line in &lines {
        assert!(
            line.contains("G and Am are both diatonic"),
            "expected diatonic reasoning, got {line:?}"
        );
    }
}
