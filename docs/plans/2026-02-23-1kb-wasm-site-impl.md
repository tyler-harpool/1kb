# 1kb WASM Personal Site Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a sub-1024-byte personal website using Rust compiled to WASM that renders an animated name on a canvas, configurable via URL params.

**Architecture:** Single HTML file with inline WASM binary as a JS typed array. Rust `#[no_std]` module exports a `tick()` function for animation math. JS reads URL params, instantiates WASM, and runs a canvas render loop. Target: `wasm32-wasip2`.

**Tech Stack:** Rust (no_std), WASM, vanilla JS, HTML5 Canvas

**Design doc:** `docs/plans/2026-02-23-1kb-wasm-site-design.md`

---

### Task 1: Project scaffolding

**Files:**
- Create: `Cargo.toml`
- Create: `src/lib.rs` (empty placeholder)
- Create: `.cargo/config.toml`

**Step 1: Initialize Cargo project as a library**

Run: `cargo init --lib /Users/THarpool/Code/personel/1kb`

**Step 2: Configure Cargo.toml for minimal WASM**

Replace `Cargo.toml` with:

```toml
[package]
name = "onekb"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]

[profile.release]
opt-level = "z"
lto = true
strip = true
codegen-units = 1
panic = "abort"
```

**Step 3: Set default target**

Create `.cargo/config.toml`:

```toml
[build]
target = "wasm32-wasip2"
```

**Step 4: Verify it compiles**

Run: `cargo build --release`
Expected: Compiles successfully, produces a `.wasm` file in `target/wasm32-wasip2/release/`

**Step 5: Check initial WASM size**

Run: `ls -l target/wasm32-wasip2/release/onekb.wasm`
Expected: A small .wasm file (likely ~100-200 bytes for an empty cdylib)

**Step 6: Commit**

```bash
git add Cargo.toml src/lib.rs .cargo/config.toml
git commit -m "Scaffold Rust WASM project with size-optimized profile"
```

---

### Task 2: Implement the tick() function in Rust

**Files:**
- Modify: `src/lib.rs`

**Step 1: Write a test for tick()**

Add to `src/lib.rs`:

```rust
#![no_std]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

/// Compute animation values for a character.
///
/// Arguments:
/// - time: current time in ms (from requestAnimationFrame)
/// - index: character index in the string
/// - effect: 0=wave, 1=glow, 2=pulse
/// - speed: animation speed multiplier (1-9)
///
/// Returns a packed u64:
/// - upper 32 bits: y_offset as f32 bits
/// - lower 32 bits: brightness (0-255) as u32
#[no_mangle]
pub extern "C" fn tick(time: f32, index: f32, effect: i32, speed: f32) -> u64 {
    let t = time * speed * 0.001;
    let i = index * 0.5;

    let (y, brightness) = match effect {
        // wave: characters bob up and down in a sine wave
        0 => {
            let y = sin(t + i) * 20.0;
            let b = ((sin(t * 2.0 + i) + 1.0) * 0.5 * 127.0 + 128.0) as u32;
            (y, b)
        }
        // glow: characters pulse brightness
        1 => {
            let b = ((sin(t * 3.0 + i) + 1.0) * 0.5 * 200.0 + 55.0) as u32;
            (0.0, b)
        }
        // pulse: characters scale/bounce
        _ => {
            let y = sin(t * 2.0 + i).abs() * -15.0;
            let b = ((sin(t + i * 0.3) + 1.0) * 0.5 * 127.0 + 128.0) as u32;
            (y, b)
        }
    };

    let y_bits = y.to_bits() as u64;
    let b = brightness.min(255) as u64;
    (y_bits << 32) | b
}

/// Minimal sine approximation using Taylor series.
/// Accurate enough for visual animation, tiny code size.
fn sin(x: f32) -> f32 {
    // Normalize to [-PI, PI]
    let pi = 3.14159265;
    let mut x = x % (2.0 * pi);
    if x > pi {
        x -= 2.0 * pi;
    } else if x < -pi {
        x += 2.0 * pi;
    }
    // Taylor series: sin(x) ≈ x - x^3/6 + x^5/120
    let x3 = x * x * x;
    let x5 = x3 * x * x;
    x - x3 / 6.0 + x5 / 120.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tick_wave_returns_packed_value() {
        let result = tick(1000.0, 0.0, 0, 3.0);
        let y_bits = (result >> 32) as u32;
        let brightness = (result & 0xFFFFFFFF) as u32;
        let y = f32::from_bits(y_bits);
        // y should be in range [-20, 20]
        assert!(y >= -20.0 && y <= 20.0, "y_offset out of range: {}", y);
        // brightness should be in range [128, 255]
        assert!(brightness >= 55 && brightness <= 255, "brightness out of range: {}", brightness);
    }

    #[test]
    fn test_tick_glow_y_is_zero() {
        let result = tick(500.0, 2.0, 1, 5.0);
        let y_bits = (result >> 32) as u32;
        let y = f32::from_bits(y_bits);
        assert!((y - 0.0).abs() < 0.001, "glow y should be 0, got {}", y);
    }

    #[test]
    fn test_tick_pulse_y_is_negative_or_zero() {
        let result = tick(750.0, 1.0, 2, 2.0);
        let y_bits = (result >> 32) as u32;
        let y = f32::from_bits(y_bits);
        assert!(y <= 0.001, "pulse y should be <= 0, got {}", y);
    }

    #[test]
    fn test_sin_approximation() {
        let pi = 3.14159265;
        assert!((sin(0.0)).abs() < 0.01);
        assert!((sin(pi / 2.0) - 1.0).abs() < 0.01);
        assert!((sin(pi)).abs() < 0.05);
        assert!((sin(-pi / 2.0) + 1.0).abs() < 0.01);
    }

    #[test]
    fn test_different_speeds_produce_different_results() {
        let r1 = tick(1000.0, 0.0, 0, 1.0);
        let r2 = tick(1000.0, 0.0, 0, 5.0);
        assert_ne!(r1, r2, "different speeds should produce different results");
    }
}
```

