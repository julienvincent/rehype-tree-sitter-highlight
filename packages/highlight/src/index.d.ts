export const enum HighlightEventType {
  Start,
  Source,
  End,
}

export type HighlightRange = {
  start: number;
  end: number;
};

export type HighlightEvent =
  | {
      type: HighlightEventType.Start;
      highlight: string;
    }
  | {
      type: HighlightEventType.End;
    }
  | {
      type: HighlightEventType.Source;
      range: HighlightRange;
    };

export class Highlighter {
  constructor(grammar_paths: string[], query_paths?: string[]);
  highlight(source: String, language: String): HighlightEvent[];
}

declare const tree_sitter_highlight: {
  Highlighter: typeof Highlighter;
  HighlightEventType: typeof HighlightEventType;
};

export default tree_sitter_highlight;
