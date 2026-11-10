use super::*;

fn pins(ids: &[&str]) -> Pins {
    Pins {
        pins: ids.iter().map(|s| s.to_string()).collect(),
    }
}

#[test]
fn pin_and_unpin() {
    let mut p = pins(&["a/x", "a/y"]);
    p.pin("b/z", Some(1)).unwrap();
    assert_eq!(p.pins, ["b/z", "a/x", "a/y"]);
    p.pin("a/y", Some(1)).unwrap();
    assert_eq!(p.pins, ["a/y", "b/z", "a/x"]);
    assert_eq!(p.position("a/x"), Some(3));
    assert!(p.unpin("b/z"));
    assert!(!p.unpin("b/z"));
}

#[test]
fn at_most_nine() {
    let mut p = Pins::default();
    for i in 0..MAX_PINS {
        p.pin(&format!("a/p{i}"), None).unwrap();
    }
    assert!(p.pin("a/extra", None).is_err());
    // Moving an existing pin still works.
    p.pin("a/p8", Some(1)).unwrap();
}

#[test]
fn merge_keeps_both_sides() {
    let base = pins(&["a", "b", "c"]);
    let ours = pins(&["c", "a", "b", "d"]);
    let theirs = pins(&["a", "c", "e"]);
    // Theirs removed b; ours added d; theirs added e. Our order wins.
    assert_eq!(merge(&base, &ours, &theirs).pins, ["c", "a", "d", "e"]);
}

#[test]
fn file_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let p = pins(&["personal/a", "acme/b"]);
    p.save(dir.path()).unwrap();
    assert_eq!(Pins::load(dir.path()).unwrap(), p);
    Pins::default().save(dir.path()).unwrap();
    assert_eq!(Pins::load(dir.path()).unwrap(), Pins::default());
    assert_eq!(
        Pins::load(&dir.path().join("missing")).unwrap(),
        Pins::default()
    );
}
