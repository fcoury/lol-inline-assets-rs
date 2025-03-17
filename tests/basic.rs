use lol_inline_assets::inline;
use std::{fs, path::Path};

fn test_image_helper(format: &str, mime_type: &str) {
    let fixture_dir = "tests/fixtures";
    let html_file = format!("{}/image_{}.html", fixture_dir, format);
    let expected_file = format!("{}/image_{}.inlined.html", fixture_dir, format);
    let image_path = format!("{}/./assets/test.{}", fixture_dir, format);

    let inlined = inline(&html_file).unwrap();
    let expected = fs::read_to_string(expected_file).unwrap();

    assert_eq!(
        inlined.html, expected,
        "HTML mismatch for {} format", format
    );

    assert!(
        inlined.html.contains(&format!("data:{};base64,", mime_type)),
        "Missing correct MIME type for {}", format
    );

    assert_eq!(
        inlined.files,
        vec![Path::new(&image_path)],
        "Incorrect file tracking for {}", format
    );
}

#[test]
fn jpg_image() {
    test_image_helper("jpg", "image/jpeg");
}

#[test]
fn svg_image() {
    test_image_helper("svg", "image/svg+xml");
}

#[test]
fn png_image() {
    let inlined:lol_inline_assets::InlineResult = inline("tests/fixtures/image_png.html").unwrap();
    let expected: String = fs::read_to_string("tests/fixtures/image_png.inlined.html").unwrap();
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

#[test]
fn html_inline_js_base64() {
    let inlined = inline("tests/fixtures/html-d3.html").unwrap();
    assert!(inlined
        .html
        .contains(r#"<script src="data:application/javascript;base64,Ly8gaHR0cHM6L"#));
}
