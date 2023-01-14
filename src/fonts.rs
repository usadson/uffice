// Copyright (C) 2022 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

use std::{path::{PathBuf, Path}, collections::HashMap, rc::Rc};

use font_kit::family_name::FamilyName;
use sfml::graphics::Font;

use crate::text_settings;

// TODO Implement file watching to look for new fonts installed during the running of the program ^_^
pub struct WinFontCacheSource {
    directories: Vec<PathBuf>,

}

impl WinFontCacheSource {
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

impl font_kit::source::Source for WinFontCacheSource {
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

pub struct FontManager {
    source: font_kit::sources::multi::MultiSource,
    cache: HashMap<String, CacheEntry>,
}

struct CacheEntry {
    variant_bold: Option<Rc<sfml::SfBox<Font>>>,
    variant_normal: Option<Rc<sfml::SfBox<Font>>>,
}

impl CacheEntry {
    fn new() -> Self {
        Self {
            variant_bold: None,
            variant_normal: None,
        }
    }
}

pub fn resolve_font_sources() -> Vec<Box<(dyn font_kit::source::Source + 'static)>> {
    let mut sources = vec![];

    #[cfg(target_os = "windows")]
    {
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
        Box::new(WinFontCacheSource::new(
            sources
        ))
    ]
}

impl FontManager {
    pub fn new() -> Self {
        Self {
            source: font_kit::sources::multi::MultiSource::from_sources(resolve_font_sources()),
            cache: HashMap::new(),
        }
    }

    pub fn load_font(&mut self, text_settings: &text_settings::TextSettings) -> Rc<sfml::SfBox<Font>> {
        let font: &str = match &text_settings.font {
            Some(font) => font,
            None => "Calibri"
        };

        let mut properties = font_kit::properties::Properties::new();

        let bold = text_settings.bold.unwrap_or(false);
        if bold {
            properties.weight = font_kit::properties::Weight::BOLD;
        }

        let entry = self.cache.entry(String::from(font)).or_insert_with(|| CacheEntry::new());
        if bold {
            if let Some(font) = &entry.variant_bold {
                return font.clone();
            }
        } else if let Some(font) = &entry.variant_normal {
            return font.clone();
        }

        let family_names = [FamilyName::Title(String::from(font))];
        let handle = self.source.select_best_match(&family_names, &properties)
                .expect("Failed to find font");


        let font = match handle {
            font_kit::handle::Handle::Memory { bytes, font_index: _ } => {
                unsafe {
                    Font::from_memory(&bytes).unwrap()
                }
            }
            font_kit::handle::Handle::Path { path, font_index: _ } => {
                Font::from_file(path.to_str().unwrap()).unwrap()
            }
        };

        if bold {
            entry.variant_bold = Some(Rc::new(font));
            return entry.variant_bold.as_ref().unwrap().clone();
        }
        entry.variant_normal = Some(Rc::new(font));
        return entry.variant_normal.as_ref().unwrap().clone();
    }
}