**Step 2: Run tests**

Run: `cargo test`
Expected: All 5 tests pass. Tests run with the host target (not wasm), which is fine for pure math.

**Step 3: Build WASM and check size**

Run: `cargo build --release && ls -l target/wasm32-wasip2/release/onekb.wasm`
Expected: WASM file should be small (target: under 300 bytes)

**Step 4: Commit**

```bash
git add src/lib.rs
git commit -m "Implement tick() animation function with sin approximation"
```

---

### Task 3: Install wasm-opt and create build script

**Files:**
- Create: `build.sh`

**Step 1: Install binaryen (provides wasm-opt)**

Run: `brew install binaryen`
Expected: `wasm-opt --version` works after install

**Step 2: Create the build script**

Create `build.sh`:

```bash
#!/bin/bash
set -e

# Build WASM
cargo build --release

WASM="target/wasm32-wasip2/release/onekb.wasm"

# Optimize WASM for size
wasm-opt -Oz "$WASM" -o dist/onekb.wasm

# Dump WASM bytes as JS array
BYTES=$(xxd -i dist/onekb.wasm | grep '0x' | tr -d '\n ')

echo "WASM size (optimized): $(wc -c < dist/onekb.wasm | tr -d ' ') bytes"
echo "WASM bytes: $BYTES"

# Read HTML template and inject WASM bytes
mkdir -p dist

# Generate final HTML by injecting WASM bytes into template
sed "s|/\*WASM_BYTES\*/|$BYTES|" template.html > dist/index.html

# Report sizes
echo ""
echo "=== Size Report ==="
echo "WASM (optimized): $(wc -c < dist/onekb.wasm | tr -d ' ') bytes"
echo "HTML (final):     $(wc -c < dist/index.html | tr -d ' ') bytes"
echo "HTML (gzipped):   $(gzip -c dist/index.html | wc -c | tr -d ' ') bytes"
echo "Budget remaining: $((1024 - $(wc -c < dist/index.html | tr -d ' '))) bytes"
```

**Step 3: Make it executable**

Run: `chmod +x build.sh`

**Step 4: Commit**

```bash
git add build.sh
git commit -m "Add build script with wasm-opt and size reporting"
```

---

### Task 4: Create the HTML template

**Files:**
- Create: `template.html`

**Step 1: Write the minimal HTML template**

Create `template.html`. This is the unminified version for readability. The build script injects WASM bytes at the `/*WASM_BYTES*/` placeholder.

