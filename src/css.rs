use anyhow::bail;
use lol_html::html_content::{ContentType, Element};
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
                    "Can't inline styles to {}: missing import files {:?}",
                    self.file.as_path().file_name().unwrap().to_str().unwrap(),
                    missing_import_paths
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
        println!("Import path: {}", path.display());
        if !path.exists() {
            missing_files.push(path);
            return "".to_string();
        } else {
            let new_root = &path.parent().unwrap_or(Path::new(""));
            deps.push(path.clone());
            match inline_imports(fs::read_to_string(&path).unwrap(), &new_root.to_path_buf()) {
                Ok((css, nested_deps)) => {
                    deps.extend(nested_deps);
                    css
                }
                Err(nested_missing_files) => {
                    missing_files.extend(nested_missing_files);
                    "".to_string()
                }
            }
        }
    });

    if !missing_files.is_empty() {
        return Err(missing_files);
    }

    Ok((css.to_string(), deps))
}