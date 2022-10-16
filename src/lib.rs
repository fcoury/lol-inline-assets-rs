use std::{fs, path::Path};

use lol_html::{element, html_content::ContentType, HtmlRewriter, Settings};

pub fn inline<P>(file: P) -> anyhow::Result<String>
where
    P: AsRef<Path>,
{
    let html = fs::read_to_string(&file)?;
    let root = file.as_ref().parent().unwrap_or(Path::new(""));

    let mut output = vec![];
    let mut rewriter = HtmlRewriter::new(
        Settings {
            element_content_handlers: vec![
                element!("img", |el| {
                    let src = el.get_attribute("src");
                    if src.is_none() {
                        return Ok(());
                    }
                    let src = src.unwrap();
                    if src.starts_with("http") || src.starts_with("data:") {
                        return Ok(());
                    }

                    let path = root.clone().join(src);
                    let img_contents = fs::read(&path)?;
                    let new_src = base64::encode(img_contents);
                    let new_src = format!("data:image/png;base64,{}", new_src);

                    el.set_attribute("src", &new_src)?;
                    Ok(())
                }),
                element!("link", |el| {
                    let rel = el.get_attribute("rel");
                    let typ = el.get_attribute("type");

                    if let Some(rel) = rel {
                        if rel != "stylesheet" {
                            return Ok(());
                        }
                    }

                    if let Some(typ) = typ {
                        if typ != "text/css" {
                            return Ok(());
                        }
                    }

                    let href = el.get_attribute("href");
                    if href.is_none() {
                        return Ok(());
                    }
                    let href = href.unwrap();

                    if !href.ends_with(".css")
                        || href.starts_with("http")
                        || href.starts_with("data:")
                    {
                        return Ok(());
                    }

                    let path = root.clone().join(href);
                    let mut css = fs::read_to_string(&path)?;

                    if let Some(media) = el.get_attribute("media") {
                        css = format!("@media {} {{ {} }}", media, css);
                    }

                    el.replace(
                        &format!(r#"<style type="text/css">{}</style>"#, css),
                        ContentType::Html,
                    );

                    Ok(())
                }),
                element!("script", |el| {
                    let typ = el.get_attribute("type");
                    if let Some(typ) = typ {
                        if typ != "text/javascript" {
                            return Ok(());
                        }
                    }

                    let src = el.get_attribute("src");
                    if src.is_none() {
                        return Ok(());
                    }
                    let src = src.unwrap();

                    if src.starts_with("http") || src.starts_with("data:") {
                        return Ok(());
                    }

                    let path = root.clone().join(src);
                    let js = fs::read_to_string(&path)?;

                    el.replace(
                        &format!("<script type=\"text/javascript\">{}</script>", js),
                        ContentType::Html,
                    );

                    Ok(())
                }),
            ],
            ..Settings::default()
        },
        |c: &[u8]| output.extend_from_slice(c),
    );

    rewriter.write(html.as_bytes())?;
    rewriter.end()?;

    let output = String::from_utf8(output)?;
    Ok(output)
}
