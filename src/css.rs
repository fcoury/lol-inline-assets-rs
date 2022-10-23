use anyhow::bail;
use lol_html::html_content::{ContentType, Element};
use mime_guess::Mime;
use regex::{Captures, Regex};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

pub struct Css<'a> {
    file: PathBuf,
    root: PathBuf,
    deps: &'a Arc<Mutex<Vec<PathBuf>>>,
}

impl<'a> Css<'a> {
    pub fn new<P>(file: P, root: P, deps: &'a Arc<Mutex<Vec<PathBuf>>>) -> Self
    where
        P: AsRef<Path>,
    {
        Self {
            file: file.as_ref().to_path_buf(),
            root: root.as_ref().to_path_buf(),
            deps,
        }
    }

    pub fn handle(&self, el: &mut Element) -> anyhow::Result<()> {
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

        if !href.ends_with(".css") || href.starts_with("http") || href.starts_with("data:") {
            return Ok(());
        }

        let path = self.root.clone().join(&href);
        if !path.exists() {
            bail!(
                "Can't inline styles to {}: file \"{}\" does not exist",
                self.file.as_path().file_name().unwrap().to_str().unwrap(),
                href,
            );
        }
        let mut css = fs::read_to_string(&path)?;
        let mut deps = self.deps.lock().unwrap();
        deps.push(path.clone());

        if let Some(media) = el.get_attribute("media") {
            css = format!("@media {} {{ {} }}", media, css);
        }

        let new_root = path.parent().unwrap_or(Path::new(""));
        let res = match inline_imports(css, &new_root.to_path_buf()) {
            Ok(res) => res,
            Err(missing_import_paths) => {
                bail!(
                    "Can't inline styles to {}: missing files from @import {:?}",
                    self.file.as_path().file_name().unwrap().to_str().unwrap(),
                    missing_import_paths
                )
            }
        };
        css = res.0;
        deps.extend(res.1);

        let res = match inline_urls(css, &new_root.to_path_buf()) {
            Ok(res) => res,
            Err(missing_url_paths) => {
                bail!(
                    "Can't inline styles to {}: missing files from url() {:?}",
                    self.file.as_path().file_name().unwrap().to_str().unwrap(),
                    missing_url_paths
                )
            }
        };
        css = res.0;
        deps.extend(res.1);

        el.replace(
            &format!(r#"<style type="text/css">{}</style>"#, css),
            ContentType::Html,
        );

        Ok(())
    }
}

fn inline_imports(css: String, root: &PathBuf) -> Result<(String, Vec<PathBuf>), Vec<PathBuf>> {
    let re = Regex::new(r#"@import\s+("(.*)"|'(.*)')\s*;"#).unwrap();

    let mut deps = vec![];
    let mut missing_files = vec![];
    let css = re.replace_all(&css, |caps: &Captures| {
        let path = caps.get(2).or_else(|| caps.get(3)).unwrap().as_str();
        let path = root.join(path);
        if !path.exists() {
            missing_files.push(path);
            return "".to_string();
        }

        let new_root = &path.parent().unwrap_or(Path::new(""));
        deps.push(path.clone());

        let css = fs::read_to_string(&path).unwrap();
        let css = match inline_urls(css, &new_root.to_path_buf()) {
            Ok((css, nested_deps)) => {
                deps.extend(nested_deps);
                css
            }
            Err(missing_import_paths) => {
                missing_files.extend(missing_import_paths);
                return "".to_string();
            }
        };

        match inline_imports(css, &new_root.to_path_buf()) {
            Ok((css, nested_deps)) => {
                deps.extend(nested_deps);
                css
            }
            Err(nested_missing_files) => {
                missing_files.extend(nested_missing_files);
                "".to_string()
            }
        }
    });

    if !missing_files.is_empty() {
        return Err(missing_files);
    }

    Ok((css.to_string(), deps))
}

fn inline_urls(css: String, root: &PathBuf) -> Result<(String, Vec<PathBuf>), Vec<PathBuf>> {
    let dismiss_re = Regex::new(r#"url\(\s*(data|https?):.*\s*\)"#).unwrap();
    if dismiss_re.is_match(&css) {
        return Ok((css, vec![]));
    }

    let re = Regex::new(r#"\burl\(("([^"]*)"|'([^']*)')\)(\s*format\(("([^"]*)"|'([^']*)')\))?"#)
        .unwrap();
    let exluded_data_re = Regex::new(r#"^(data|https?):"#).unwrap();

    let mut deps = vec![];
    let mut missing_files = vec![];
    let css = re.replace_all(&css, |caps: &Captures| {
        let path = caps.get(2).or_else(|| caps.get(3)).unwrap().as_str();
        if exluded_data_re.is_match(path) {
            return caps.get(0).unwrap().as_str().to_string();
        }
        let path = path.split_once('?').unwrap_or_else(|| (path, path)).0;
        let path = path.split_once('#').unwrap_or_else(|| (path, path)).0;
        let path = root.join(path);

        if !path.exists() {
            missing_files.push(path);
            return "".to_string();
        }

        let format = match caps.get(6).or_else(|| caps.get(7)) {
            Some(format) => format!(r#" format("{}")"#, format.as_str()),
            None => "".to_string(),
        };

        let new_path = &path
            .parent()
            .unwrap_or(Path::new(""))
            .join(path.file_name().unwrap().to_str().unwrap());
        deps.push(path.clone());
        let mime = mime_guess::from_path(&path).first_or_octet_stream();
        format!(
            r#"url("{}"){}"#,
            to_base64(&fs::read(&new_path).unwrap(), mime),
            format,
        )
    });

    if !missing_files.is_empty() {
        return Err(missing_files);
    }

    Ok((css.to_string(), deps))
}

fn to_base64(contents: &[u8], mime: Mime) -> String {
    let contents = base64::encode(contents);
    format!("data:{};base64,{}", mime, contents)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_base64() {
        let contents = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let mime = mime_guess::from_path("test.eot").first_or_octet_stream();
        let expected = "data:application/vnd.ms-fontobject;base64,AAECAwQFBgcICQ==";
        assert_eq!(to_base64(&contents, mime), expected);
    }
}
