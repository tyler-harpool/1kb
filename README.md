# Building the World's Smallest WASM Website

**Goal:** Beat the #1 spot on [1kb.club](https://1kb.club/) — a leaderboard for websites under 1KB of total transfer weight — while actually using WebAssembly for something real.

**Lightest version:** 245 bytes. Just WASM + hash + onload. No extras.
**Current version:** 314 bytes. Adds favicon suppressor and source link.

Both powered by a 36-byte WASM binary that gates page rendering through a CPU bit-manipulation instruction.

**Live:** [tyler-harpool.github.io/1kb/#TylerHarpool](https://tyler-harpool.github.io/1kb/#TylerHarpool)

---

## The Architecture

```
URL: tyler-harpool.github.io/1kb/#TylerHarpool
                                  └── name lives here (0 bytes in HTML)

Browser loads 245-byte HTML (314 with favicon + source link)
  → onload fires
  → 36-byte WASM instantiated from inline base64
  → f(location.hash.length) → popcnt(14) = 3 (truthy)
  → body.append(location.hash) → "#TylerHarpool" rendered

No hash? → f(0) → popcnt(0) = 0 (falsy) → black void
```

The entire WASM module in WAT:

```wat
(module (func (export "f") (param i32) (result i32) local.get 0 i32.popcnt))
```

One function. One CPU instruction. 36 bytes compiled.

---

## The Journey: Every Optimization, Every Dead End

### Round 1: The Wrong Target (13KB → 86 bytes)

**Failure: `wasm32-wasip2`**

Started with Rust targeting `wasm32-wasip2`. An empty module compiled to **13KB**. The WASI component model wraps every module in adapter layers, type definitions, and canonical ABI scaffolding. For a 1KB budget, this is a non-starter.

**Fix:** Switched to `wasm32-unknown-unknown`. An empty module: **86 bytes**. The freestanding target strips all platform assumptions. This was the first real decision — abandon the "proper" WASI ecosystem for raw, minimal output.

### Round 2: Sin Approximation (869 → 133 bytes)

**Failure: Floating-point modulo**

Built a `tick()` animation function with a Taylor series `sin()` approximation for wave/glow/pulse effects. The WASM binary hit **869 bytes**. The main culprit: Rust's `%` operator on `f32` compiles to a software `fmod` implementation — a massive function the linker pulls in. One operator cost ~400 bytes.

**Fix:** Removed float modulo entirely. Let JavaScript handle range normalization. Also switched the Taylor series to Horner form for fewer instructions. WASM dropped to **133 bytes** after `wasm-opt -Oz`.

**Failure: 3-term Taylor series at pi**

The 3-term sin approximation (through x^5/120) had 0.52 error at pi, failing the 0.1 tolerance test.

**Fix:** Added a 4th term (x^7/5040). Tests passed across the range.

### Round 3: Hand-Written WAT (133 → 73 → 38 bytes)

**The Rust compiler adds bloat you can't configure away.** Even with `#![no_std]`, `opt-level = "z"`, LTO, and strip, the compiled WASM contained:

- A `memory` section (even when unused)
- `__data_end` and `__heap_base` exports
- Extra type section entries

**Fix:** Wrote the WAT by hand. Kept the Rust source only for `cargo test` (native tests, never compiled to WASM). The hand-written WAT became the source of truth for the binary. This was a key architectural decision: Rust for testing, WAT for production.

A minimal function with no memory, no globals, no unnecessary exports: **38 bytes** for `f(x) = x * x`.

Later, switching to `i32.popcnt` (population count — a single CPU instruction) dropped it to **36 bytes** because the function body shrank from 5 opcodes to 2.

### Round 4: HTML Minimization (994 → 250 bytes)

**Failure: Hex byte array encoding**

Initially encoded the WASM binary as a hex byte array in JavaScript. For a 73-byte binary, the hex representation was **365 characters**.

**Fix:** Base64 encoding. The same 73 bytes became **100 characters**. Saved 265 bytes in one change. We already had `atob()` available — free decoder built into every browser.

**Key HTML tricks that worked:**

| Technique | Bytes saved |
|---|---|
| `bgcolor=0` instead of `bgcolor=#000` | 3 |
| `W=WebAssembly;new W.Instance(new W.Module(...))` alias | 10 |
| Synchronous instantiation (no `.then()` callback) | ~20 |
| `&&document.write(...)` conditional gate | ~15 |
| No `<html>`, `<head>`, `<!DOCTYPE>` tags | ~40 |
| No closing `</body>`, `</html>` tags | ~15 |

**What didn't work: `instantiateStreaming` with data URI.** The async API requires a `.then()` chain and the data URI overhead negates any binary savings.

### Round 5: Beating #1 — The 250-Byte Wall

The previous #1, [cenzontle.us](https://cenzontle.us), sat at **252 bytes**. Our canvas animation version was 898 bytes — not close. Everything had to go.

**Decision: Strip the animation entirely.** The question became: what's the smallest meaningful thing WASM can do?

**Failure: WASM returning a constant**

First attempt at minimal: WASM exports a function that returns `1`. Page renders if truthy. **247 bytes.** But WASM was doing literally nothing — just returning a hardcoded value. That's not using WASM, that's carrying it as dead weight.

**Fix: Pure function gate.** WASM exports `f(x) = x * x`. JavaScript calls `f(13)` where 13 is the length of "Tyler Harpool". Returns 169 (truthy), page renders. `f(0)` would return 0 (falsy), page stays blank. The WASM computation has a semantic connection to the content. **250 bytes.**

### Round 6: URL Hash as Data Store (250 → 245 bytes)

Inspired by [Scott Antipa's article on storing app state in URLs](https://www.scottantipa.com/store-app-state-in-urls): the URL hash fragment is client-side only, never sent to the server, and `location.hash` is dirt cheap to access.

**The insight:** Move the name from the HTML into the URL. The name costs 0 bytes in the response because it lives in the hash fragment.

```
https://tyler-harpool.github.io/1kb/#TylerHarpool
                                     └── free, client-side only
```

Now WASM validates that content exists: `f(location.hash.length)` returns 0 for empty hash (blank page) and nonzero for present hash (render). WASM became the **content gate** — it decides whether there's anything worth displaying.

**Failure: Spaces in hash → `%20` encoding**

`location.hash` does NOT decode percent-encoding. `# Tyler Harpool` in the URL renders as `#%20Tyler%20Harpool`. Adding `decodeURI()` costs 12 bytes — more than we saved.

**Fix:** Drop the spaces. `#TylerHarpool`. Clean URL, clean display. The `#` prefix reads like a markdown heading or shell prompt on the green-on-black terminal aesthetic.

### Round 7: `onload` vs `<script>` (245 → 245 bytes, then +favicon)

**The `onload` attribute trick:** `</script>` costs 9 bytes. `">` costs 2 bytes. Moving JavaScript from a `<script>` tag into `<body onload="...">` saves **7 bytes**.

Combined with inline assignment `(h=location.hash).length` instead of a separate `h=location.hash;` declaration, we squeezed out another byte.

**Inline favicon (`<link rel=icon href=data:,>`):** Borrowed from cenzontle.us. Without it, the browser requests `/favicon.ico` — a whole extra HTTP response. The 27-byte inline data URI prevents that request entirely. Costs bytes in HTML but saves hundreds in total transfer weight.

### Round 8: The Header Problem

**1kb.club measures total transfer weight** — that includes HTTP response headers.

| Host | Header bytes | Body bytes | Total |
|---|---|---|---|
| cenzontle.us | 84 | 169 | 253 |
| GitHub Pages | 670 | 314 | 984 |
| Cloudflare Workers | ~150 | 314 | ~464 |

GitHub Pages injects 20+ headers (CDN cache metadata, security headers, Varnish/Fastly identifiers). cenzontle.us runs a minimal server with just 3 headers: `Content-Type`, `Content-Length`, `Connection`.

**Deployed to Cloudflare Workers** for header control. The worker returns only what's necessary:

```javascript
export default {
  async fetch() {
    return new Response(HTML, {
      headers: { "content-type": "text/html" },
    });
  },
};
```

---

## Size Progression

| Version | Bytes | What changed |
|---|---|---|
| WASI component model | 13,000+ | Wrong target |
| `wasm32-unknown-unknown` empty | 86 | Right target |
| sin() animation | 869 | Float modulo bloat |
| sin() optimized | 133 | Removed fmod, Horner form |
| Hand-written WAT | 73 | Stripped compiler bloat |
| Canvas animation page | 994 | First sub-1KB build |
| Base64 encoding | 924 | Hex → base64 saved 265 chars |
| Minified HTML | 898 | Attribute tricks |
| WASM gate + document.write | 250 | Stripped animation, pure function |
| **URL hash + onload + popcnt** | **245** | **Lightest build. Name in URL, smaller WASM** |
| + favicon suppressor | 272 | Prevents extra HTTP request |
| + source link (current) | 314 | GitHub link, no closing `</a>` |

---

## What We Learned

**The Rust compiler is not your friend at this scale.** Below ~200 bytes, hand-written WAT beats any compiler output. Rust's value is in testing — `cargo test` validates the logic natively while the production binary is hand-tuned.

**Base64 is the right encoding.** Every browser has `atob()`. For small binaries, base64 is more compact than hex arrays and the decoder is free.

**URL fragments are free storage.** The hash is never sent to the server. `location.hash` costs 13 characters to read. For content that doesn't need to be indexed by search engines, it's the cheapest data store available.

**HTTP headers matter more than you think.** A 314-byte page can have 670 bytes of headers on GitHub Pages. The response body is less than half the total transfer. Hosting choice is an optimization.

**Every byte has a story.** `bgcolor=0` saves 3 bytes over `bgcolor=#000`. Dropping `</a>` saves 4. Using `onload` instead of `<script>` saves 7. Using `i32.popcnt` instead of `i32.mul` saves 4 bytes of base64. None of these matter in normal web development. At 314 bytes, they're the whole game.

---

## Build It Yourself

```bash
git clone https://github.com/tyler-harpool/1kb
cd 1kb
# Requires: cargo, wasm-tools (cargo install wasm-tools)
bash build.sh
# Open dist/index.html in a browser with #YourName in the URL
```

## Stack

- **Rust** — testing only (`cargo test`)
- **WAT** — hand-written WebAssembly Text Format (source of truth)
- **wasm-tools** — WAT → WASM assembly
- **Cloudflare Workers** — minimal-header hosting
- **GitHub Pages** — CI/CD with size guard (rejects builds over 1024 bytes)
