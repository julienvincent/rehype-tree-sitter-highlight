use anyhow::Result;
use grammar::Grammars;
use std::{collections::HashMap, path::PathBuf};
use tree_sitter::{Language, Parser, Point, Query, Range};

pub mod grammar;
mod highlights;
mod injections;
pub mod queries;

use crate::highlights::HighlightRegion;

pub struct HighlightConfiguration {
  pub language: Language,
  pub injections: Query,
  pub highlights: Query,
}

type Configurations = HashMap<String, HighlightConfiguration>;

pub fn load_highlight_config(
  grammar: &grammar::LoadedGrammar,
  queries_dirs: &[PathBuf],
) -> Result<HighlightConfiguration> {
  let config = HighlightConfiguration {
    language: grammar.lang.clone(),
    injections: queries::load_query(grammar, &grammar.injections, queries_dirs, "injections.scm")
      .map_err(|err| anyhow::format_err!("{err:?}"))?,
    highlights: queries::load_query(grammar, &grammar.highlights, queries_dirs, "highlights.scm")
      .map_err(|err| anyhow::format_err!("{err:?}"))?,
  };

  Ok(config)
}

impl HighlightConfiguration {
  pub fn from_query_paths(grammars: &Grammars, query_dirs: &[PathBuf]) -> Configurations {
    grammars
      .iter()
      .filter_map(
        |(lang, grammar)| match load_highlight_config(grammar, query_dirs) {
          Ok(config) => Some((lang.clone(), config)),
          Err(err) => {
            eprintln!("Failed to load grammar {lang}: {err:?}");
            None
          }
        },
      )
      .collect()
  }
}

pub struct Highlighter {
  configurations: Configurations,
  parser: Parser,
}

impl Highlighter {
  pub fn new(configurations: Configurations) -> Self {
    Highlighter {
      parser: Parser::new(),
      configurations,
    }
  }
}

fn query_highlights<'a>(
  parser: &'a mut Parser,
  lang: &str,
  configurations: &'a Configurations,
  source: &[u8],
  layer: u32,
) -> Vec<highlights::HighlightRegion> {
  let Some(config) = configurations.get(lang) else {
    return Vec::new();
  };

  let injections =
    injections::query_injections(parser, &config.language, source, &config.injections).unwrap();
  let mut highlights =
    highlights::query_highlights(parser, &config.language, source, &config.highlights).unwrap();

  for region in injections {
    let injection_highlights = query_highlights(
      parser,
      &region.lang,
      configurations,
      &source[region.range.start_byte..region.range.end_byte],
      layer * 10,
    )
    .iter()
    .map(|highlight| highlights::HighlightRegion {
      highlight: highlight.highlight.clone(),
      priority: highlight.priority,
      pattern_index: layer * highlight.pattern_index,
      range: Range {
        start_byte: region.range.start_byte + highlight.range.start_byte,
        end_byte: region.range.start_byte + highlight.range.end_byte,
        start_point: Point {
          row: region.range.start_point.row + highlight.range.start_point.row,
          column: region.range.start_point.column + highlight.range.start_point.column,
        },
        end_point: Point {
          row: region.range.end_point.row + highlight.range.end_point.row,
          column: region.range.end_point.column + highlight.range.end_point.column,
        },
      },
    })
    .collect::<Vec<_>>();

    for highlight in injection_highlights {
      highlights.push(highlight)
    }
  }

  highlights
}

#[derive(Debug)]
pub enum HighlightEvent {
  Highlight(String),
  Source { start: usize, end: usize },
  HighlightEnd,
}

impl Highlighter {
  pub fn highlight(&mut self, source: &[u8], lang: &str) -> Vec<HighlightEvent> {
    let mut highlights = query_highlights(&mut self.parser, lang, &self.configurations, source, 1);
    highlights.sort_by(|a, b| {
      let start_pos = a.range.start_byte.cmp(&b.range.start_byte);
      match start_pos {
        std::cmp::Ordering::Equal => {
          let end_pos = b.range.end_byte.cmp(&a.range.end_byte);
          match end_pos {
            std::cmp::Ordering::Equal => a.pattern_index.cmp(&b.pattern_index),
            ordering => ordering,
          }
        }
        ordering => ordering,
      }
    });

    let mut events = Vec::new();

    let mut byte = 0;
    let mut index = 0;
    let mut stack: Vec<HighlightRegion> = Vec::new();
    while index < highlights.len() {
      let current = &highlights[index];

      let current_start = current.range.start_byte;

      if let Some(previous) = stack.last() {
        let previous_start = previous.range.start_byte;
        let previous_end = previous.range.end_byte;

        if previous_end <= current_start {
          if byte < previous_end {
            events.push(HighlightEvent::Source {
              start: byte,
              end: previous_end,
            });
            byte = previous_end;
          }

          stack.pop();
          events.push(HighlightEvent::HighlightEnd);
          continue;
        }

        if previous_start < current_start && byte < current_start {
          events.push(HighlightEvent::Source {
            start: byte,
            end: current_start,
          });
          byte = current_start;
        }
      } else if byte < current_start {
        events.push(HighlightEvent::Source {
          start: byte,
          end: current_start,
        });

        byte = current_start;
      }

      events.push(HighlightEvent::Highlight(current.highlight.clone()));
      stack.push(current.clone());
      index += 1;
    }

    stack.reverse();

    for highlight in stack {
      if byte <= highlight.range.end_byte {
        events.push(HighlightEvent::Source {
          start: byte,
          end: highlight.range.end_byte,
        });
        byte = highlight.range.end_byte;
      }
      events.push(HighlightEvent::HighlightEnd);
    }

    if byte < source.len() - 1 {
      events.push(HighlightEvent::Source {
        start: byte,
        end: source.len() - 1,
      });
    }

    events
  }
}
