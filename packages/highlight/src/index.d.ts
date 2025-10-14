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

export type HighlightParams = {
  source: string;
  language: string;
  query_paths?: string[];
  grammar_paths?: string[];
};

export function highlight(params: HighlightParams): HighlightEvent[];

declare const tree_sitter_highlight: {
  highlight: typeof highlight;
  HighlightEventType: typeof HighlightEventType;
};

export default tree_sitter_highlight;
