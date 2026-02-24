#!/bin/bash
set -e

echo "Running tests..."
cargo test

echo "Assembling WASM from WAT..."
mkdir -p dist
wasm-tools parse plasma.wat -o dist/onekb.wasm

B64=$(base64 -i dist/onekb.wasm | tr -d '\n')

echo "Injecting WASM into HTML template..."
sed "s|/\*W\*/|$B64|" template.html > dist/index.html

WASM_SIZE=$(wc -c < dist/onekb.wasm | tr -d ' ')
HTML_SIZE=$(wc -c < dist/index.html | tr -d ' ')
GZIP_SIZE=$(gzip -c dist/index.html | wc -c | tr -d ' ')
BR_SIZE=$(brotli -c dist/index.html | wc -c | tr -d ' ')

echo ""
echo "=== Size Report ==="
echo "WASM:             ${WASM_SIZE} bytes"
echo "HTML (raw):       ${HTML_SIZE} bytes"
echo "HTML (gzip):      ${GZIP_SIZE} bytes"
echo "HTML (brotli):    ${BR_SIZE} bytes"
echo "Budget (raw):     $((1024 - HTML_SIZE)) bytes remaining"
echo "Budget (brotli):  $((1024 - BR_SIZE)) bytes remaining"
echo ""
echo "Serve locally: miniserve dist --index index.html -p 8080"
