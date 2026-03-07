# 2D Nesting Research for Thetta Project

## Project Goal (from IDEA.md)
Import STEP file + thickness/dimensions/separation -> extract matching solids -> flatten to 2D -> nest/pack into rectangles -> export STEP files.

This is a **2D irregular bin packing problem** (also called "nesting" or "irregular strip packing").

---

## 1. Problem Classification

The nesting problem belongs to the family of **Cutting & Packing (C&P)** problems. Specifically:
- **2D Irregular Strip Packing Problem (2DISPP)**: Pack irregular shapes into a strip of fixed width, minimizing length
- **2D Irregular Bin Packing Problem (2DIBPP)**: Pack irregular shapes into fixed-size bins, minimizing number of bins

Our project is the **2DIBPP variant** — we have fixed-size rectangles (x * y) and need to fill as few as possible.

This is NP-hard. No polynomial-time exact solution exists. All practical solutions use heuristics or meta-heuristics.

---

## 2. Key Algorithms

### 2.1 No-Fit Polygon (NFP) — The Core Geometric Tool

The NFP is the fundamental geometric concept for nesting. Given two polygons A and B, the NFP defines the locus of points where B's reference point can be placed such that A and B touch but don't overlap.

**How it works:**
1. Fix polygon A in place
2. Slide polygon B around A's boundary (orbiting)
3. Track B's reference point — this trace is the NFP
4. Any placement of B's reference point *inside* the NFP means overlap; *outside* means no overlap; *on the boundary* means touching

**Key papers:**
- Burke et al. 2007 — "Complete and robust no-fit polygon generation" (orbital method, handles all edge cases) — **DOWNLOADED**
- Rocha 2019 — "Robust NFP generation for nesting problems" (simplified, handles numerical precision) — **DOWNLOADED**

**Inner Fit Polygon (IFP):** The dual concept — defines where a piece can be placed *inside* a container without exceeding boundaries.

### 2.2 Bottom-Left Fill (BLF) Placement Heuristic

The standard placement strategy:
1. Sort pieces by some criterion (area, bounding box, etc.)
2. For each piece, find the bottom-left-most feasible position using NFP
3. Place it there

Variants: Bottom-Left (BL), Bottom-Left-Fill (BLF), Lowest-Gravity-Center

**Key paper:** Burke et al. 2006 — "A New Bottom-Left-Fill Heuristic Algorithm"

### 2.3 Meta-heuristics for Optimization

The two-level approach dominates:
- **Outer level:** Meta-heuristic determines piece ordering and rotation angles
- **Inner level:** Placement heuristic (BLF + NFP) evaluates each candidate

Common meta-heuristics used:
| Method | Strengths |
|--------|-----------|
| **Genetic Algorithm (GA)** | Good global search, easy to parallelize |
| **Simulated Annealing (SA)** | Good at escaping local optima |
| **GA + LP hybrid** | LP compaction after GA placement — best published results |
| **Tabu Search** | Systematic neighborhood exploration |
| **Particle Swarm (PSO)** | Alternative swarm intelligence |

### 2.4 Alternative Geometric Approaches

- **Trigonometric collision detection** (used by jagua-rs/sparrow) — avoids NFP entirely, uses direct trigonometric queries. Faster for some cases.
- **Raster/pixel methods** — discretize shapes into grids. Simpler but less precise.
- **Phi-functions** — mathematical functions defining non-overlap conditions. Used in exact MIP formulations.
- **Minkowski sums** — related to NFP (NFP of convex shapes = Minkowski difference)

### 2.5 Emerging: Learning-Based Methods

- **GFPack++** (2024) — uses diffusion models with attention to learn gradient fields that push pieces toward compact configurations. Outperforms SVGnest on benchmarks. — **REPO + PAPER DOWNLOADED**

---

## 3. Downloaded Research Papers

Located in `research/papers/`:

