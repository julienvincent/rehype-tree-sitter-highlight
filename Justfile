platform := `uname -s | tr '[:upper:]' '[:lower:]'`
arch := `uname -m`
target := if platform == "darwin" { if arch == "arm64" { "aarch64-apple-darwin" } else { "x86_64-apple-darwin" } } else if platform == "linux" { if arch == "aarch64" { "aarch64-unknown-linux-gnu" } else { "x86_64-unknown-linux-gnu" } } else { "unknown" }
pkg_dir := "packages/highlight-arch"

build-lib profile="release":
    cargo build --profile {{ profile }}

build-and-copy-lib profile="release": (build-lib profile)
    if [ "{{ profile }}" = "release" ]; then \
      profile="release"; \
    else \
      profile="debug"; \
    fi; \
    if [ "{{ platform }}" = "darwin" ]; then \
        libfile="target/$profile/libhighlight.dylib"; \
        dest="{{ pkg_dir }}/darwin-{{ arch }}/index.node"; \
    elif [ "{{ platform }}" = "linux" ]; then \
        if [ "{{ arch }}" = "aarch64" ]; then \
            libfile="target/$profile/libhighlight.so"; \
            dest="{{ pkg_dir }}/linux-arm64-gnu/index.node"; \
        else \
            libfile="target/$profile/libhighlight.so"; \
            dest="{{ pkg_dir }}/linux-x64-gnu/index.node"; \
        fi; \
    elif [[ "{{ platform }}" =~ "mingw" ]] || [[ "{{ platform }}" =~ "msys" ]]; then \
        libfile="target/$profile/highlight.dll"; \
        dest="{{ pkg_dir }}/windows-x64/index.node"; \
    fi; \
    mkdir -p "$(dirname "$dest")"; \
    cp "$libfile" "$dest"

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
