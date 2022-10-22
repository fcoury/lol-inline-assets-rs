use lol_inline_assets::inline;
use std::{fs, path::Path};

#[test]
fn css_with_import() {
    let inlined = inline("tests/fixtures/css-nested.html").unwrap();
    let expected = fs::read_to_string("tests/fixtures/css-nested.inlined.html").unwrap();
    fs::write("/tmp/css-nested.inlined.html", &inlined.html).unwrap();
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
