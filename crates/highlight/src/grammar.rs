use anyhow::{Context, Result};
use std::{collections::HashMap, fs, path::PathBuf};
use tree_sitter::Language;
use tree_sitter_loader::{CompileConfig, Loader};

#[derive(Debug)]
pub struct LoadedGrammar {
  pub name: String,
  pub lang: Language,
  pub injections: Vec<PathBuf>,
  pub highlights: Vec<PathBuf>,
}

pub type Grammars = HashMap<String, LoadedGrammar>;

pub fn load_grammars(search_paths: &[PathBuf]) -> Result<Grammars> {
  let mut loader = Loader::new()?;
  let mut languages = HashMap::new();

  for dir in search_paths {
    let entries = fs::read_dir(dir)
      .with_context(|| format!("Failed to read directory {:?}", dir))?
      .filter_map(|entry| match entry {
        Ok(entry) => {
          let path = entry.path();
          if path.is_dir() { Some(path) } else { None }
        }
        Err(_) => None,
      });

    for path in entries {
      loader
        .find_language_configurations_at_path(&path, false)
        .with_context(|| format!("Failed to load language configuration from {:?}", path))?;
    }
  }

  for (config, path) in loader.get_all_language_configurations() {
    let src_path = path.join("src");

    let language = loader
      .load_language_at_path(CompileConfig::new(&src_path, None, None))
      .with_context(|| format!("Failed to load language {}", config.language_name))?;

    let injections = config
      .injections_filenames
      .clone()
      .unwrap_or_default()
      .iter()
      .map(|path| config.root_path.join(path))
      .collect::<Vec<_>>();

    let highlights = config
      .highlights_filenames
      .clone()
      .unwrap_or_default()
      .iter()
      .map(|path| config.root_path.join(path))
      .collect::<Vec<_>>();

    languages.insert(
      config.language_name.clone(),
      LoadedGrammar {
        name: config.language_name.clone(),
        lang: language,
        injections,
        highlights,
      },
    );
  }

  Ok(languages)
}