| File | Paper | Year | Key Contribution |
|------|-------|------|------------------|
| `Burke_2007_Complete_Robust_NFP.pdf` | Complete and robust NFP generation | 2007 | Definitive orbital NFP algorithm |
| `Rocha_2019_Robust_NFP_Generation.pdf` | Robust NFP generation | 2019 | Simplified, numerically stable NFP |
| `Frontiers_2022_2D_Irregular_Packing_Review.pdf` | 2D irregular packing review | 2022 | Comprehensive survey of all methods |
| `Qin_2020_Adaptive_GA_SA_Nesting.pdf` | Adaptive GA + SA | 2020 | Hybrid GA+SA for irregular pieces |
| `MIP_Vertical_Slices_2023.pdf` | MIP models with vertical slices | 2023 | Exact optimization approach |
| `GFPack_2024_Learning_Gradient_Field.pdf` | GFPack++ gradient field learning | 2024 | ML-based nesting, SOTA results |
| `Quantum_Nesting_2024.pdf` | Quantum optimization for nesting | 2024 | Quantum computing exploration |
| `MergeDJD_2025.pdf` | MergeDJD constructive algorithm | 2025 | Piece merging + fast heuristic |
| `Sparrow_2025_Open_Source_Nesting.pdf` | sparrow open-source heuristic | 2025 | New SOTA, open source, Rust |
| `UMD_Sheet_Metal_Nesting.pdf` | Sheet metal nesting algorithms | — | Practical CNC focus |

**Papers that couldn't be downloaded (paywalled):**
- Bennell & Oliveira 2008 — "The geometry of nesting problems: A tutorial" (the foundational tutorial — available on ScienceDirect)
- Leao et al. 2020 — "Irregular packing problems: A review of mathematical models" (EJOR)
- Burke et al. 2006 — "A New Bottom-Left-Fill Heuristic Algorithm" (Operations Research)
- Gomes & Oliveira 2006 — "Solving Irregular Strip Packing by hybridising SA and LP" (EJOR)

---

## 4. Downloaded Software Projects

Located in `research/repos/`:

### Tier 1: Most Relevant to Our Project

| Repo | Language | Description | Relevance |
|------|----------|-------------|-----------|
| `sparrow/` | Rust | State-of-the-art nesting heuristic (2025) | Best algorithm, new SOTA results |
| `jagua-rs/` | Rust | Fast collision detection engine for nesting | Core engine behind sparrow |
| `SVGnest/` | JavaScript | Browser-based nesting (NFP + GA) | Well-documented reference implementation |
| `Deepnest/` | JS/C++ | Desktop nesting app | Most popular open-source nesting tool |
| `deepnest-next/` | JS/C/Rust | Actively maintained Deepnest fork | Latest releases (v1.5.6) |
| `libnest2d/` | C++ | Library used in PrusaSlicer/Cura | Battle-tested in production |

### Tier 2: Useful Reference Implementations

| Repo | Language | Description |
|------|----------|-------------|
| `libnfporb/` | C++ | Robust NFP generation library (orbital approach) |
| `u-nesting/` | Rust | High-perf nesting with C FFI |
| `Packaide/` | Python/C++ | Fast heuristic nesting with CGAL backend |
| `2D-Irregular-Packing-Algorithm/` | Python | Multiple algorithm implementations (SA+LP, GA, cuckoo search, BLF) |
| `2D-Irregular-Cutting-Stock-Problem/` | Python | PSO + Differential Evolution for bin cutting |
| `GFPack-pp/` | Python | ML-based nesting (diffusion model) |
| `2DIBPP-MergeDJD/` | ? | Piece merging + DJD heuristic, 515 new best-known values |

### Tier 3: Wrappers and Ports

| Repo | Language | Description |
|------|----------|-------------|
| `spyrrow/` | Python | Python wrapper for sparrow |
| `pynest2d/` | Python | Python bindings for libnest2d |
| `rust-babushka-nfp/` | Rust | NFP library for deepnest-next ecosystem |
| `FuseNest/` | Python | Fusion 360 nesting add-in based on SVGnest |

---

## 5. STEP File Processing (for our pipeline)

Our project needs to: STEP -> extract 2D profiles -> nest -> export STEP

