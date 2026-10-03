#!/bin/sh
# Prints the path of a pinned mdBook binary, downloading the release archive
# for this host into target/tools/mdbook-<version>/ on first use.
set -eu

MDBOOK_VERSION=v0.5.4

root=$(cd "$(dirname "$0")/../.." && pwd)
dir="$root/target/tools/mdbook-$MDBOOK_VERSION"
bin="$dir/mdbook"

if [ ! -x "$bin" ]; then
    os=$(uname -s)
    arch=$(uname -m)
    case "$os/$arch" in
        Linux/x86_64) triple=x86_64-unknown-linux-musl ;;
        Linux/aarch64 | Linux/arm64) triple=aarch64-unknown-linux-musl ;;
        Darwin/x86_64) triple=x86_64-apple-darwin ;;
        Darwin/arm64) triple=aarch64-apple-darwin ;;
        *)
            echo "mdbook.sh: no mdBook $MDBOOK_VERSION download for $os/$arch; install it and put it on PATH, or build with cargo" >&2
            exit 1
            ;;
    esac
    url="https://github.com/rust-lang/mdBook/releases/download/$MDBOOK_VERSION/mdbook-$MDBOOK_VERSION-$triple.tar.gz"
    mkdir -p "$dir"
    tmp="$dir/download.tar.gz"
    echo "Downloading mdBook $MDBOOK_VERSION ($triple)" >&2
    curl -fsSL "$url" -o "$tmp"
    tar -xzf "$tmp" -C "$dir" mdbook
    rm -f "$tmp"
fi

echo "$bin"
