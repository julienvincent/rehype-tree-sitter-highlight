import highlight from "@julienvincent/tree-sitter-highlight";
import { visit } from "unist-util-visit";
import type { Element, ElementContent } from "hast";

export type HighlighterOptions = {
  enter?: (node: Element) => boolean;
  leave?: (node: Element) => void;
  resolveQueryPath?: (node: string) => string;
  highlight_mapping?: Record<string, string>;
  grammar_paths?: string[];
  query_paths?: string[];
};

function extractLanguage(class_name: string[]): string | null {
  return class_name.reduce((acc: string | null, value) => {
    if (acc) {
      return acc;
    }
    const match = /^language-(.*)/.exec(value);
    if (!match) {
      return acc;
    }

    const [_, lang] = match;
    return lang;
  }, null);
}

function parseMeta(meta?: any): Record<string, string | boolean> {
  if (typeof meta !== "string") {
    return {};
  }

  return Object.fromEntries(
    meta.split(" ").map((entry) => {
      const [key, value] = entry.split("=");
      let cleaned = value || true;
      if (typeof cleaned === "string") {
        cleaned = cleaned.replaceAll('"', "");
      }
      return [key, cleaned];
    }),
  );
}

function resetContentOffset(content: string): [string, number] {
  const lines = content.split("\n");
  if (lines.length === 0) {
    return ["", 0];
  }

  while (lines[0].trim() === "") {
    lines.shift();
  }
  while (lines[lines.length - 1].trim() === "") {
    lines.pop();
  }

  if (lines.length === 0) {
    return ["", 0];
  }

  const [first] = lines;
  const index = first.search(/\S/);

  lines.push("");

  if (index === 0) {
    return [lines.join("\n"), 0];
  }

  const trimmed = lines
    .map((line) => {
      const offset_total = line.search(/\S/);
      if (offset_total > index) {
        return line.substring(index);
      }
      return line.substring(offset_total);
    })
    .join("\n");

  return [trimmed, index];
}

function highlightsSinceReset(stack: string[]) {
  const highlights: string[] = [];

  const reversed = Array.from(stack);
  reversed.reverse();
  for (const highlight of reversed) {
    if (highlight === "none") {
      return highlights;
    }

    highlights.unshift(highlight);
  }

  return Array.from(new Set(highlights));
}

export default function rehypeCodeTreeSitter(options?: HighlighterOptions) {
  const grammar_paths = options?.grammar_paths || [];
  const default_query_paths = Array.from(options?.query_paths || []);
  const highlighter = new highlight.Highlighter(
    grammar_paths,
    default_query_paths,
  );

  return function transformer(tree: Element) {
    visit(
      tree,
      { type: "element", tagName: "code" },
      (node, _index, parent?: Element) => {
        if (parent?.tagName !== "pre") {
          return;
        }

        const maybe_class_name = node.properties.className;
        if (!Array.isArray(maybe_class_name)) {
          return null;
        }
        const class_name = maybe_class_name.filter((element) => {
          return typeof element === "string";
        });

        const lang = extractLanguage(class_name);
        if (!lang) {
          return;
        }

        const query_paths = [];
        const meta = parseMeta((node.data as any)?.meta);
        if (meta.query && typeof meta.query === "string") {
          const query_path = options?.resolveQueryPath?.(meta.query);
          if (query_path) {
            query_paths.push(query_path);
          }
        }

        const [child] = node.children;
        if (!child || child.type !== "text") {
          return;
        }

        if (options?.enter) {
          if (!options.enter(node)) {
            return;
          }
        }

        const [source] = resetContentOffset(child.value);

        // This is not the best way to do this. Ideally there is a way to
        // specify temporary queries that are loaded for a single call to
        // highlight without needing to recreate the entire highlighter.
        let local_highlighter = highlighter;
        if (query_paths.length > 0) {
          local_highlighter = new highlight.Highlighter(
            grammar_paths,
            default_query_paths.concat(query_paths),
          );
        }
        const events = local_highlighter.highlight(source, lang);

        const children: ElementContent[] = [];
        const highlights_stack: string[] = [];
        for (const event of events) {
          switch (event.type) {
            case highlight.HighlightEventType.Start: {
              const highlight = event.highlight;
              const mapped_highlight =
                options?.highlight_mapping?.[highlight] || highlight;
              highlights_stack.push(mapped_highlight);
              break;
            }
            case highlight.HighlightEventType.End: {
              highlights_stack.pop();
              break;
            }

            case highlight.HighlightEventType.Source: {
              const subtext = source.substring(
                event.range.start,
                event.range.end,
              );

              const stack = highlightsSinceReset(highlights_stack);
              if (stack.length > 0) {
                children.push({
                  type: "element",
                  tagName: "span",
                  properties: {
                    className: stack.pop(),
                  },
                  children: [
                    {
                      type: "text",
                      value: subtext,
                    },
                  ],
                });
              } else {
                children.push({
                  type: "element",
                  tagName: "span",
                  properties: {},
                  children: [
                    {
                      type: "text",
                      value: subtext,
                    },
                  ],
                });
              }

              break;
            }
          }
        }

        // Trim off any trailing newline nodes
        if (children.length > 0) {
          const last = children[children.length - 1];
          switch (last.type) {
            case "text": {
              if (last.value === "\n") {
                children.pop();
              }
              break;
            }
            case "element": {
              if (
                last.children[0]?.type === "text" &&
                last.children[0].value === "\n"
              ) {
                children.pop();
                break;
              }
            }
          }
        }

        node.children = children;

        options?.leave?.(node);
      },
    );
  };
}
