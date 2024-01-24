// Copyright (C) 2022 - 2023 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

use std::path::PathBuf;

/// Looks for fonts in the given directories.
pub struct DirectoryFontSource {
    directories: Vec<PathBuf>,
}

impl DirectoryFontSource {
    pub fn new(directories: Vec<PathBuf>) -> Self {
        Self {
            directories
        }
    }
}

type SelectionResult = Result<font_kit::family_handle::FamilyHandle, font_kit::error::SelectionError>;

struct FontSearcher<'a> {
    family_name: &'a str
}

impl<'a> FontSearcher<'a> {
    fn search(&self, dir: &PathBuf) -> Option<font_kit::family_handle::FamilyHandle> {
        if !dir.is_dir() {
            return None;
        }

        if let Ok(iter) = std::fs::read_dir(dir) {
            for entry in iter {
                if entry.is_err() {
                    continue;
                }

                let entry = entry.unwrap();
                if !entry.path().is_dir() {
                    continue;
                }

                if entry.file_name().eq_ignore_ascii_case(self.family_name) {
                    if let Some(result) = self.search_family_name_dir(&entry.path()) {
                        return Some(result);
                    }
                }

                if let Some(result) = self.search(&entry.path()) {
                    return Some(result);
                }
            }
        }

        None
    }

    fn search_family_name_dir(&self, dir: &PathBuf) -> Option<font_kit::family_handle::FamilyHandle> {
        let iter = std::fs::read_dir(dir);
        if iter.is_err() {
            return None;
        }

        let mut handle = font_kit::family_handle::FamilyHandle::new();

        let mut index = 0;

        let iter = iter.unwrap();
        for entry in iter {
            if entry.is_err() {
                continue;
            }

            let entry = entry.unwrap();
            if entry.path().is_dir() {
                continue;
            }

            if let Some(extension) = entry.path().extension() {
                if !extension.to_string_lossy().eq_ignore_ascii_case("ttf") {
                    continue;
                }

                handle.push(font_kit::handle::Handle::from_path(entry.path(), index));
                index += 1;
            }
        }

        if handle.is_empty() {
            return None;
        }

        Some(handle)
    }

}

impl font_kit::source::Source for DirectoryFontSource {
    fn all_fonts(&self) -> Result<Vec<font_kit::handle::Handle>, font_kit::error::SelectionError> {
        todo!()
    }

    fn all_families(&self) -> Result<Vec<String>, font_kit::error::SelectionError> {
        todo!()
    }

    fn select_family_by_name(&self, family_name: &str) -> SelectionResult {
        for directory in &self.directories {
            if !directory.is_dir() {
                continue;
            }

            let searcher = FontSearcher{family_name};
            if let Some(result) = searcher.search(directory) {
                return Ok(result);
            }
        }

        Err(font_kit::error::SelectionError::NotFound)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        todo!()
    }

    fn as_mut_any(&mut self) -> &mut dyn std::any::Any {
        todo!()
    }
}

/// Generates font sources based on the platform.
pub fn resolve_font_sources() -> Vec<Box<(dyn font_kit::source::Source + 'static)>> {
    #[cfg(target_os = "windows")]
    let mut sources = vec![];

    #[cfg(target_os = "windows")]
    {
        use std::path::Path;
        let str = format!("{}\\Microsoft\\FontCache\\4\\CloudFonts", env!("LOCALAPPDATA"));

        match Path::new(&str).canonicalize() {
            Ok(path) => {
                sources.push(path);
            }
            Err(e) => {
                println!("[ResolveFontSources] Failed to locate Windows FontCache \"{}\": {:?}", str, e);
                std::process::exit(0)
            }
        }
    }

    vec![
        Box::new(font_kit::source::SystemSource::new()),

        #[cfg(target_os = "windows")]
        Box::new(DirectoryFontSource::new(
            sources
        ))
    ]
}
