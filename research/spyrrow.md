# spyrrow Analysis

## 1. Overview

**spyrrow** is a Python wrapper around the Rust project [sparrow](https://github.com/JeroenGar/sparrow), which solves 2D [strip packing problems](https://en.wikipedia.org/wiki/Strip_packing_problem). In strip packing, a set of 2D items must be placed within a strip of fixed height and variable width, minimizing the total strip width used.

- **Author:** Paul Durand-Lupinski (Reeverse Systems)
- **License:** MIT
- **Version:** 0.8.1
- **PyPI:** Published at `pypi.org/project/spyrrow/`
- **Documentation:** Hosted on ReadTheDocs

sparrow itself is built on top of [jagua-rs](https://github.com/JeroenGar/jagua-rs), a Rust library providing collision detection (using quadtrees) and geometric operations for irregular packing problems. spyrrow wraps sparrow's optimizer to provide a clean, high-level Python API for strip packing.

The relationship chain is: **spyrrow (Python)** -> **sparrow (Rust optimizer)** -> **jagua-rs (Rust geometry/CDE engine)**.

## 2. Architecture

### Binding Technology: PyO3 + Maturin

spyrrow uses **PyO3** (v0.27.2) to create native Python extension modules from Rust code. The build system is **Maturin**, the standard tool for building and publishing PyO3-based Python packages.

The entire binding is contained in a single Rust file: `src/lib.rs` (approximately 406 lines). This file defines all Python-visible classes and the module registration.

### Build System

- `pyproject.toml` declares maturin as the build backend with `requires = ["maturin>=1.8,<2.0"]`
- `Cargo.toml` defines the crate as a `cdylib` (C dynamic library, required for Python extensions)
- Rust edition 2024
- Python requires >= 3.10

**Key Rust dependencies (from Cargo.toml):**
- `sparrow` -- the actual optimizer, pinned to a specific git revision (`04f54ff7`)
- `jagua-rs` -- geometry engine, pinned to revision `877aaa3c`, with the `spp` (strip packing problem) feature
- `pyo3` -- Python bindings
- `rand` + `rand_xoshiro` -- random number generation for reproducibility
- `serde` + `serde_json` -- JSON serialization for `to_json_str()` methods
- `num_cpus` -- auto-detecting available cores

### How Wrapping Works

Each Python class is a Rust struct annotated with `#[pyclass]`, and methods are annotated with `#[pymethods]`. PyO3 handles all type marshalling between Python and Rust automatically. The wrapper structs are thin data containers -- the actual algorithm invocation happens in `StripPackingInstancePy::solve()`, which:

1. Translates the Python-facing `StripPackingInstancePy` into sparrow's native `ExtSPInstance` via a `From` trait implementation.
2. Configures sparrow's `DEFAULT_SPARROW_CONFIG` with settings from `StripPackingConfigPy`.
3. Calls `py.detach()` to release the Python GIL during the computation-heavy optimization.
4. Runs `sparrow::optimizer::optimize()`.
5. Exports the solution and maps it back to Python-facing `PlacedItemPy` structs.

The GIL release (`py.detach()`) is significant -- it means the Python interpreter is not blocked during the potentially long optimization, allowing other Python threads to run.

### CI/CD

The GitHub Actions CI (`.github/workflows/CI.yml`) builds wheels for:
- Linux (x86_64, x86, aarch64, s390x, ppc64le) including musl variants
- Windows (x64, x86)
- macOS (Intel and Apple Silicon)

Tests run via `pytest -n auto` (parallel test execution with pytest-xdist). Releases publish to PyPI via maturin.

## 3. API

The API consists of 5 classes. A type stub file (`spyrrow.pyi`) provides IDE support.

### Item

```python
spyrrow.Item(
    id: str,
    shape: Sequence[tuple[float, float]],
    demand: int,
    allowed_orientations: Sequence[float] | None
)
```

- `id` -- unique identifier string
- `shape` -- polygon boundary as sequence of (x, y) tuples; closing edge is inferred if the last point does not repeat the first
- `demand` -- how many copies of this item to place (must be >= 1)
- `allowed_orientations` -- angles in degrees; `None` means free rotation; empty list treated as `[0.0]`
- All attributes are readable and writable (`get_all, set_all`)
- `to_json_str()` -- JSON serialization
- Supports `deepcopy`

### StripPackingInstance

```python
spyrrow.StripPackingInstance(
    name: str,
    strip_height: float,
    items: Sequence[Item]
)
```

- `name` -- instance name (can be empty string)
- `strip_height` -- fixed height of the strip
- `items` -- list of Items; IDs must be unique (validated, raises `ValueError`)
- `solve(config: StripPackingConfig) -> StripPackingSolution` -- runs the optimization
- `to_json_str()` -- JSON serialization
- Supports `deepcopy`

### StripPackingConfig

```python
spyrrow.StripPackingConfig(
    early_termination: bool = True,
    quadtree_depth: int = 4,
    min_items_separation: float | None = None,
    total_computation_time: int | None = 600,
    exploration_time: int | None = None,
    compression_time: int | None = None,
    num_workers: int | None = None,
    seed: int | None = None
)
```

**Time budget:** Either provide `total_computation_time` (split 80/20 between exploration and compression) OR provide both `exploration_time` and `compression_time`. Other combinations raise `ValueError`.

- `early_termination` -- allow the algorithm to stop early if no progress is being made
- `quadtree_depth` -- depth of the collision detection quadtree (typical values: 3-5)
- `min_items_separation` -- minimum distance between items
- `num_workers` -- threads for collision detection; defaults to CPU core count
- `seed` -- for reproducibility; random if not specified
- `to_json_str()` -- JSON serialization
- Supports `deepcopy`

### StripPackingSolution (output only)

- `width: float` -- the resulting strip width
- `density: float` -- fraction of strip area used by items
- `placed_items: list[PlacedItem]` -- placement details
- Supports `deepcopy`

### PlacedItem (output only)

- `id: str` -- references the Item id from the instance
- `rotation: float` -- rotation angle in degrees
- `translation: tuple[float, float]` -- (x, y) translation vector, applied after rotation

### Usage Example

```python
import spyrrow

rect = spyrrow.Item("rect", [(0,0), (1,0), (1,1), (0,1), (0,0)], demand=4, allowed_orientations=[0])
tri = spyrrow.Item("tri", [(0,0), (1,0), (1,1), (0,0)], demand=6, allowed_orientations=[0, 90, 180, -90])

instance = spyrrow.StripPackingInstance("example", strip_height=2.001, items=[rect, tri])
config = spyrrow.StripPackingConfig(early_termination=False, total_computation_time=60, num_workers=3, seed=0)
sol = instance.solve(config)

print(sol.width)    # ~4.0
print(sol.density)
for pi in sol.placed_items:
    print(pi.id, pi.rotation, pi.translation)
```

## 4. Strengths and Weaknesses

### Strengths

- **Simple, clean API** -- only 5 classes with a straightforward workflow: define Items, create Instance, configure, solve, read Solution.
- **Easy installation** -- `pip install spyrrow` works out of the box. Pre-built wheels for Linux, macOS, and Windows across multiple architectures.
- **Float coordinates** -- uses `f32` for all geometry, supporting real-valued coordinates and dimensions.
- **Continuous rotation support** -- `allowed_orientations=None` enables free rotation, not just fixed angle increments.
- **Demand/multiplicity** -- items can have `demand > 1`, placing multiple copies without duplicating shape definitions.
- **GIL release during solving** -- `py.detach()` frees the GIL during optimization, enabling Python concurrency.
- **Reproducibility** -- seed parameter for deterministic results.
- **Multi-threaded** -- configurable number of worker threads for the collision detection engine.
- **Type stubs** -- `spyrrow.pyi` provides full type information for IDE auto-completion and type checking.
- **Good test coverage** -- tests for basic packing, early termination, zero demand (error), no items, single item, concave polygons, continuous rotation, consecutive solve calls, and deepcopy of all types.
- **JSON serialization** -- all input types support `to_json_str()` for debugging and interoperability.
- **Two-phase optimization** -- configurable exploration and compression time budgets.

### Weaknesses

- **Strip packing only** -- solves only the strip packing variant (fixed height, minimize width). Does not support bin packing (fixed bin, minimize count) or other nesting problem types.
- **No holes in items** -- explicitly documented limitation. Items are simple polygons only.
- **No callback/progress reporting** -- the `solve()` method blocks (though GIL-free) with no way to observe intermediate solutions. The code has a `DummySolListener` placeholder.
- **No way to retrieve the actual placed polygon geometry** -- `PlacedItem` provides rotation + translation, but the user must reconstruct the final polygon shape themselves.
- **Limited solver configurability** -- many sparrow internals (shrink decay strategy, consecutive failure limits, polygon simplification tolerance) are hardcoded or use defaults with no Python-level exposure.
- **Pinned git revisions** -- both `sparrow` and `jagua-rs` are pinned to specific commits (not releases), creating fragility and making it unclear which version of the underlying solver is in use.
- **No Ctrl+C handling** -- noted in `TODO.txt` as a needed improvement; long-running solves cannot be interrupted from Python.
- **Typo in README example** -- `num_wokers` parameter name (though the actual API uses the correct `num_workers`).
- **All attributes are mutable** on input types (`get_all, set_all`) including on `StripPackingConfig` where internal `Duration` fields (`exploration_time`, `compression_time`) are exposed but would be difficult to set correctly from Python.

## 5. Key Code Paths

### Single Source File
`/Users/finn.carlsvi/thetta/research/repos/spyrrow/src/lib.rs` -- the entire wrapper implementation.

### Item Definition
Lines 19-90. `ItemPy` struct with `id`, `demand`, `allowed_orientations`, `shape` fields. Constructor, `__repr__`, `__deepcopy__`, and `to_json_str`.

### Instance-to-Sparrow Conversion
Lines 262-289. The `From<StripPackingInstancePy> for ExtSPInstance` implementation converts Python-facing items into jagua-rs's `ExtItem` format, wrapping shapes as `ExtShape::SimplePolygon`.

### Solve Method
Lines 328-393. `StripPackingInstancePy::solve()`:
1. Handles empty-items edge case (returns zero-width solution).
2. Configures sparrow with exploration/compression time limits, worker count, quadtree depth, early termination settings, seed.
3. Calls `py.detach()` to release the GIL.
4. Runs `sparrow::optimizer::optimize()`.
5. Exports solution via `jagua_rs::probs::spp::io::export()`.
6. Maps placed items back to `PlacedItemPy` structs with degrees-converted rotation.

### Config Validation
Lines 184-241. `StripPackingConfigPy::new()` validates time budget arguments, ensuring either `total_computation_time` alone or both `exploration_time` and `compression_time` are provided.

### Type Stubs
`/Users/finn.carlsvi/thetta/research/repos/spyrrow/spyrrow.pyi` -- complete type stub file for IDE integration.

### Tests
- `/Users/finn.carlsvi/thetta/research/repos/spyrrow/tests/test_basic.py` -- functional tests including packing correctness assertions
- `/Users/finn.carlsvi/thetta/research/repos/spyrrow/tests/test_copy.py` -- deepcopy tests for all types
- `/Users/finn.carlsvi/thetta/research/repos/spyrrow/tests/utils.py` -- utility for converting instances to sparrow's JSON format
