use rehype_tree_sitter_highlight::{HighlightConfiguration, grammar};

#[test]
fn js_highlight_ordering() -> anyhow::Result<()> {
  let cwd = std::env::current_dir()?;

  let grammars = grammar::load_grammars(&[cwd.join("../../fixtures/grammars/")])?;
  let highlight_configs = HighlightConfiguration::from_query_paths(&grammars, &[]);
  let mut highlighter = rehype_tree_sitter_highlight::Highlighter::new(highlight_configs);

  let source = b"console.log(\"content\")";

  let events = highlighter.highlight(source, "javascript");

  let highlights = events
    .iter()
    .filter_map(|event| match event {
      rehype_tree_sitter_highlight::HighlightEvent::Highlight(highlight) => Some(highlight),
      _ => None,
    })
    .collect::<Vec<_>>();

  assert_eq!(
    highlights,
    &[
      "variable",
      "variable.builtin",
      "punctuation.delimiter",
      "property",
      "function.method",
      "punctuation.bracket",
      "string",
      "punctuation.bracket",
    ]
  );

  Ok(())
}

#[test]
fn js_object_property_ordering() -> anyhow::Result<()> {
  let cwd = std::env::current_dir()?;

  let grammars = grammar::load_grammars(&[cwd.join("../../fixtures/grammars/")])?;
  let highlight_configs = HighlightConfiguration::from_query_paths(&grammars, &[]);
  let mut highlighter = rehype_tree_sitter_highlight::Highlighter::new(highlight_configs);

  let source = b"console.log({a: 1})";

  let events = highlighter.highlight(source, "javascript");

  let highlights = events
    .iter()
    .filter_map(|event| match event {
      rehype_tree_sitter_highlight::HighlightEvent::Highlight(highlight) => Some(highlight),
      _ => None,
    })
    .collect::<Vec<_>>();

  assert_eq!(
    highlights,
    &[
      "variable",
      "variable.builtin",
      "punctuation.delimiter",
      "property",
      "function.method",
      "punctuation.bracket",
      "punctuation.bracket",
      "property",
      "number",
      "punctuation.bracket",
      "punctuation.bracket"
    ]
  );

  Ok(())
}

#[test]
fn clojure_highlight_ordering() -> anyhow::Result<()> {
  let cwd = std::env::current_dir()?;

  let grammars = grammar::load_grammars(&[cwd.join("../../fixtures/grammars/")])?;
  let highlight_configs = HighlightConfiguration::from_query_paths(
    &grammars,
    &[cwd.join("../../fixtures/nvim-treesitter/queries")],
  );
  let mut highlighter = rehype_tree_sitter_highlight::Highlighter::new(highlight_configs);

  let source = b"(sum 1 22)";
  let events = highlighter.highlight(source, "clojure");

  let highlights = events
    .iter()
    .filter_map(|event| match event {
      rehype_tree_sitter_highlight::HighlightEvent::Highlight(highlight) => Some(highlight),
      _ => None,
    })
    .collect::<Vec<_>>();

  assert_eq!(
    highlights,
    &[
      "punctuation.bracket",
      "variable",
      "function.call",
      "number",
      "number",
      "punctuation.bracket"
    ]
  );

  Ok(())
}
