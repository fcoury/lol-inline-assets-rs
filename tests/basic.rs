use std::{fs, path::Path};

use lol_inline_assets::inline;

#[test]
fn image() {
    let inlined = inline(Path::new("tests/fixtures/image.html")).unwrap();
    let expected = fs::read_to_string("tests/fixtures/image.inlined.html").unwrap();
    assert_eq!(inlined, expected);
}

#[test]
fn css() {
    let inlined = inline(Path::new("tests/fixtures/css.html")).unwrap();
    let expected = fs::read_to_string(Path::new("tests/fixtures/css.inlined.html")).unwrap();
    assert_eq!(inlined, expected);
}

#[test]
fn javascript() {
    let inlined = inline(Path::new("tests/fixtures/javascript.html")).unwrap();
    let expected = fs::read_to_string(Path::new("tests/fixtures/javascript.inlined.html")).unwrap();
    assert_eq!(inlined, expected);
}
