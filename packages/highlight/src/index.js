import { createRequire } from "node:module";

const { platform, arch } = process;
const require = createRequire(import.meta.url);
const binding = require(
  `@julienvincent/tree-sitter-highlight-${platform}-${arch}`,
);

export default binding;