### Libraries for STEP Import/Export
- **OpenCASCADE (OCCT)** — The gold standard open-source CAD kernel. Reads/writes STEP natively. C++ with Python bindings (pythonocc).
- **CadQuery** — Python CAD library built on OCCT. High-level API for solid modeling. Can slice solids, extract faces.
- **FreeCAD** — Uses OCCT internally. Python scriptable. Can import STEP, extract faces by thickness.
- **build123d** — Modern Python CAD library, successor to CadQuery philosophy.
- **trimesh** — Python library for 3D meshes (not STEP native, but can work with exported meshes).

### Extracting 2D Profiles from 3D Solids
The approach for our "extract solids by thickness":
1. Import STEP file using OCCT/CadQuery
2. For each solid, compute bounding box in Z (or primary axis)
3. Filter solids where Z-extent == specified thickness
4. For each matching solid, extract the largest face (top or bottom) as a 2D profile
5. These 2D profiles become the shapes to nest

### Relevant approach with CadQuery/build123d:
```python
import cadquery as cq

# Load STEP
assembly = cq.importers.importStep("input.step")

# For each solid, check thickness
for solid in assembly.solids():
    bb = solid.BoundingBox()
    thickness = bb.zlen  # or whichever axis
    if abs(thickness - target_thickness) < tolerance:
        # Extract top face as 2D profile
        top_face = solid.faces(">Z").val()
        # Convert to 2D polygon for nesting
```

---

## 6. Recommended Architecture for Thetta

Based on the research, here's the recommended pipeline:

```
STEP File Input
    |
    v
[1. STEP Parser] -- OpenCASCADE / CadQuery
    |
    v
[2. Thickness Filter] -- Filter solids by Z-extent
    |
    v
[3. Profile Extractor] -- Extract 2D face outlines
    |
    v
[4. Nesting Engine] -- sparrow/jagua-rs (best) or NFP+GA (proven)
    |
    v
[5. Rectangle Bin Packing] -- Assign nested groups to x*y bins
    |
    v
[6. STEP Exporter] -- Place original 3D solids at nested positions
```

### Technology Recommendations

**For the nesting engine, ranked by preference:**

1. **sparrow + jagua-rs** (Rust) — Current academic SOTA. Open source. Fast. But Rust-only, would need FFI bindings or rewrite pipeline in Rust.

2. **libnest2d** (C++) — Production-proven in PrusaSlicer. Good C++ API. Could bind to Python.

3. **SVGnest algorithm** (JS/Python ports) — Well-documented, many ports available. NFP + GA approach. Good starting point for understanding.

4. **Custom implementation** using NFP (Burke 2007) + GA — Full control, can optimize for our specific bin-packing variant.

**For STEP processing:**
- **CadQuery** or **build123d** (Python) — Easiest to work with, built on OCCT
- **pythonocc** — Lower-level OCCT bindings, more control

---

## 7. Key References (Not Downloaded — For Further Reading)

- Bennell & Oliveira 2008 — "The geometry of nesting problems: A tutorial" — https://www.sciencedirect.com/science/article/abs/pii/S0377221706012379
- Leao et al. 2020 — "Irregular packing problems: A review of mathematical models" — https://doi.org/10.1016/j.ejor.2019.04.045
- Burke et al. 2006 — "A New Bottom-Left-Fill Heuristic" — https://pubsonline.informs.org/doi/10.1287/opre.1060.0293
- Gomes & Oliveira 2006 — "Hybridising SA and LP" — https://www.sciencedirect.com/science/article/abs/pii/S0377221704005879
- ESICUP benchmark datasets — https://www.euro-online.org/websites/esicup/data-sets/

---

## 8. Benchmark Datasets

The **ESICUP** (EURO Special Interest Group on Cutting and Packing) maintains standard benchmark instances:
- Albano (24 items), Dagli (30), Fu (12), Jakobs1 (25), Jakobs2 (25), Mao (20)
- Plus: blaz, marques, shirts, trousers
- Used by virtually all papers for comparison

Available at: https://www.euro-online.org/websites/esicup/data-sets/
