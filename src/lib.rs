use css::Css;
use lol_html::{element, html_content::ContentType, HtmlRewriter, Settings};
use std::{
    fs,
    io::{Error, ErrorKind},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

mod css;
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

    let css = Css::new(file.as_ref(), root, &deps);

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

                    let path = root.join(&src);
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

                    let mime_type:String = if path.extension().map_or(false, |element| element == "svg") {
                        "image/svg+xml".into()
                    } else if let Some(kind) = infer::get(&img_contents) {
                        kind.mime_type().into()
                    } else {
                        let extension = path.extension()
                            .and_then(|element| element.to_str())
                            .unwrap_or("");
                        mime_guess::from_ext(extension)
                            .first_or_octet_stream()
                            .to_string()
                    };
                    
                    if !mime_type.starts_with("image/") {
                        return Err(Box::new(Error::new(
                            ErrorKind::InvalidData,
                            format!("File {} is not a recognized image type", src),
                        )))
                    }

                    let mut deps = deps.lock().unwrap();
                    deps.push(path);
                    let new_src = base64::encode(img_contents);
                    let new_src = format!("data:{};base64,{}", mime_type, new_src);

                    el.set_attribute("src", &new_src)?;
                    Ok(())
                }),
                element!("link", |el| match css.handle(el) {
                    Ok(_) => Ok(()),
                    Err(e) => {
                        let err: Box<Error> = Box::new(e.downcast().unwrap());
                        Err(err)
                    }
                }),
                element!("include", |el| {
                    let src = el.get_attribute("src");
                    if src.is_none() {
                        return Ok(());
                    }
                    let src = src.unwrap();

                    if src.starts_with("http") || src.starts_with("data:") {
                        return Ok(());
                    }

                    let path = root.join(&src);
                    if !path.exists() {
                        return Err(Box::new(Error::new(
                            ErrorKind::NotFound,
                            format!(
                                "Can't include to {}: file \"{}\" does not exist",
                                file.as_ref().file_name().unwrap().to_str().unwrap(),
                                src,
                            ),
                        )));
                    }
                    let contents = fs::read_to_string(&path)?;
                    let mut deps = deps.lock().unwrap();
                    deps.push(path);

                    el.replace(&contents, ContentType::Html);

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

                    let base64 = el.get_attribute("base64");
                    if base64.is_some() {
                        let path = root.join(&src);
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
                        let js = fs::read(&path)?;
                        let mut deps = deps.lock().unwrap();
                        deps.push(path);
                        let new_src = base64::encode(js);
                        let new_src = format!("data:application/javascript;base64,{}", new_src);

                        el.set_attribute("src", &new_src)?;
                        return Ok(());
                    }

                    let path = root.join(&src);
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
