# Tech Stack Comparison for 2D Nesting Application on macOS (Apple Silicon)

## Executive Summary

After extensive research, the **recommended stack** is:

**Primary recommendation: Rust (egui + wgpu) + OpenCASCADE via FFI (opencascade-sys)**

**Runner-up: Rust nesting engine (via PyO3) + Python GUI (PySide6/Qt) + CadQuery for STEP**

The rationale and full analysis follow below.

---

## 1. Platform: Apple Silicon (ARM64) Considerations

### Rust on Apple Silicon
- Rust compiles natively for `aarch64-apple-darwin` with first-class support.
- ARM NEON SIMD is mandatory on all AArch64 CPUs and is available by default — no feature flags needed.
- With `target-cpu=apple-m1` and LTO enabled, benchmarks show **30-45% performance boosts** over default compiler settings.
- `std::simd` (nightly) and the `wide`/`pulp` crates provide portable SIMD abstractions; hand-written NEON intrinsics are best for interleaved data patterns.
- Compile times are competitive: incremental builds on M-series chips are fast. An M4 Max compiles large Rust projects ~2x faster than an M2 Max.
- **Verdict: Excellent.** Rust is one of the best choices for Apple Silicon performance.

### C++ on Apple Silicon
- OpenCASCADE officially tests on Apple M1 (ARM64) and passes with minor deviations (~150 test failures out of thousands, mostly floating-point precision).
- Build time is 1.5-1.7x faster on M1 vs older Intel i5.
- Available via `brew install opencascade`.
- **Verdict: Excellent.** Mature and well-tested.

### Python on Apple Silicon
- Python 3.11+ has native arm64 installers. NumPy, SciPy, etc. all have native wheels.
- CadQuery's `cadquery-ocp` now ships native `osx-arm64` wheels for Python 3.10-3.13.
- pythonocc may still require Rosetta 2 for some conda packages (20-30% performance penalty).
- **Verdict: Good for GUI/glue, poor for compute-heavy nesting loops.**

---

## 2. Nesting Engine Performance Analysis

The nesting algorithm (NFP computation, genetic algorithm, collision detection) is the most performance-critical component. This is a **CPU-bound, algorithmic workload** with millions of collision queries per second.

### Existing Nesting Tools and Their Stacks

| Tool | Language | Architecture | Performance |
|------|----------|-------------|-------------|
| **sparrow/jagua-rs** | Rust | Pure Rust, CLI | State-of-the-art. ~2.4M samples/sec on complex instances. Outperforms all published academic results. |
| **Deepnest** | JavaScript (Electron) + C addon | Electron app, genetic algorithm + NFP | Functional but slow. Speed-critical NFP code in C. |
| **SVGnest** | JavaScript | Browser-based, pure JS | Accessible but slow for large problems. |
| **libnest2d** | C++ | Header-only library, Boost Geometry backend | Moderate performance. Used in Cura/PrusaSlicer. Still "not feature complete." |
| **u-nesting** | Rust | Rayon parallelism, NFP-based | Newer project. Supports holes and curves. |

### Language Performance for Nesting

- **Rust vs C++**: Within 5-10% of each other in general benchmarks. Rust's memory safety eliminates entire classes of bugs without runtime cost.
- **Python**: Orders of magnitude slower for tight loops. Even with NumPy, Python cannot match Rust/C++ for collision detection queries that run millions of times per second.
- **PyO3 overhead**: ~22 microseconds per FFI call. This means you **must batch work** — calling a Rust function per collision query from Python would be catastrophic. But calling Rust to "run 1000 iterations of the genetic algorithm" is fine.

### Recommendation for Nesting Engine
**Rust is the clear winner.** jagua-rs and sparrow prove that Rust can deliver state-of-the-art nesting performance. The entire jagua-rs collision detection engine and sparrow optimizer are MIT-licensed and can serve as a foundation or reference.

---

## 3. STEP File Support

This is the **hardest constraint** and significantly narrows the viable options.

### OpenCASCADE (C++ — the gold standard)
- The only mature, open-source B-Rep kernel with comprehensive STEP AP203/AP214/AP242 support.
- Battle-tested over decades. Used by FreeCAD, CadQuery, pythonocc, and virtually all open-source CAD.
- Supports `BRepAlgoAPI_Section` to slice 3D solids with a plane and extract 2D profiles — exactly what's needed.
- Builds on Apple Silicon via Homebrew: `brew install opencascade`.

