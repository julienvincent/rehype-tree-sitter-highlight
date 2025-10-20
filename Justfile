platform := `uname -s | tr '[:upper:]' '[:lower:]'`
default_target := `rustc -vV | sed -n 's/^host: //p'`

build-lib profile="release" target=(default_target):
    cargo build --profile {{ profile }} --target {{ target }}

build-and-copy-lib profile="release" target=(default_target): (build-lib profile target)
    #!/bin/bash
    profile="{{ profile }}";
    if [ "{{ profile }}" = "dev" ]; then
      profile="debug";
    fi;

    if [ "{{ target }}" = "aarch64-apple-darwin" ]; then
        ext="dylib";
        dest="darwin-arm64";
    elif [ "{{ target }}" = "x86_64-apple-darwin" ]; then
        ext="dylib";
        dest="darwin-x64";
    elif [ "{{ target }}" = "aarch64-unknown-linux-gnu" ]; then
        ext="so";
        dest="linux-arm64-gnu";
    elif [ "{{ target }}" = "x86_64-unknown-linux-gnu" ]; then
        ext="so";
        dest="linux-x64-gnu";
    fi;

    if [ "${dest}" = "" ]; then
      echo "Unknown target {{ target }}"
      exit 1
    fi;

    cp target/{{ target }}/$profile/libhighlight.$ext packages/highlight-arch/$dest/index.node

build: build-and-copy-lib
    pnpm run -r build

download-test-fixtures:
    #!/bin/sh
    rm -rf fixtures
    mkdir -p fixtures

    git clone --no-checkout --depth=1 --filter=tree:0 https://github.com/nvim-treesitter/nvim-treesitter fixtures/nvim-treesitter
    cd fixtures/nvim-treesitter
    git sparse-checkout set --no-cone /queries
    git checkout

    cd ../../

    mkdir -p fixtures/grammars

    git clone --depth=1 https://github.com/tree-sitter/tree-sitter-javascript fixtures/grammars/javascript
    git clone https://github.com/tree-sitter-grammars/tree-sitter-markdown --depth 1 fixtures/grammars/markdown
    git clone https://github.com/sogaiu/tree-sitter-clojure --depth 1 fixtures/grammars/clojure

test:
    pnpm run -r test
    cargo test -p rehype-tree-sitter-highlight
