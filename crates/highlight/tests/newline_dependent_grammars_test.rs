use rehype_tree_sitter_highlight::{HighlightConfiguration, HighlightEvent, grammar};

#[test]
fn markdown_without_newline() -> anyhow::Result<()> {
  let cwd = std::env::current_dir()?;

  let grammars = grammar::load_grammars(&[cwd.join("../../fixtures/grammars/")])?;
  let highlight_configs = HighlightConfiguration::from_query_paths(&grammars, &[]);
  let mut highlighter = rehype_tree_sitter_highlight::Highlighter::new(highlight_configs);

  let source = b"```clojure
(println 1)
```";

  let events = highlighter.highlight(source, "markdown");

  assert_eq!(
    events,
    &[
      HighlightEvent::Highlight("text.literal".into()),
      HighlightEvent::Highlight("punctuation.delimiter".into()),
      HighlightEvent::Source { start: 0, end: 3 },
      HighlightEvent::HighlightEnd,
      HighlightEvent::Source { start: 3, end: 11 },
      HighlightEvent::Highlight("none".into()),
      HighlightEvent::Highlight("punctuation.special".into()),
      HighlightEvent::HighlightEnd,
      HighlightEvent::Source { start: 11, end: 23 },
      HighlightEvent::HighlightEnd,
      HighlightEvent::Highlight("punctuation.delimiter".into()),
      HighlightEvent::Highlight("punctuation.special".into()),
      HighlightEvent::Source { start: 23, end: 23 },
      HighlightEvent::HighlightEnd,
      HighlightEvent::Source { start: 23, end: 26 },
      HighlightEvent::HighlightEnd,
      HighlightEvent::Source { start: 26, end: 26 },
      HighlightEvent::HighlightEnd
    ]
  );

  Ok(())
}

#[test]
fn clojure_with_markdown_no_newline() -> anyhow::Result<()> {
  let cwd = std::env::current_dir()?;

  let grammars = grammar::load_grammars(&[cwd.join("../../fixtures/grammars/")])?;
  let highlight_configs =
    HighlightConfiguration::from_query_paths(&grammars, &[cwd.join("../../fixtures/queries")]);
  let mut highlighter = rehype_tree_sitter_highlight::Highlighter::new(highlight_configs);

  let source = b"(defn some-function 
 \"## This is markdown
  
   ```clojure
   (println 1)
   ```\"
  [])";

  let events = highlighter.highlight(source, "clojure");

  assert_eq!(
    events,
    &[
      HighlightEvent::Highlight("punctuation.bracket".into()),
      HighlightEvent::Source { start: 0, end: 1 },
      HighlightEvent::HighlightEnd,
      HighlightEvent::Highlight("variable".into()),
      HighlightEvent::Highlight("function.call".into()),
      HighlightEvent::Highlight("keyword.function".into()),
      HighlightEvent::Source { start: 1, end: 5 },
      HighlightEvent::HighlightEnd,
      HighlightEvent::HighlightEnd,
      HighlightEvent::HighlightEnd,
      HighlightEvent::Source { start: 5, end: 6 },
      HighlightEvent::Highlight("variable".into()),
      HighlightEvent::Highlight("function".into()),
      HighlightEvent::Source { start: 6, end: 19 },
      HighlightEvent::HighlightEnd,
      HighlightEvent::HighlightEnd,
      HighlightEvent::Source { start: 19, end: 22 },
      HighlightEvent::Highlight("string".into()),
      HighlightEvent::Source { start: 22, end: 23 },
      HighlightEvent::Highlight("punctuation.special".into()),
      HighlightEvent::Source { start: 23, end: 25 },
      HighlightEvent::HighlightEnd,
      HighlightEvent::Source { start: 25, end: 26 },
      HighlightEvent::Highlight("text.title".into()),
      HighlightEvent::Source { start: 26, end: 42 },
      HighlightEvent::HighlightEnd,
      HighlightEvent::Source { start: 42, end: 46 },
      HighlightEvent::Highlight("text.literal".into()),
      HighlightEvent::Highlight("punctuation.delimiter".into()),
      HighlightEvent::Source { start: 46, end: 52 },
      HighlightEvent::HighlightEnd,
      HighlightEvent::Source { start: 52, end: 60 },
      HighlightEvent::Highlight("none".into()),
      HighlightEvent::Highlight("punctuation.special".into()),
      HighlightEvent::HighlightEnd,
      HighlightEvent::Source { start: 60, end: 63 },
      HighlightEvent::Highlight("punctuation.bracket".into()),
      HighlightEvent::Source { start: 63, end: 64 },
      HighlightEvent::HighlightEnd,
      HighlightEvent::Highlight("variable".into()),
      HighlightEvent::Highlight("function.call".into()),
      HighlightEvent::Highlight("function.builtin".into()),
      HighlightEvent::Source { start: 64, end: 71 },
      HighlightEvent::HighlightEnd,
      HighlightEvent::HighlightEnd,
      HighlightEvent::HighlightEnd,
      HighlightEvent::Source { start: 71, end: 72 },
      HighlightEvent::Highlight("number".into()),
      HighlightEvent::Source { start: 72, end: 73 },
      HighlightEvent::HighlightEnd,
      HighlightEvent::Highlight("punctuation.bracket".into()),
      HighlightEvent::Source { start: 73, end: 74 },
      HighlightEvent::HighlightEnd,
      HighlightEvent::Source { start: 74, end: 75 },
      HighlightEvent::HighlightEnd,
      HighlightEvent::Highlight("punctuation.delimiter".into()),
      HighlightEvent::Highlight("punctuation.special".into()),
      HighlightEvent::HighlightEnd,
      HighlightEvent::Source { start: 75, end: 81 },
      HighlightEvent::HighlightEnd,
      HighlightEvent::HighlightEnd,
      HighlightEvent::Source { start: 81, end: 82 },
      HighlightEvent::HighlightEnd,
      HighlightEvent::Source { start: 82, end: 85 },
      HighlightEvent::Highlight("punctuation.bracket".into()),
      HighlightEvent::Source { start: 85, end: 86 },
      HighlightEvent::HighlightEnd,
      HighlightEvent::Highlight("punctuation.bracket".into()),
      HighlightEvent::Source { start: 86, end: 87 },
      HighlightEvent::HighlightEnd,
      HighlightEvent::Highlight("punctuation.bracket".into()),
      HighlightEvent::Source { start: 87, end: 88 },
      HighlightEvent::HighlightEnd
    ]
  );

  Ok(())
}
