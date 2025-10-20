use anyhow::Result;
use std::{collections::HashMap, ops::Deref};
use tree_sitter::{
  Language, Parser, Point, Query, QueryCursor, QueryPredicate, QueryPredicateArg, QueryProperty,
  Range, StreamingIterator,
};

pub fn get_lang_name(properties: &[QueryProperty]) -> Option<String> {
  for property in properties {
    if property.key.deref() == "injection.language" {
      return property.value.clone().map(String::from);
    }
  }
  None
}

#[derive(Debug)]
struct RangeOffset {
  start_row: isize,
  start_col: isize,
  end_row: isize,
  end_col: isize,
}

fn parse_offset_predicate(pred: &QueryPredicate) -> Result<(u32, RangeOffset)> {
  if pred.args.len() != 5 {
    anyhow::bail!("Offset predicate requires 5 arguments");
  }

  let [
    QueryPredicateArg::Capture(capture),
    QueryPredicateArg::String(start_row),
    QueryPredicateArg::String(start_col),
    QueryPredicateArg::String(end_row),
    QueryPredicateArg::String(end_col),
  ] = pred.args.deref()
  else {
    anyhow::bail!("Offset predicate contained unexpected arguments");
  };

  let range = RangeOffset {
    start_row: start_row.parse()?,
    start_col: start_col.parse()?,
    end_row: end_row.parse()?,
    end_col: end_col.parse()?,
  };

  Ok((*capture, range))
}

fn get_offset_modifiers(predicates: &[QueryPredicate]) -> HashMap<u32, RangeOffset> {
  let mut map = HashMap::new();
  for pred in predicates {
    if pred.operator.deref() != "offset!" {
      continue;
    }

    let Ok((capture, range)) = parse_offset_predicate(pred) else {
      continue;
    };

    map.insert(capture, range);
  }

  map
}

fn point_to_byte(source: &str, point: Point) -> Option<usize> {
  let mut byte_index = 0;

  for (current_row, line) in source.split_inclusive('\n').enumerate() {
    if current_row == point.row {
      let mut col_byte = 0;
      for (i, ch) in line.chars().enumerate() {
        if i == point.column {
          break;
        }
        col_byte += ch.len_utf8();
      }
      return Some(byte_index + col_byte);
    }

    byte_index += line.len();
  }

  None
}

fn calculate_point_offset(value: usize, offset: isize) -> usize {
  ((value as isize) + offset) as usize
}

fn apply_offset_to_range(source: &str, range: &Range, offset: &RangeOffset) -> Range {
  let new_start_point = Point {
    row: calculate_point_offset(range.start_point.row, offset.start_row),
    column: calculate_point_offset(range.start_point.column, offset.start_col),
  };
  let new_end_point = Point {
    row: calculate_point_offset(range.end_point.row, offset.end_row),
    column: calculate_point_offset(range.end_point.column, offset.end_col),
  };

  let new_start_byte = point_to_byte(source, new_start_point).unwrap();
  let new_end_byte = point_to_byte(source, new_end_point).unwrap();

  Range {
    start_byte: new_start_byte,
    end_byte: new_end_byte,
    start_point: new_start_point,
    end_point: new_end_point,
  }
}

#[derive(Debug)]
pub struct InjectedRegion {
  pub range: Range,
  pub lang: String,
}

pub fn query_injections(
  parser: &mut Parser,
  lang: &Language,
  source: &[u8],
  query: &Query,
) -> Result<Vec<InjectedRegion>> {
  let source_str = String::from_utf8(Vec::from(source))?;

  parser.set_language(lang)?;
  let tree = parser
    .parse(source, None)
    .ok_or_else(|| anyhow::anyhow!("Parse returned None"))?;

  let mut injected_regions = Vec::new();

  let mut cursor = QueryCursor::new();
  let mut matches = cursor.matches(query, tree.root_node(), source);

  let lang_capture_index = query.capture_index_for_name("injection.language");
  let Some(content_capture_index) = query.capture_index_for_name("injection.content") else {
    return Ok(Vec::new());
  };

  while let Some(query_match) = matches.next() {
    let harcoded_lang_name = get_lang_name(query.property_settings(query_match.pattern_index));

    let mut lang_capture = None;
    let mut content_capture = None;
    for capture in query_match.captures {
      if let Some(lang_capture_index) = lang_capture_index {
        if capture.index == lang_capture_index {
          lang_capture = Some(capture);
        }
      }
      if capture.index == content_capture_index {
        content_capture = Some(capture);
      }
    }

    let Some(lang_name) = harcoded_lang_name.or_else(|| {
      lang_capture.and_then(|capture| capture.node.utf8_text(source).ok().map(String::from))
    }) else {
      continue;
    };

    let Some(content_capture) = content_capture else {
      continue;
    };

    let offset_modifiers =
      get_offset_modifiers(query.general_predicates(query_match.pattern_index));

    let range = if let Some(offset) = offset_modifiers.get(&content_capture.index) {
      apply_offset_to_range(&source_str, &content_capture.node.range(), offset)
    } else {
      content_capture.node.range()
    };

    injected_regions.push(InjectedRegion {
      lang: lang_name.clone(),
      range,
    });
  }

  Ok(injected_regions)
}