### Accessing OpenCASCADE from Rust

| Crate | Status | STEP Support | Maturity |
|-------|--------|-------------|----------|
| **opencascade-rs** (bschwind) | Work in progress, hobby project | STEP is a stated goal; low-level `opencascade-sys` FFI exists | **Low.** API in flux, one maintainer working part-time. |
| **opencascade-sys** | Provides raw FFI bindings via cxx.rs | Can call OCCT's STEPControl_Reader/Writer through unsafe FFI | **Usable but requires manual wrapping.** |
| **occt-sys** | Static build of OCCT as a Rust dependency | Provides the C++ library; you write your own bindings | **Build infrastructure only.** |
| **truck** | Pure Rust CAD kernel | STEP I/O via companion `ruststep` project. Active development. Can read assemblies, write cylinders/shells. | **Moderate.** Progressing but not yet on par with OCCT for complex STEP files. |

### Accessing OpenCASCADE from Python

| Library | Status | Apple Silicon | STEP Support |
|---------|--------|--------------|-------------|
| **CadQuery** | Mature, active | Native ARM64 wheels available (cadquery-ocp 7.9.3) | Full STEP read/write via OCCT |
| **pythonocc-core** | Mature | Conda packages exist; ARM64 status unclear (may need Rosetta) | Full STEP read/write via OCCT |
| **OCP** (OpenCascade Python) | CadQuery's OCCT wrapper | ARM64 wheels on PyPI | Direct OCCT bindings |

### Accessing OpenCASCADE from C++
- Direct, native access. No bindings needed.
- Maximum control and the full OCCT API surface.
- But requires managing C++ build complexity (CMake, linking).

### Recommendation for STEP
- **Safest path**: Use CadQuery/OCP from Python, or OpenCASCADE directly from C++.
- **Rust path**: Use `opencascade-sys` for low-level FFI calls to OCCT's STEP reader/writer. This works but requires writing unsafe wrapper code. Alternatively, shell out to a small Python/C++ helper for STEP operations.
- **truck** is promising but not yet reliable enough for arbitrary industrial STEP files.

---

## 4. GUI Framework Comparison

### Rust GUI Options

