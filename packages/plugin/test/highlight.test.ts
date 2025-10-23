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
        console.log("this is a string");
        return a + b;
      }
    </code>
  </pre>
</body>
</html>`;

  const processor = rehype()
    .use(rehypeTreeSitter, {
      grammar_paths: [path.join(__dirname, "../../../fixtures/grammars/")],
    })
    .freeze();

  const output = processor.processSync(html).value;
  expect(output).matchSnapshot();
});

test("highlights markdown code with injections", () => {
  const html = `
<html>
<head></head>
<body>
  <pre>
    <code class="language-markdown">
      ## This is a title

      This is some text. Some text with \`a code block\`

      \`\`\`javascript
      console.log("And this is some javascript")
      \`\`\`

      Some more text
    </code>
  </pre>
</body>
</html>`;

  const processor = rehype()
    .use(rehypeTreeSitter, {
      grammar_paths: [path.join(__dirname, "../../../fixtures/grammars/")],
    })
    .freeze();

  const output = processor.processSync(html).value;
  expect(output).matchSnapshot();
});

test("highlights clojure code with markdown injections", () => {
  const html = `
<html>
<head></head>
<body>
  <pre>
    <code class="language-clojure">
      (defn some-function 
        "## This is markdown
       
         \`\`\`clojure
         (println 1)
         \`\`\`"
        [])
    </code>
  </pre>
</body>
</html>`;

  const processor = rehype()
    .use(rehypeTreeSitter, {
      grammar_paths: [path.join(__dirname, "../../../fixtures/grammars/")],
      query_paths: [path.join(__dirname, "../../../fixtures/queries/")],
    })
    .freeze();

  const output = processor.processSync(html).value;
  expect(output).matchSnapshot();
});

test("highlights markdown code with no trailing text", () => {
  const html = `
<html>
<head></head>
<body>
  <pre>
    <code class="language-markdown">
      ## This is markdown
      
      \`\`\`clojure
      (println 1)
      \`\`\`
    </code>
  </pre>
</body>
</html>`;

  const processor = rehype()
    .use(rehypeTreeSitter, {
      grammar_paths: [path.join(__dirname, "../../../fixtures/grammars/")],
    })
    .freeze();

  const output = processor.processSync(html).value;
  expect(output).matchSnapshot();
});
