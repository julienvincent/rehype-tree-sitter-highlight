platform := `uname -s | tr '[:upper:]' '[:lower:]'`
arch := `uname -m`
target := if platform == "darwin" { if arch == "arm64" { "aarch64-apple-darwin" } else { "x86_64-apple-darwin" } } else if platform == "linux" { if arch == "aarch64" { "aarch64-unknown-linux-gnu" } else { "x86_64-unknown-linux-gnu" } } else { "unknown" }
pkg_dir := "packages/highlight-arch"

build:
    cargo build --release
    pnpm run -r build

build-and-copy: build
    if [ "{{ platform }}" = "darwin" ]; then \
        libfile="target/release/libhighlight.dylib"; \
        dest="{{ pkg_dir }}/darwin-{{ arch }}/index.node"; \
    elif [ "{{ platform }}" = "linux" ]; then \
        if [ "{{ arch }}" = "aarch64" ]; then \
            libfile="target/release/libhighlight.so"; \
            dest="{{ pkg_dir }}/linux-arm64-gnu/index.node"; \
        else \
            libfile="target/release/libhighlight.so"; \
            dest="{{ pkg_dir }}/linux-x64-gnu/index.node"; \
        fi; \
    elif [[ "{{ platform }}" =~ "mingw" ]] || [[ "{{ platform }}" =~ "msys" ]]; then \
        libfile="target/release/highlight.dll"; \
        dest="{{ pkg_dir }}/windows-x64/index.node"; \
    fi; \
    mkdir -p "$(dirname "$dest")"; \
    cp "$libfile" "$dest"

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

test:
    pnpm run -r test
