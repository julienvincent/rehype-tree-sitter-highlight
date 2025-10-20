use napi_derive::napi;
use rehype_tree_sitter_highlight::{HighlightConfiguration, grammar};

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
pub struct Highlighter {
  highlighter: rehype_tree_sitter_highlight::Highlighter,
}

#[napi]
impl Highlighter {
  #[napi(constructor)]
  pub fn new(grammar_paths: Vec<String>, query_paths: Option<Vec<String>>) -> napi::Result<Self> {
    let cwd = std::env::current_dir()?;

    let search_paths = grammar_paths
      .iter()
      .map(|dir| cwd.join(dir))
      .collect::<Vec<_>>();

    let query_dirs = query_paths
      .unwrap_or_default()
      .iter()
      .map(|dir| cwd.join(dir))
      .collect::<Vec<_>>();

    let grammars = grammar::load_grammars(&search_paths)
      .map_err(|err| napi::Error::from_reason(format!("{err:?}")))?;

    let highlight_configs = HighlightConfiguration::from_query_paths(&grammars, &query_dirs);

    Ok(Self {
      highlighter: rehype_tree_sitter_highlight::Highlighter::new(highlight_configs),
    })
  }

  #[napi]
  pub fn highlight(
    &mut self,
    source: String,
    language: String,
  ) -> napi::Result<Vec<HighlightEvent>> {
    let source = source.into_bytes();

    let highlights = self.highlighter.highlight(source.as_slice(), &language);

    let events = highlights
      .iter()
      .map(|event| match event {
        rehype_tree_sitter_highlight::HighlightEvent::Highlight(s) => HighlightEvent {
          event_type: HighlightEventType::Start,
          highlight: Some(s.clone()),
          range: None,
        },
        rehype_tree_sitter_highlight::HighlightEvent::Source { start, end } => HighlightEvent {
          event_type: HighlightEventType::Source,
          highlight: None,
          range: Some(HighlightRange {
            start: *start as u32,
            end: *end as u32,
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
}
