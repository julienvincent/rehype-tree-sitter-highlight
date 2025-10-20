use anyhow::Result;
use std::{fs, path::PathBuf};
use tree_sitter::Query;

use crate::grammar;

fn read_files(paths: &[PathBuf]) -> Result<String> {
  let mut out = String::new();
  for (i, p) in paths.iter().enumerate() {
    let contents = fs::read_to_string(p)
      .map_err(|e| anyhow::format_err!("Failed to read {}: {e}", p.display()))?;
    if i > 0 {
      out.push('\n');
    }
    out.push_str(&contents);
  }
  Ok(out)
}

fn merge_queries(base: &str, overlay: &str) -> String {
  if base.is_empty() {
    return overlay.to_owned();
  }

  if overlay.is_empty() {
    return base.to_owned();
  }

  let mut merged = String::with_capacity(base.len() + overlay.len() + 1);
  merged.push_str(base);
  if !base.ends_with('\n') {
    merged.push('\n');
  }
  merged.push_str(overlay);

  merged
}

fn is_extending(contents: &str) -> bool {
  contents
    .lines()
    .next()
    .map(|line| line.trim_start().starts_with(";; extends"))
    .unwrap_or(false)
}

fn read_query(queries_dirs: &[PathBuf], name: &str, filename: &str, base: &str) -> Result<String> {
  let mut result = base.to_owned();

  for dir in queries_dirs {
    let path = dir.join(name).join(filename);
    if path.is_file() {
      let contents = fs::read_to_string(&path)
        .map_err(|e| anyhow::format_err!("Failed to read {}: {e}", path.display()))?;

      if is_extending(&contents) {
        result = merge_queries(&result, &contents);
      } else {
        result = contents;
      }
    }
  }

  Ok(result)
}

pub fn load_query(
  grammar: &grammar::LoadedGrammar,
  base_files: &[PathBuf],
  queries_dirs: &[PathBuf],
  file_name: &str,
) -> Result<Query> {
  let base_queries = read_files(base_files)?;

  let query_content = read_query(queries_dirs, &grammar.name, file_name, &base_queries)?;

  Query::new(&grammar.lang, &query_content).map_err(|err| anyhow::format_err!("{err:?}"))
}
