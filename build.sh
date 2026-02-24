#!/bin/bash
set -e

echo "Running tests..."
cargo test

echo "Assembling WASM from WAT..."
mkdir -p dist
wasm-tools parse minimal.wat -o dist/onekb.wasm

B64=$(base64 -i dist/onekb.wasm | tr -d '\n')

echo "Injecting WASM into HTML template..."
sed "s|/\*W\*/|$B64|" template.html > dist/index.html

WASM_SIZE=$(wc -c < dist/onekb.wasm | tr -d ' ')
HTML_SIZE=$(wc -c < dist/index.html | tr -d ' ')
GZIP_SIZE=$(gzip -c dist/index.html | wc -c | tr -d ' ')

echo ""
echo "=== Size Report ==="
echo "WASM:           ${WASM_SIZE} bytes"
echo "HTML (final):   ${HTML_SIZE} bytes"
echo "HTML (gzipped): ${GZIP_SIZE} bytes"
echo "Budget left:    $((1024 - HTML_SIZE)) bytes (uncompressed)"
