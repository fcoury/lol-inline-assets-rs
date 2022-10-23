use lol_inline_assets::inline;
use std::{fs, path::Path};

#[test]
fn image() {
    let inlined = inline("tests/fixtures/image.html").unwrap();
    let expected = fs::read_to_string("tests/fixtures/image.inlined.html").unwrap();
    assert_eq!(inlined.html, expected);
    assert_eq!(
        inlined.files,
        vec![Path::new("tests/fixtures/./assets/logo.png")]
    );
}

#[test]
fn css() {
    let inlined = inline("tests/fixtures/css.html").unwrap();
    let expected = fs::read_to_string("tests/fixtures/css.inlined.html").unwrap();
    assert_eq!(inlined.html, expected);
    assert_eq!(
        inlined.files,
        vec![Path::new("tests/fixtures/./assets/style.css")]
    );
}

#[test]
fn javascript() {
    let inlined = inline("tests/fixtures/javascript.html").unwrap();
    let expected = fs::read_to_string("tests/fixtures/javascript.inlined.html").unwrap();
    assert_eq!(inlined.html, expected);
    assert_eq!(
        inlined.files,
        vec![Path::new("tests/fixtures/./assets/script.js")]
    );
}

#[test]
fn html() {
    let inlined = inline("tests/fixtures/html-include.html").unwrap();
    let expected = fs::read_to_string("tests/fixtures/html-include.inlined.html").unwrap();
    assert_eq!(inlined.html, expected);
    assert_eq!(
        inlined.files,
        vec![Path::new("tests/fixtures/./assets/js/include.js")]
    );
}
