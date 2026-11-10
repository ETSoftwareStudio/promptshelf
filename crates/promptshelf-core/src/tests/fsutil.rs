use super::*;

#[test]
fn civil_dates() {
    assert_eq!(civil_from_days(0), (1970, 1, 1));
    assert_eq!(civil_from_days(20_401), (2025, 11, 9));
    assert_eq!(civil_from_days(11_016), (2000, 2, 29));
}

#[test]
fn create_new_does_not_overwrite() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("a.md");
    assert!(create_new(&path, b"one").unwrap());
    assert!(!create_new(&path, b"two").unwrap());
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "one");
}
