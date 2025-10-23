use std::borrow::Cow;
use tree_sitter::{Point, Range};

fn point_for_byte(source: &[u8], byte_index: usize) -> Point {
  let target = byte_index.min(source.len());
  let mut row = 0;
  let mut column = 0;

  for byte in source.iter().take(target) {
    if *byte == b'\n' {
      row += 1;
      column = 0;
    } else {
      column += 1;
    }
  }

  Point { row, column }
}

pub type EndPoint = (usize, Point);

pub fn with_newline<'a>(source: &'a [u8]) -> (Cow<'a, [u8]>, Option<EndPoint>) {
  let original_len = source.len();
  let should_append_newline = !source.ends_with(b"\n");
  let source_with_newline: Cow<[u8]> = if should_append_newline {
    let mut owned = Vec::with_capacity(original_len + 1);
    owned.extend_from_slice(source);
    owned.push(b'\n');
    Cow::Owned(owned)
  } else {
    Cow::Borrowed(source)
  };
  let original_endpoint =
    should_append_newline.then(|| (original_len, point_for_byte(source, original_len)));

  (source_with_newline, original_endpoint)
}

pub fn remap_range_for_appended_newline(
  range: Range,
  original_endpoint: &Option<EndPoint>,
) -> Range {
  let Some((end_byte, end_point)) = original_endpoint else {
    return range;
  };

  if range.end_byte < *end_byte {
    return range;
  }

  Range {
    start_byte: range.start_byte,
    start_point: range.start_point,
    end_byte: *end_byte,
    end_point: *end_point,
  }
}

pub fn remap_injected_region_highlight_range(
  injection_range: &Range,
  highlight_range: &Range,
) -> Range {
  Range {
    start_byte: injection_range.start_byte + highlight_range.start_byte,
    end_byte: injection_range.start_byte + highlight_range.end_byte,
    start_point: Point {
      row: injection_range.start_point.row + highlight_range.start_point.row,
      column: injection_range.start_point.column + highlight_range.start_point.column,
    },
    end_point: Point {
      row: injection_range.end_point.row + highlight_range.end_point.row,
      column: injection_range.end_point.column + highlight_range.end_point.column,
    },
  }
}
