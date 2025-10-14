import { expect, test } from "vitest";

import { fileURLToPath } from "node:url";
import path from "node:path";

import rehypeTreeSitter from "../src/index.ts";
import { rehype } from "rehype";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

test("highlights javascript code", () => {
  const html = `
<html>
<head></head>
<body>
  <pre>
    <code class="language-javascript">
      function sum(a, b) {
        return a + b;
      }
    </code>
  </pre>
</body>
</html>`;

  const processor = rehype()
    .use(rehypeTreeSitter, {
      grammar_paths: [path.join(__dirname, "../../../fixtures/grammars/")],
      query_paths: [
        path.join(__dirname, "../../../fixtures/nvim-treesitter/queries/"),
      ],
    })
    .freeze();

  const output = processor.processSync(html).value;
  expect(output).matchSnapshot();
});
