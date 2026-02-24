# 1kb WASM Personal Site Design

## Goal

Build a personal website under 1,024 bytes total transfer size for submission to 1kb.club. Written in Rust, compiled to WASM, hosted on GitHub Pages.

## Concept

A configurable WASM rendering engine that displays an animated name on a canvas. All personalization lives in URL parameters — the page itself is a tiny generic renderer.

**Default URL:** `?n=Tyler+Harpool&c=ff6600&s=3&e=wave`

## URL Parameters

| Param | Purpose        | Default |
|-------|----------------|---------|
| `n`   | Display name   | `hi`    |
| `c`   | Base color hex  | `fff`   |
| `s`   | Speed (1-9)    | `3`     |
| `e`   | Effect         | `wave`  |

Effects: `wave`, `glow`, `pulse`

## Architecture

Single HTML file with inline WASM binary as a JS `Uint8Array`. No external resources.

### Byte Budget (~500-700 bytes uncompressed)

- HTML + canvas + style: ~80 bytes
- JS param parser + WASM loader + render loop: ~200 bytes
- Inline WASM binary (typed array): ~150-250 bytes
- Buffer: ~200+ bytes margin for GitHub Pages transfer overhead

### Data Flow

```
URL params -> JS parser -> config {name, color, speed, effect}
                              |
                              v
                       WASM instantiate
                              |
                              v
                  requestAnimationFrame loop
                       |           |
                  time + idx -> WASM tick()
                                   |
                                   v
                            [y, r, g, b]
                                   |
                                   v
                         canvas fillText()
```

### WASM Module

One exported function: `tick(time, index, effect, speed) -> packed result`

Returns y_offset and RGB color. Pure math (sin waves, color cycling). No std, no alloc, no imports.

### Rust Configuration

- Target: `wasm32-unknown-unknown`
- `#[no_std]`, `#[no_mangle]`
- `opt-level = "z"`, `lto = true`, `strip = true`
- Post-process with `wasm-opt -Oz`

### Build Pipeline

1. `cargo build --release --target wasm32-unknown-unknown`
2. `wasm-opt -Oz` on the output
3. Custom script dumps WASM bytes as JS array literal
4. Inject into HTML template
5. Minify final HTML

## Error Handling

- No URL params: falls back to defaults
- Invalid params: JS `||` fallback, no validation code
- No WASM/canvas support: blank page (acceptable)
- No error handling in WASM (pure math, cannot fail)

## Testing

- Build script outputs final byte count
- `curl -so /dev/null -w '%{size_download}'` against local server
- DebugBear Page Speed Analyzer (1kb.club's measurement tool)
- Visual check with various param combinations
- Rust unit tests on `tick()` function
- GitHub Pages deploy + actual transfer size measurement

## Hosting

GitHub Pages. Minimal headers overhead. Custom domain optional.

## Measurement

1kb.club uses DebugBear to measure total transfer weight including HTTP headers. GitHub Pages adds headers that consume part of the budget.
