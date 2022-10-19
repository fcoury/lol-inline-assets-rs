use std::{
    fs,
    io::{Error, ErrorKind},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use lol_html::{element, html_content::ContentType, HtmlRewriter, Settings};

pub struct InlineResult {
    pub html: String,
    pub files: Vec<PathBuf>,
}

pub fn inline<P>(file: P) -> anyhow::Result<InlineResult>
where
    P: AsRef<Path>,
{
    let html = fs::read_to_string(&file)?;
    let root = file.as_ref().parent().unwrap_or(Path::new(""));

    let mut output = vec![];
    let deps = Arc::new(Mutex::new(vec![]));
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

                    let path = root.clone().join(&src);
                    if !path.exists() {
                        return Err(Box::new(Error::new(
                            ErrorKind::NotFound,
                            format!(
                                "Can't inline image to {}: file \"{}\" does not exist",
                                file.as_ref().file_name().unwrap().to_str().unwrap(),
                                src,
                            ),
                        )));
                    }
                    let img_contents = fs::read(&path)?;
                    let mut deps = deps.lock().unwrap();
                    deps.push(path);
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

                    let path = root.clone().join(&href);
                    if !path.exists() {
                        return Err(Box::new(Error::new(
                            ErrorKind::NotFound,
                            format!(
                                "Can't inline styles to {}: file \"{}\" does not exist",
                                file.as_ref().file_name().unwrap().to_str().unwrap(),
                                href,
                            ),
                        )));
                    }
                    let mut css = fs::read_to_string(&path)?;
                    let mut deps = deps.lock().unwrap();
                    deps.push(path);

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

                    let path = root.clone().join(&src);
                    if !path.exists() {
                        return Err(Box::new(Error::new(
                            ErrorKind::NotFound,
                            format!(
                                "Can't inline script to {}: file \"{}\" does not exist",
                                file.as_ref().file_name().unwrap().to_str().unwrap(),
                                src,
                            ),
                        )));
                    }
                    let js = fs::read_to_string(&path)?;
                    let mut deps = deps.lock().unwrap();
                    deps.push(path);

                    el.replace(&format!("<script>{}</script>", js), ContentType::Html);

                    Ok(())
                }),
            ],
            ..Settings::default()
        },
        |c: &[u8]| output.extend_from_slice(c),
    );

    rewriter.write(html.as_bytes())?;
    rewriter.end()?;

    let html = String::from_utf8(output)?;
    let files = Arc::try_unwrap(deps).unwrap();
    let files = files.into_inner().unwrap();
    Ok(InlineResult { html, files })
}