#### egui/eframe
- **Architecture**: Immediate mode GUI. Renders via wgpu (Metal on macOS).
- **2D Rendering**: Built-in `Painter` API for custom drawing (lines, convex polygons, paths). Anti-aliased rendering.
- **Zoom/Pan**: No built-in pan/zoom container (open feature request #1811). Must implement manually via `RectTransform`. Well-documented pattern.
- **Maturity**: Pre-1.0, API still evolving. But very actively developed with huge community (30k+ GitHub stars).
- **CAD precedent**: MuginCAD is a functional 2D CAD app built with egui.
- **Packaging**: Compiles to a single native binary. Use `cargo-bundle` for .app, though some Apple Silicon bundling issues reported.
- **Pros**: Fastest to prototype, huge community, excellent wgpu integration, MIT license.
- **Cons**: No built-in zoom/pan widget, immediate mode can get messy for complex UIs, API instability.

#### iced
- **Architecture**: Retained mode, Elm-like (model-update-view). Renders via wgpu.
- **2D Rendering**: Canvas widget designed for "2D drawing, charts, CAD applications."
- **Zoom/Pan**: Canvas widget supports transformations.
- **Maturity**: v0.14 (Dec 2025). Used by COSMIC desktop. Pre-1.0 but rapidly maturing.
- **Pros**: Strong architecture for complex apps, wgpu-native, good state management. MIT license.
- **Cons**: Smaller widget ecosystem, steeper learning curve, less documentation than egui.

#### Slint
- **Architecture**: Declarative DSL (`.slint` markup files). Multiple renderers (femtovg/Skia/software/wgpu).
- **2D Rendering**: WGPU integration added in v1.12. OpenGL underlay/texture for custom rendering.
- **Maturity**: **1.x stable release** — the only Rust GUI framework with API stability commitment.
- **Zoom/Pan**: Would need custom implementation via the rendering integration.
- **Licensing**: **Royalty-free for desktop apps** (free). GPL for open source. Commercial license for embedded.
- **Pros**: API stability, professional backing (Slint GmbH), polished look, Figma integration.
- **Cons**: DSL adds a learning curve, custom rendering integration historically harder than egui/iced, commercial entity controls direction.

#### Tauri + Web Frontend
- **Architecture**: Rust backend + web frontend (React/Svelte) in OS WebView.
- **2D Rendering**: Use HTML Canvas or WebGL in the frontend. Or attempt wgpu directly (experimental, not yet working visually on macOS).
- **Performance**: 30-40MB memory (vs 200-300MB Electron). Sub-500ms startup.
- **Packaging**: Excellent. Built-in .app bundling, code signing, notarization, DMG creation, App Store support.
- **Pros**: Best packaging/distribution story, familiar web stack for UI, tiny bundles.
- **Cons**: WebView rendering has performance ceiling for complex 2D scenes. wgpu-in-Tauri is experimental and broken on macOS. Web Canvas is not ideal for high-performance CAD rendering with thousands of polygons.

### Python GUI Options

#### PySide6/PyQt6 (Qt)
- **2D Rendering**: QGraphicsScene/QGraphicsView — designed for exactly this use case. Handles millions of items, built-in zoom/pan/rotate, anti-aliased polygon rendering.
- **Maturity**: Extremely mature. Qt has decades of development. QPainter can use OpenGL for hardware acceleration.
- **Packaging**: `py2app` or `briefcase` for .app bundles. More complex than Rust single-binary.
- **Pros**: Best out-of-the-box 2D scene graph for polygon rendering with zoom/pan. Massive ecosystem, excellent documentation.
- **Cons**: Python runtime overhead, larger distribution size, GIL limitations for threading.

#### VisPy / ModernGL
- **2D Rendering**: GPU-accelerated via OpenGL. VisPy handles tens of millions of data points in real-time.
- **Maturity**: VisPy is mature for scientific visualization. ModernGL is a simpler OpenGL wrapper.
- **Pros**: Very high performance for 2D rendering, good for scientific applications.
- **Cons**: Not a full GUI framework — need to combine with Qt or similar. OpenGL is being deprecated by Apple in favor of Metal.

### C++ GUI Options

#### Qt (C++)
- **2D Rendering**: Same QGraphicsScene/QGraphicsView as PySide6, but native C++ performance.
- **OCCT Integration**: Well-documented. occQt6 project demonstrates Qt6 + OCCT 7.5+ on macOS.
- **Pros**: Maximum performance, direct OCCT access, mature ecosystem.
- **Cons**: C++ complexity, slower development iteration, more build system complexity (CMake + Qt + OCCT).

---

## 5. Stack Combinations Evaluated

### Option A: Pure Rust (egui + wgpu + opencascade-sys)
```
Nesting Engine: Rust (jagua-rs/sparrow as reference)
GUI: egui/eframe with custom wgpu 2D renderer
STEP: opencascade-sys FFI to OCCT
Packaging: cargo-bundle + codesign
```
**Pros:**
- Single language, single binary
- Maximum nesting performance
- Excellent Apple Silicon optimization (NEON, LTO)
- Small binary size

**Cons:**
- opencascade-sys requires writing unsafe FFI wrapper code for STEP operations
- egui's zoom/pan needs manual implementation
- No API stability guarantee from egui
- C++ OCCT dependency complicates the build (CMake + Cargo)

**Risk: Medium.** The STEP FFI is the main risk. Everything else is well-proven.

---

### Option B: Rust + Tauri + Web Frontend
```
Nesting Engine: Rust (Tauri backend)
GUI: Svelte/React + HTML Canvas or Three.js for 2D view
STEP: opencascade-sys in Tauri backend, or shell out to Python
Packaging: Tauri built-in (excellent)
```
**Pros:**
- Best packaging and distribution story
- Familiar web development for UI
- Clean separation of concerns
- Small bundle size (3-10MB)

**Cons:**
- 2D rendering via WebView Canvas has performance ceiling
- wgpu-in-Tauri is broken on macOS (experimental)
- Two-language complexity (Rust + JS/TS)
- WebView inconsistencies across OS versions

**Risk: Medium-High.** The 2D rendering performance through WebView is the main concern for a CAD-like application with potentially thousands of polygons, zoom, and pan.

---

### Option C: Rust Engine + Python GUI (PyO3 + PySide6 + CadQuery)
```
Nesting Engine: Rust library exposed via PyO3
GUI: PySide6 (Qt6) with QGraphicsScene for 2D rendering
STEP: CadQuery/OCP (native ARM64 wheels)
Packaging: py2app or briefcase for .app
```
**Pros:**
- QGraphicsScene is purpose-built for interactive 2D polygon rendering with zoom/pan
- CadQuery provides the most battle-tested, easiest STEP handling on Apple Silicon
- PyO3 is mature and well-documented
- Rapid UI iteration in Python
- spyrrow (Python wrapper for sparrow) proves this architecture works

**Cons:**
- PyO3 FFI overhead (~22 microseconds per call; must batch)
- Python packaging is more complex (bundling interpreter + dependencies)
- Larger distribution size
- Two-language build system (Cargo + pip/maturin)
- GIL can complicate threading (use `Python::detach` in PyO3)

**Risk: Low-Medium.** All components are proven. The main downsides are packaging complexity and the two-language overhead.

---

### Option D: Pure C++ (Qt + OpenCASCADE + custom nesting)
```
Nesting Engine: C++ (port jagua-rs concepts or use libnest2d)
GUI: Qt6 with QGraphicsView
STEP: OpenCASCADE directly
Packaging: CMake + macdeployqt
```
**Pros:**
- Direct OCCT access, no bindings overhead
- Qt + OCCT is a proven combination (occQt6)
- Maximum control over everything
- libnest2d provides a starting point for nesting

**Cons:**
- C++ development is significantly slower than Rust or Python
- Memory safety issues (no borrow checker)
- libnest2d is incomplete and less performant than jagua-rs
- Would need to reimplement sparrow-level algorithms from scratch in C++
- Build system complexity (CMake + Qt + OCCT)

**Risk: Medium.** Proven technology, but high development effort and C++ foot-guns.

---

### Option E: Rust Engine + iced GUI + opencascade-sys
```
Nesting Engine: Rust
GUI: iced with Canvas widget
STEP: opencascade-sys FFI
Packaging: cargo-bundle
```
**Pros:**
- iced's Canvas widget was designed for "2D drawing, charts, CAD applications"
- Elm architecture scales well for complex state management
- Single language
- wgpu-native rendering

**Cons:**
- Smaller ecosystem than egui
- Same OCCT FFI challenges as Option A
- Pre-1.0, though rapidly maturing
- Less documentation and fewer examples

**Risk: Medium.** Similar profile to Option A but with better architecture for complex apps and slightly less community support.

---

### Option F: Hybrid — Rust Engine + Python STEP helper + egui GUI
```
Nesting Engine: Rust (core binary)
GUI: egui/eframe
STEP: Subprocess call to a small Python script using CadQuery
     (or use opencascade-sys for read, CadQuery for complex operations)
Packaging: Bundle Python as a sidecar or use a pre-processing step
```
**Pros:**
- Gets the best of both worlds: Rust performance + CadQuery STEP reliability
- STEP import/export is not a hot path (done once per file load/save)
- Avoids the hardest part of pure-Rust STEP (writing full OCCT FFI wrappers)

**Cons:**
- Bundling a Python interpreter as a sidecar adds complexity and size
- Two runtimes in one application
- Less elegant architecture

**Risk: Medium.** Pragmatic but architecturally messy.

---

## 6. Detailed Component Analysis

### 6.1 opencascade-rs / opencascade-sys
- **Status**: Work in progress. One maintainer (bschwind), part-time hobby project.
- **STEP support**: The low-level `opencascade-sys` crate can call OCCT's `STEPControl_Reader` and `STEPControl_Writer` through unsafe FFI. The high-level wrapper is incomplete.
- **Build**: Uses `occt-sys` which builds OCCT from source as a static library. This works but adds significant compile time.
- **Apple Silicon**: OCCT builds fine on ARM64. The Rust FFI layer should work.
- **Assessment**: Usable for STEP read/write if you're willing to write raw FFI code against OCCT's C++ API. Not a polished, ready-to-use solution.

### 6.2 truck (Rust CAD kernel)
- **Status**: Active development by ricosjp. Pure Rust, modular crate design.
- **STEP support**: Via companion `ruststep` project (EXPRESS parser + STEP toolkit). Can read assemblies, write some geometry types. Active development on STEP I/O in changelog.
- **Limitations**: Not yet on par with OCCT for reading arbitrary industrial STEP files. May fail on complex assemblies or unusual AP214 entities.
- **Assessment**: Promising for the future but **not ready for production STEP handling** of arbitrary industrial files today.

### 6.3 CadQuery on Apple Silicon
- **Status**: Native ARM64 wheels available via `cadquery-ocp` on PyPI (v7.9.3, Feb 2026).
- **Python versions**: 3.10-3.13 supported on `macosx_11_0_arm64`.
- **STEP handling**: Full read/write. Based on OCCT. Includes `BRepAlgoAPI_Section` for cross-sectioning.
- **Assessment**: **The easiest, most reliable path to STEP support on Apple Silicon today.**

### 6.4 Tauri vs Electron (2025-2026)
| Metric | Tauri | Electron |
|--------|-------|----------|
| Memory (idle) | 30-40 MB | 200-300 MB |
| Bundle size | 3-10 MB | 60-150 MB |
| Startup time | < 500ms | 1-2 seconds |
| Rendering | OS WebView | Bundled Chromium |
| Backend language | Rust | Node.js |
| Ecosystem maturity | Growing (Tauri 2.0, late 2024) | Very mature |
| macOS packaging | Excellent (built-in signing, notarization, DMG) | Good |
| Custom rendering (wgpu) | Experimental, broken on macOS | N/A (uses Chromium) |

For a nesting app: Tauri's WebView-based rendering is the bottleneck. HTML Canvas can handle hundreds of polygons smoothly but may struggle with thousands during real-time zoom/pan at 60fps. Electron has the same limitation since both use web rendering.

### 6.5 Metal GPU Access
- **From Rust**: wgpu uses Metal as first-class backend on macOS. Also `metal-rs` for direct Metal API. `rust-gpu` can compile Rust to SPIR-V, which wgpu translates to MSL.
- **From Python**: `wgpu-py` provides Python bindings to wgpu (which uses Metal on macOS).
- **Relevance to nesting**: The nesting algorithm itself is CPU-bound (collision detection, genetic algorithm). GPU could accelerate rendering of the nested layout visualization, but this is not the bottleneck. GPU-accelerated NFP computation is theoretically possible but no existing implementation does this.

---

## 7. Build/Packaging Comparison

| Stack | .app Bundle | Code Signing | Complexity |
|-------|------------|-------------|------------|
| Rust (egui) | cargo-bundle (some ARM64 issues) | Manual codesign/notarize | Medium |
| Rust (Tauri) | Built-in, excellent | Built-in with env vars | **Low** |
| Rust (iced/Slint) | cargo-bundle | Manual | Medium |
| Python (PySide6) | py2app or briefcase | Manual | **High** (bundling interpreter) |
| C++ (Qt) | macdeployqt | Manual | Medium-High |

---

## 8. Community & Documentation

| Framework | GitHub Stars | Documentation | Community Size |
|-----------|-------------|---------------|---------------|
| egui | ~30k | Good, many examples | Large |
| iced | ~15k | Growing (new book in 0.14) | Medium |
| Slint | ~20k | Excellent (commercial backing) | Medium |
| Tauri | ~90k | Excellent | Very Large |
| PySide6/Qt | N/A (Qt) | Excellent (decades of docs) | Very Large |
| jagua-rs | ~500 | Academic paper + README | Small (niche) |
| sparrow | ~200 | Paper + reproducible experiments | Small (niche) |
| opencascade-rs | ~500 | Minimal | Small |
| CadQuery | ~3k | Good | Medium |

---

## 9. Final Recommendation

### For Maximum Performance & Elegance: **Rust (egui + wgpu) + opencascade-sys**
Choose this if you:
- Are comfortable writing some unsafe FFI code for STEP operations
- Want a single-language, single-binary solution
- Prioritize nesting engine performance above all
- Can tolerate egui's API instability

### For Fastest Time-to-Market & Lowest STEP Risk: **Rust Engine (PyO3) + Python GUI (PySide6) + CadQuery**
Choose this if you:
- Want the most proven STEP handling (CadQuery on ARM64)
- Value QGraphicsScene's built-in polygon zoom/pan
- Are comfortable with two languages and more complex packaging
- Want to leverage spyrrow/sparrow as a reference for the PyO3 bridge pattern

### For Best Distribution Experience: **Rust (Tauri) + Svelte + opencascade-sys backend**
Choose this if you:
- Prioritize polished distribution (code signing, DMG, auto-update)
- The 2D view has moderate complexity (hundreds, not thousands of polygons)
- Want a familiar web frontend

### Not Recommended:
- **Pure Python nesting engine**: Too slow by orders of magnitude for competitive nesting.
- **Pure C++**: Development speed penalty is significant, and Rust matches C++ performance.
- **truck for STEP**: Not mature enough for arbitrary industrial STEP files.
- **Electron**: No advantage over Tauri in 2026. Larger, slower, more memory.

---

## 10. Suggested Architecture (Recommended Stack A)

```
┌─────────────────────────────────────────┐
│              egui/eframe App             │
│  ┌───────────────┐  ┌────────────────┐  │
│  │  UI Controls  │  │  2D Viewport   │  │
│  │  (egui panels,│  │  (custom wgpu  │  │
│  │   buttons,    │  │   renderer or  │  │
│  │   settings)   │  │   epaint with  │  │
│  │               │  │   RectTransform│  │
│  └───────────────┘  │   zoom/pan)    │  │
│                     └────────────────┘  │
├─────────────────────────────────────────┤
│           Application Logic             │
│  ┌────────────────┐ ┌────────────────┐  │
│  │ Nesting Engine │ │  STEP Handler  │  │
│  │ (jagua-rs CDE  │ │ (opencascade-  │  │
│  │  + custom GA/  │ │  sys FFI to    │  │
│  │  optimizer)    │ │  OCCT for      │  │
│  │                │ │  read/write/   │  │
│  │                │ │  section)      │  │
│  └────────────────┘ └────────────────┘  │
├─────────────────────────────────────────┤
│  Dependencies:                          │
│  - jagua-rs (MIT) — collision detection │
│  - occt-sys — static OCCT build        │
│  - egui/eframe — GUI                    │
│  - wgpu — GPU rendering (Metal backend) │
└─────────────────────────────────────────┘
```

### Key Implementation Notes:

1. **Nesting Engine**: Use jagua-rs as the collision detection backend. Implement your own optimizer on top (similar to sparrow's feasibility-decomposition approach). Both are MIT-licensed.

2. **STEP Read**: Call OCCT's `STEPControl_Reader` via `opencascade-sys`. Extract 2D profiles using `BRepAlgoAPI_Section` (slice 3D solid with XY plane). This is ~50-100 lines of unsafe FFI code.

3. **STEP Write**: Call OCCT's `STEPControl_Writer` to export the nested arrangement. Transform each part's position/rotation and write back to STEP.

4. **2D Rendering**: Use egui's `Painter` with `RectTransform` for zoom/pan. Draw polygons with `Shape::convex_polygon()` and `Shape::Path` for concave shapes. For very large part counts, drop down to raw wgpu for instanced rendering.

5. **Packaging**: Use `cargo-bundle` for .app, manually `codesign` and `notarytool` for distribution. Or wrap in Tauri just for the packaging layer (use Tauri's build system but render via egui in a separate window).

---

## Sources

### Rust GUI Frameworks
- [egui GitHub](https://github.com/emilk/egui)
- [MuginCAD — 2D CAD with egui](https://github.com/Hakkology/MuginCAD)
- [iced 0.14 Release](https://www.phoronix.com/news/Iced-0.14-Rust-GUI-LIbrary)
- [Slint — Declarative GUI](https://slint.dev/)
- [Slint Pricing/Licensing](https://slint.dev/pricing)
- [Rust GUI Libraries Compared 2025](https://an4t.com/rust-gui-libraries-compared/)
- [State of Rust GUI](https://weeklyrust.substack.com/p/the-state-of-rust-gui-the-good-and)
- [Tritium: Slint Desktop Experience](https://tritium.legal/blog/desktop)
- [egui Pan/Zoom Feature Request](https://github.com/emilk/egui/issues/1811)

### Nesting Algorithms
- [jagua-rs — Collision Detection Engine](https://github.com/JeroenGar/jagua-rs)
- [sparrow — State-of-the-Art Nesting](https://github.com/JeroenGar/sparrow)
- [sparrow Paper (arXiv)](https://arxiv.org/html/2509.13329v2)
- [spyrrow — Python Wrapper for Sparrow](https://github.com/PaulDL-RS/spyrrow)
- [Deepnest Architecture (DeepWiki)](https://deepwiki.com/deepnest-next/deepnest)
- [SVGnest](https://github.com/Jack000/SVGnest)
- [libnest2d](https://github.com/tamasmeszaros/libnest2d)
- [u-nesting](https://github.com/iyulab/u-nesting)

### STEP / CAD Kernels
- [opencascade-rs](https://github.com/bschwind/opencascade-rs)
- [truck — Rust CAD Kernel](https://github.com/ricosjp/truck)
- [truck STEP I/O Issue](https://github.com/ricosjp/truck/issues/2)
- [CadQuery](https://github.com/CadQuery/cadquery)
- [cadquery-ocp on PyPI](https://pypi.org/project/cadquery-ocp/)
- [CadQuery Apple Silicon Issue](https://github.com/CadQuery/cadquery/issues/891)
- [OpenCASCADE STEP Translator Docs](https://dev.opencascade.org/doc/overview/html/occt_user_guides__step.html)
- [OCCT Section Operation (Forum)](https://dev.opencascade.org/content/using-plane-slice-solid-and-extract-single-face)
- [OCCT on Apple M1](https://dev.opencascade.org/content/testing-occt-apple-m1-arm64)

### Rust on Apple Silicon / SIMD
- [Rust SIMD Benchmark on Apple M4](https://dev.to/erioharrison/rust-simd-benchmark-stdsimd-vs-neon-on-apple-m4-47jd)
- [Optimizing Rust for Apple Silicon M3](https://markaicode.com/rust-apple-silicon-m3-optimization/)
- [State of SIMD in Rust 2025](https://shnatsel.medium.com/the-state-of-simd-in-rust-in-2025-32c263e5f53d)
- [Rust Compile Times on Apple Silicon](https://users.rust-lang.org/t/rust-development-on-apple-silicon-m4-can-we-test-the-incremental-build-speed/121262)

### Tauri / Packaging
- [Tauri vs Electron Comparison](https://www.gethopp.app/blog/tauri-vs-electron)
- [Tauri macOS Code Signing](https://v2.tauri.app/distribute/sign/macos/)
- [Tauri macOS Application Bundle](https://v2.tauri.app/distribute/macos-application-bundle/)
- [Shipping Production macOS App with Tauri 2.0](https://dev.to/massi_24/shipping-a-production-macos-app-with-tauri-20-code-signing-notarization-and-homebrewpublished-o10)
- [cargo-bundle](https://github.com/burtonageo/cargo-bundle)

### GPU / Rendering
- [wgpu — Portable Graphics for Rust](https://wgpu.rs/)
- [Metal from Rust (Lambda Class Blog)](https://blog.lambdaclass.com/using-metal-and-rust-to-make-fft-even-faster/)
- [rust-gpu](https://github.com/Rust-GPU/rust-gpu)
- [wgpu-tauri-experiment](https://github.com/dceddia/wgpu-tauri-experiment)

### PyO3 / Python Integration
- [PyO3 Performance Guide](https://pyo3.rs/main/performance)
- [PyO3 GitHub](https://github.com/PyO3/pyo3)
- [Making Python 100x Faster with Rust](https://ohadravid.github.io/posts/2023-03-rusty-python/)

### Python GUI / Rendering
- [PySide6 QGraphicsView Tutorial](https://www.pythonguis.com/tutorials/pyside6-qgraphics-vector-graphics/)
- [PyQtGraph](https://pyqtgraph.com/)
- [VisPy](https://vispy.org/)
