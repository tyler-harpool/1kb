# How I Tried to Build the Smallest WASM Website on the Internet

There's a site called [1kb.club](https://1kb.club/) that ranks the lightest websites on the internet by total transfer weight. The #1 spot belongs to [cenzontle.us](https://cenzontle.us) at 252 bytes. I wanted to see if I could beat it — and I wanted to do it with WebAssembly, which is arguably the worst possible technology choice for this goal.

This is the story of every dumb thing I tried, what actually worked, and the mass of dead ends in between.

**Lightest build:** 245 bytes
**Current build:** 314 bytes (with a favicon trick and a source link)
**Live:** [tyler-harpool.github.io/1kb/#TylerHarpool](https://tyler-harpool.github.io/1kb/#TylerHarpool)

---

## Why WASM?

Honestly, because it's a terrible idea. WASM has overhead — module headers, section encoding, type definitions. Cramming it into a sub-300 byte page is solving a problem nobody has. But that's kind of the point. cenzontle.us is 169 bytes of clean HTML and CSS. No JavaScript, no WASM, no nonsense. It's elegant. I wanted to see if you could get anywhere close while carrying the weight of a compiled binary.

Spoiler: you can, but you'll question your life choices along the way.

## Round 1: Picking the Wrong Target

I started with Rust compiled to `wasm32-wasip2` because that's what the docs suggest. An empty module — literally nothing in it — compiled to **13KB**. Thirteen thousand bytes. For nothing. The WASI component model wraps everything in adapter layers and canonical ABI scaffolding. It's the right choice for real applications. It's the wrong choice when your entire budget is 1,024 bytes.

Switching to `wasm32-unknown-unknown` got an empty module down to **86 bytes**. Still felt like a lot for doing nothing, but at least we were in the right order of magnitude.

## Round 2: The Animation That Wouldn't Fit

The original plan was ambitious: a canvas animation with wave effects, glowing text, pulsing colors. I implemented a Taylor series `sin()` approximation in Rust. The WASM binary came out to **869 bytes**. The entire page budget is 1,024.

The biggest surprise was where the bytes went. Rust's `%` operator on floating point numbers compiles to a full software `fmod` implementation. One modulo operator cost roughly **400 bytes**. A single `%`. I stared at the disassembly for a while.

Removing the modulo (and letting JavaScript handle range normalization instead) plus switching to Horner form for the polynomial got WASM down to **133 bytes** after `wasm-opt -Oz`. Still too big for what I was after, but I learned something important: at this scale, you can't just write Rust and hope the optimizer saves you. You need to know what every line compiles to.

I also learned that a 3-term Taylor series falls apart at pi. The error was 0.52 against a 0.1 tolerance. Had to add the x^7 term. Ask me how long I spent debugging that before checking the math.

## Round 3: Giving Up on the Compiler

Even with `#![no_std]`, `opt-level = "z"`, LTO, and strip, the Rust compiler adds things you didn't ask for:

- A `memory` section (I wasn't using memory)
- `__data_end` and `__heap_base` exports (I didn't want exports)
- Extra type section padding

About 36 bytes of pure overhead on a function that was 5 bytes of actual logic.

So I gave up on compiling Rust to WASM. Instead, I wrote the WAT (WebAssembly Text Format) by hand and kept the Rust source around only for `cargo test`. The tests run natively — they never touch WASM. The hand-tuned WAT is what actually ships.

This felt like admitting defeat, but it dropped the binary from 133 bytes to **73 bytes**, and later to **38 bytes** for a simple `f(x) = x * x` function. The final version uses `i32.popcnt` (population count — counts the set bits in an integer) which got it to **36 bytes** because the function body is just two opcodes.

The entire WASM module:

```wat
(module (func (export "f") (param i32) (result i32) local.get 0 i32.popcnt))
```

That's it.

## Round 4: Minifying the HTML

The first page that worked came in at **994 bytes**. Sub-1KB, technically, but not competitive. Here's what I learned about HTML golf:

`bgcolor=0` works. The browser interprets `0` as black. Saves 3 bytes over `bgcolor=#000`. I don't know why this works and I'm afraid to look it up in case it's undefined behavior.

Base64 is better than hex. I was encoding the WASM binary as a hex byte array (365 characters for 73 bytes). Switching to base64 and `atob()` got the same data into 100 characters. The decoder is free — every browser has it.

Synchronous WASM instantiation is shorter than async. `new WebAssembly.Instance(new WebAssembly.Module(...))` looks verbose, but it avoids the `.then()` callback chain. Aliasing `W=WebAssembly` saves a few more.

You don't need `<!DOCTYPE>`, `<html>`, `<head>`, `<body>`, or any closing tags. The browser figures it out. This isn't news to anyone who's done code golf, but it still felt wrong.

The `instantiateStreaming` API with a data URI seemed promising but wasn't. The async overhead and data URI prefix cost more than they save.

## Round 5: The Existential Crisis at 250 Bytes

To actually beat cenzontle.us at 252, I had to strip the canvas animation entirely. No effects, no color cycling, no sin waves. The question became: what's the smallest meaningful thing WASM can do?

My first answer was embarrassing. I made WASM return `1`. That's it. The function took no input, did no computation, just returned a constant. The page checked if the result was truthy and rendered my name. **247 bytes.** Five bytes under the leader.

Then I looked at it and thought: this isn't a WASM site. This is an HTML site with a WASM binary duct-taped to the side doing nothing. If you removed the WASM entirely and just wrote `document.write('Tyler Harpool')`, the page would be smaller.

So I gave WASM a real job. The function `f(x) = x * x` takes the length of my name (13) and returns 169. Truthy → page renders. Pass in 0 → returns 0 → blank page. It's a gate. The computation is tied to the content. Not the most impressive use of a compiled binary, but it's honest. **250 bytes.**

## Round 6: Storing My Name in the URL

This one came from reading [Scott Antipa's post about storing app state in URL hash fragments](https://www.scottantipa.com/store-app-state-in-urls). The hash fragment (`#whatever`) is never sent to the server. It's purely client-side. And `location.hash` is 13 characters to read in JavaScript.

So I moved my name out of the HTML and into the URL:

```
tyler-harpool.github.io/1kb/#TylerHarpool
```

The name costs **0 bytes** in the response. The browser already has it. WASM now validates that a hash exists — `f(location.hash.length)` returns 0 when there's no hash (blank page) and nonzero when there is (render the name). The WASM function went from decorative to functional: it decides whether there's content worth displaying.

I immediately hit a wall with spaces. `# Tyler Harpool` in the URL becomes `#%20Tyler%20Harpool` on screen because `location.hash` doesn't decode percent-encoding. `decodeURI()` costs 12 bytes. So I just dropped the spaces. `#TylerHarpool`. On a green-on-black terminal page, the `#` prefix looks like a shell prompt. I'm calling it a feature.

## Round 7: Death by a Thousand Bytes

Small wins that added up:

Moving JavaScript from `<script>...</script>` into `<body onload="...">` saves **7 bytes** because `">` is shorter than `</script>`.

Inline assignment `(h=location.hash).length` instead of `h=location.hash;...h.length` saves **1 byte** by eliminating the semicolon and the separate statement.

Dropping `</a>` on the source link saves **4 bytes**. The browser auto-closes it. cenzontle.us does this too — they never close their `<a>` tag either.

`<link rel=icon href=data:,>` prevents the browser from requesting `/favicon.ico`. Costs 27 bytes in HTML but prevents an entire extra HTTP round trip. cenzontle.us taught me this one.

## Round 8: The Part I Didn't Expect

After all that HTML and WASM optimization, the biggest problem turned out to be **HTTP headers**.

1kb.club measures total transfer weight. That includes response headers. I checked:

- **cenzontle.us:** 84 bytes of headers. Three headers total.
- **GitHub Pages:** 670 bytes of headers. Twenty-something headers. CDN metadata, cache directives, Varnish identifiers, Fastly request IDs.

My 314-byte page had 670 bytes of headers wrapped around it. The headers were twice the size of the content. The hosting platform was the bottleneck, not the code.

cenzontle.us runs on what appears to be a custom server that returns exactly three headers: `Content-Type`, `Content-Length`, `Connection`. That's 84 bytes. Minimalism all the way down.

This is an unsolved problem for me. I deployed to Cloudflare Workers for more header control, but that domain got blocked by my network's Jamf security policy. The GitHub Pages version works but carries the header tax. I'm still looking for the right minimal hosting setup.

---

## Where It Landed

| Version | Bytes | Notes |
|---|---|---|
| WASI target | 13,000+ | Wrong target entirely |
| Empty `wasm32-unknown-unknown` | 86 | Starting point |
| sin() animation | 869 | One `%` operator cost 400 bytes |
| sin() optimized | 133 | Removed fmod, Horner form |
| Hand-written WAT | 73 | Compiler adds ~36 bytes of bloat |
| First working page | 994 | Sub-1KB with canvas animation |
| Base64 encoding | 924 | Hex → base64 saved 265 chars |
| Stripped to pure function | 250 | Killed the animation, kept the WASM honest |
| **URL hash + onload + popcnt** | **245** | **Lightest. Name in URL, WASM validates content** |
| + favicon suppressor | 272 | Prevents extra HTTP request |
| + source link | 314 | Current deployed version |

The lightest version is 245 bytes of HTML wrapping a 36-byte WASM binary that runs a single CPU instruction. Whether that counts as "using WebAssembly" is debatable. Whether it was worth the effort is definitely not — it wasn't. But I learned more about WASM binary encoding, HTML parsing quirks, HTTP header overhead, and the cost of a single floating-point modulo than I would have in a month of normal development.

If you want to poke at it: [github.com/tyler-harpool/1kb](https://github.com/tyler-harpool/1kb)

```bash
git clone https://github.com/tyler-harpool/1kb && cd 1kb
bash build.sh  # needs cargo + wasm-tools
# open dist/index.html#YourName in a browser
```
