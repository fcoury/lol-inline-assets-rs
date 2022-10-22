use lol_inline_assets::inline;
use std::{fs, path::Path};

#[test]
fn css_with_import() {
    let inlined = inline("tests/fixtures/css-nested.html").unwrap();
    let expected = fs::read_to_string("tests/fixtures/css-nested.inlined.html").unwrap();
    assert_eq!(inlined.html, expected);
    assert_eq!(
        inlined.files,
        vec![
            Path::new("tests/fixtures/./assets/import.css"),
            Path::new("tests/fixtures/./assets/external/style.css"),
            Path::new("tests/fixtures/./assets/external/second.css"),
        ]
    );
}

#[test]
fn css_with_url() {
    let inlined = inline("tests/fixtures/css-nested-withurl.html").unwrap();
    let expected = fs::read_to_string("tests/fixtures/css-nested-withurl.inlined.html").unwrap();
    assert_eq!(inlined.html, expected);
    assert_eq!(
        inlined.files,
        vec![
            Path::new("tests/fixtures/./assets/url.css"),
            Path::new("tests/fixtures/./assets/fonts/OpenSans-Bold.woff2"),
            Path::new("tests/fixtures/./assets/fonts/OpenSans-Bold.eot"),
            Path::new("tests/fixtures/./assets/fonts/OpenSans-Bold.woff2"),
            Path::new("tests/fixtures/./assets/fonts/OpenSans-Bold.woff"),
            Path::new("tests/fixtures/./assets/fonts/OpenSans-Bold.ttf"),
            Path::new("tests/fixtures/./assets/fonts/OpenSans-Bold.svg")
        ]
    );
}