```html
<!DOCTYPE html><html><head><meta name=viewport content="width=device-width"><title>.</title><style>*{margin:0}canvas{width:100vw;height:100vh;background:#000}</style></head><body><canvas id=c></canvas><script>
var p=new URLSearchParams(location.search),
n=p.get('n')||'hi',
c=p.get('c')||'fff',
s=+(p.get('s')||3),
e='wgp'.indexOf((p.get('e')||'w')[0]),
v=document.getElementById('c'),
x=v.getContext('2d'),
w=new Uint8Array([/*WASM_BYTES*/]),
m;
v.width=innerWidth;v.height=innerHeight;
WebAssembly.instantiate(w).then(r=>{m=r.instance.exports;requestAnimationFrame(d)});
function d(t){x.clearRect(0,0,v.width,v.height);x.font='bold 48px sans-serif';x.textAlign='center';
var r=parseInt(c.substr(0,2)||'ff',16),g=parseInt(c.substr(2,2)||'ff',16),b=parseInt(c.substr(4,2)||'ff',16);
for(var i=0;i<n.length;i++){var k=m.tick(t,i,e<0?0:e,s),yb=k>>32n,br=Number(k&0xFFFFFFFFn),y=new Float32Array(new Uint32Array([Number(yb)]).buffer)[0],f=br/255;
x.fillStyle='rgb('+~~(r*f)+','+~~(g*f)+','+~~(b*f)+')';x.fillText(n[i],v.width/2+(i-n.length/2)*30,v.height/2+y)}requestAnimationFrame(d)}
</script></body></html>
```

**Step 2: Verify template size (without WASM bytes)**

Run: `wc -c template.html`
Expected: Under 700 bytes (the WASM bytes placeholder adds more but the total must stay under ~900 to leave room for HTTP overhead)

**Step 3: Commit**

```bash
git add template.html
git commit -m "Add HTML template with canvas renderer and URL param parsing"
```

---

### Task 5: End-to-end build and size optimization

**Files:**
- Modify: `src/lib.rs` (if needed to shrink WASM)
- Modify: `template.html` (if needed to shrink HTML)
- Modify: `build.sh` (if pipeline needs tweaks)

**Step 1: Run the full build**

Run: `mkdir -p dist && bash build.sh`
Expected: Builds successfully, prints size report

**Step 2: Check total size against budget**

Read the size report output. If `dist/index.html` is over ~800 bytes (leaving 200+ for HTTP headers), optimize:

Optimization levers (in order):
1. Shorten JS variable names further
2. Remove whitespace from HTML template
3. Simplify the `tick()` function (fewer match arms = smaller WASM)
4. Use shorter param defaults
5. Reduce sin() Taylor terms from 3 to 2

**Step 3: Test in browser**

Run: `python3 -m http.server 8080 -d dist`
Open: `http://localhost:8080/?n=Tyler+Harpool&c=ff6600&s=3&e=wave`
Expected: Animated text "Tyler Harpool" in orange with wave effect

Also test:
- `http://localhost:8080/` (defaults: "hi" in white, wave)
- `http://localhost:8080/?n=Test&e=glow&c=00ff00` (green glow)
- `http://localhost:8080/?n=Pulse&e=pulse&s=7` (fast pulse)

**Step 4: Measure transfer size**

Run: `curl -so /dev/null -w '%{size_download}' http://localhost:8080/`
Expected: Under 1024 bytes

**Step 5: Commit**

```bash
git add -A
git commit -m "Complete build pipeline, verify sub-1kb output"
```

---

### Task 6: GitHub Pages deployment setup

**Files:**
- Create: `.github/workflows/deploy.yml`
- Or: configure `dist/` as GitHub Pages source

**Step 1: Create GitHub Actions workflow**

Create `.github/workflows/deploy.yml`:

```yaml
name: Deploy to GitHub Pages

on:
  push:
    branches: [main]

permissions:
  contents: read
  pages: write
  id-token: write

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-wasip2
      - name: Install binaryen
        run: sudo apt-get install -y binaryen
      - name: Build
        run: bash build.sh
      - name: Upload artifact
        uses: actions/upload-pages-artifact@v3
        with:
          path: dist

  deploy:
    needs: build
    runs-on: ubuntu-latest
    environment:
      name: github-pages
      url: ${{ steps.deployment.outputs.page_url }}
    steps:
      - name: Deploy to GitHub Pages
        id: deployment
        uses: actions/deploy-pages@v4
```

**Step 2: Commit**

```bash
git add .github/workflows/deploy.yml
git commit -m "Add GitHub Pages deployment workflow"
```

---

### Task 7: Final verification and size audit

**Step 1: Run cargo tests one final time**

Run: `cargo test`
Expected: All tests pass

**Step 2: Run full build**

Run: `bash build.sh`
Expected: Size report shows HTML under 800 bytes

**Step 3: Visual test all effects**

Test each combination in browser via `python3 -m http.server 8080 -d dist`

**Step 4: Run DebugBear check**

Navigate to https://www.debugbear.com/test/website-speed and test the deployed URL
Expected: Total weight under 1024 bytes

**Step 5: Final commit and tag**

```bash
git tag v1.0.0
```
