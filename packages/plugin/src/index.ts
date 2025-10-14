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

export default function rehypeCodeTreeSitter(options?: HighlighterOptions) {
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

        const query_paths = Array.from(options?.query_paths || []);

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

        let events;
        try {
          events = highlight.highlight({
            language: lang,
            source: child.value,
            query_paths,
            grammar_paths: options?.grammar_paths || [],
          });
        } catch (e) {
          console.error("Failed to highlight", e);
          return;
        }

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
              const subtext = child.value.substring(
                event.range.start,
                event.range.end,
              );

              if (highlights_stack.length > 0) {
                children.push({
                  type: "element",
                  tagName: "span",
                  properties: {
                    className: Array.from(highlights_stack),
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
                  type: "text",
                  value: subtext,
                });
              }

              break;
            }
          }
        }

        node.children = children;

        options?.leave?.(node);
      },
    );
  };
}
