use anyhow::Result;
use std::{collections::HashMap, ops::Deref};
use tree_sitter::{Language, Parser, Query, QueryCursor, QueryProperty, Range, StreamingIterator};

#[derive(Debug, Clone)]
pub struct HighlightRegion {
  pub range: Range,
  pub highlight: String,
  pub priority: u32,
  pub pattern_index: u32,
}

pub fn get_priority(properties: &[QueryProperty]) -> Option<u32> {
  for property in properties {
    if property.key.deref() == "priority" {
      return property
        .value
        .clone()
        .and_then(|value| value.parse::<u32>().ok());
    }
  }
  None
}

pub fn query_highlights(
  parser: &mut Parser,
  lang: &Language,
  source: &[u8],
  query: &Query,
) -> Result<Vec<HighlightRegion>> {
  parser.set_language(lang)?;
  let tree = parser
    .parse(source, None)
    .ok_or_else(|| anyhow::anyhow!("Parse returned None"))?;

  let mut cursor = QueryCursor::new();
  let mut matches = cursor.matches(query, tree.root_node(), source);

  let capture_index = query
    .capture_names()
    .iter()
    .filter_map(|name| {
      query
        .capture_index_for_name(name)
        .map(|index| (index, *name))
    })
    .collect::<HashMap<_, _>>();

  let mut highlights: Vec<HighlightRegion> = Vec::new();
  while let Some(query_match) = matches.next() {
    let properties = query.property_settings(query_match.pattern_index);
    let predicates = query.general_predicates(query_match.pattern_index);
    let priority = get_priority(properties).unwrap_or(100);

    // Right now this highlighter does not support the lua-match? predicate and therefore these
    // captures should just be completely excluded (as they only optionally match)
    if predicates
      .iter()
      .any(|pred| pred.operator.deref() == "lua-match?")
    {
      continue;
    }

    for capture in query_match.captures {
      if let Some(highlight_name) = capture_index.get(&capture.index) {
        match *highlight_name {
          "nospell" => {}
          "spell" => {}
          "conceal" => {}
          value => {
            if !value.starts_with("_") {
              highlights.push(HighlightRegion {
                highlight: value.to_string(),
                range: capture.node.range(),
                pattern_index: query_match.pattern_index as u32,
                priority,
              });
            }
          }
        }
      }
    }
  }

  Ok(highlights)
}
