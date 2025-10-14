use napi_derive::napi;
use rehype_tree_sitter_highlight::{HighlightConfiguration, Highlighter};
use std::{
  collections::{HashMap, HashSet},
  fs,
  path::PathBuf,
  sync::LazyLock,
};

mod grammar;

static HIGHLIGHT_NAMES: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
  vec![
    "attribute",
    "boolean",
    "carriage-return",
    "comment",
    "comment.documentation",
    "constant",
    "constant.builtin",
    "constructor",
    "constructor.builtin",
    "embedded",
    "error",
    "escape",
    "function",
    "function.builtin",
    "function.call",
    "keyword",
    "markup",
    "markup.bold",
    "markup.heading",
    "markup.italic",
    "markup.link",
    "markup.link.url",
    "markup.list",
    "markup.list.checked",
    "markup.list.numbered",
    "markup.list.unchecked",
    "markup.list.unnumbered",
    "markup.quote",
    "markup.raw",
    "markup.raw.block",
    "markup.raw.inline",
    "markup.strikethrough",
    "module",
    "number",
    "operator",
    "property",
    "property.builtin",
    "punctuation",
    "punctuation.bracket",
    "punctuation.delimiter",
    "punctuation.special",
    "string",
    "string.escape",
    "string.regexp",
    "string.special",
    "string.special.symbol",
    "tag",
    "type",
    "type.builtin",
    "variable",
    "variable.builtin",
    "variable.member",
    "variable.parameter",
  ]
  .into_iter()
  .collect()
});

fn read_files(paths: &[PathBuf]) -> napi::Result<String> {
  let mut out = String::new();
  for (i, p) in paths.iter().enumerate() {
    let contents = fs::read_to_string(p)
      .map_err(|e| napi::Error::from_reason(format!("Failed to read {}: {e}", p.display())))?;
    if i > 0 {
      out.push('\n');
    }
    out.push_str(&contents);
  }
  Ok(out)
}

fn read_merged_query(queries_dirs: &[PathBuf], name: &str, filename: &str) -> String {
  let mut contents = Vec::new();
  for dir in queries_dirs {
    let path = dir.join(name).join(filename);
    if let Ok(text) = fs::read_to_string(&path) {
      contents.push(text);
    }
  }
  contents.join("\n")
}

fn merge_queries(base: &str, extra: &str) -> String {
  match (base.is_empty(), extra.is_empty()) {
    (true, true) => String::new(),
    (false, true) => base.to_string(),
    (true, false) => extra.to_string(),
    (false, false) => format!("{base}\n{extra}"),
  }
}

fn load_highlight_config(
  highlight_names: &[&str],
  grammar: &grammar::LoadedGrammar,
  queries_dirs: &[PathBuf],
) -> napi::Result<HighlightConfiguration> {
  let base_highlights = read_files(&grammar.highlights)?;
  let base_injections = read_files(&grammar.injections)?;
  let base_locals = read_files(&grammar.locals)?;

  let extra_highlights = read_merged_query(queries_dirs, &grammar.name, "highlights.scm");
  let extra_injections = read_merged_query(queries_dirs, &grammar.name, "injections.scm");
  let extra_locals = read_merged_query(queries_dirs, &grammar.name, "locals.scm");

  let merged_highlights = merge_queries(&base_highlights, &extra_highlights);
  let merged_injections = merge_queries(&base_injections, &extra_injections);
  let merged_locals = merge_queries(&base_locals, &extra_locals);

  let mut config = HighlightConfiguration::new(
    grammar.lang.clone(),
    grammar.name.as_str(),
    &merged_highlights,
    &merged_injections,
    &merged_locals,
  )
  .map_err(|err| napi::Error::from_reason(format!("{err:?}")))?;

  config.configure(highlight_names);
  Ok(config)
}

#[napi]
pub enum HighlightEventType {
  Start,
  Source,
  End,
}

#[napi(object)]
pub struct HighlightRange {
  pub start: u32,
  pub end: u32,
}

#[napi(object)]
pub struct HighlightEvent {
  #[napi(js_name = "type")]
  pub event_type: HighlightEventType,
  pub highlight: Option<String>,
  pub range: Option<HighlightRange>,
}

#[napi(object)]
pub struct HighlightParams {
  pub source: String,
  pub language: String,

  #[napi(js_name = "query_paths")]
  pub query_paths: Option<Vec<String>>,

  #[napi(js_name = "grammar_paths")]
  pub grammar_paths: Option<Vec<String>>,
}

#[napi]
pub fn highlight(params: HighlightParams) -> napi::Result<Vec<HighlightEvent>> {
  let cwd = std::env::current_dir()?;

  let query_dirs = params
    .query_paths
    .unwrap_or_default()
    .iter()
    .map(|dir| cwd.join(dir))
    .collect::<Vec<_>>();

  let search_paths = params
    .grammar_paths
    .unwrap_or_default()
    .iter()
    .map(|dir| cwd.join(dir))
    .collect::<Vec<_>>();

  let highlight_names = HIGHLIGHT_NAMES.iter().cloned().collect::<Vec<_>>();

  let grammars = grammar::load_grammars(&search_paths)
    .map_err(|err| napi::Error::from_reason(format!("{err:?}")))?;

  let highlight_configs = grammars
    .iter()
    .filter_map(|(lang, grammar)| {
      match load_highlight_config(&highlight_names, grammar, &query_dirs) {
        Ok(config) => Some((lang.as_str(), config)),
        Err(err) => {
          eprintln!("Failed to load grammar: {err:?}");
          None
        }
      }
    })
    .collect::<HashMap<_, _>>();

  let mut highlighter = Highlighter::new();

  let lang_to_config = |name: &str| highlight_configs.get(name);

  let config = highlight_configs.get(params.language.as_str());
  let Some(config) = config else {
    return Err(napi::Error::from_reason(format!(
      "Unknown language {}",
      params.language
    )));
  };

  let source = params.source.into_bytes();

  let highlights = highlighter
    .highlight(config, source.as_slice(), lang_to_config)
    .unwrap();

  let events = highlights
    .map(|event| match event.unwrap() {
      rehype_tree_sitter_highlight::HighlightEvent::HighlightStart(s) => HighlightEvent {
        event_type: HighlightEventType::Start,
        highlight: Some(highlight_names[s.0].into()),
        range: None,
      },
      rehype_tree_sitter_highlight::HighlightEvent::Source { start, end } => HighlightEvent {
        event_type: HighlightEventType::Source,
        highlight: None,
        range: Some(HighlightRange {
          start: start as u32,
          end: end as u32,
        }),
      },
      rehype_tree_sitter_highlight::HighlightEvent::HighlightEnd => HighlightEvent {
        event_type: HighlightEventType::End,
        highlight: None,
        range: None,
      },
    })
    .collect::<Vec<_>>();

  Ok(events)
}
