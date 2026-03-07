# pynest2d Analysis

## 1. Overview

**pynest2d** is a set of CPython bindings for [libnest2d](https://github.com/tamasmeszaros/libnest2d), a C++ library that solves 2D bin packing problems. It is developed and maintained by **Ultimaker B.V.** and is a component of the **Cura** 3D printing slicer ecosystem. The project is licensed under LGPL-3.0.

libnest2d implements algorithms for arranging 2D polygons into bins with minimal wasted space. pynest2d exists to expose this functionality to Python, specifically for use within Cura's arranging pipeline. Ultimaker maintains their own fork of libnest2d at `github.com/Ultimaker/libnest2d`, which may contain unmerged patches.

The README notes that an alternative Python binding project (`nest2D` by markfink) exists, but pynest2d was created because the alternative "doesn't expose enough of the configurability that Cura requires and its bindings aren't as transparent."

**Version:** 5.10.0 (as defined in `conandata.yml`)

## 2. Architecture

### Binding Technology: SIP

pynest2d uses **SIP** (specifically SIP 6.x) to generate Python bindings from `.sip` definition files. SIP is a tool historically used by PyQt to create C/C++ bindings for Python. Each `.sip` file describes a C++ class and how it maps to Python.

The `.sip` files are located in the `python/` directory:

| File | Purpose |
|------|---------|
| `pynest2d.sip` | Module root; includes all other SIP files, defines `nest()` function and `std::vector<Item*>` mapped type |
| `Point.sip` | Wraps `Point` (Clipper's `IntPoint`) |
| `Item.sip` | Wraps `Item` -- the main polygonal shape class |
| `Box.sip` | Wraps `Box` -- rectangular bin/shape |
| `Circle.sip` | Wraps `Circle` |
| `Rectangle.sip` | Wraps `Rectangle` (a subclass-like specialization of Item) |
| `NfpConfig.sip` | Wraps `NfpPConfig` -- configuration for the NFP placer |
| `BottomLeftConfig.sip` | Wraps `BLConfig` -- configuration for the Bottom-Left placer |
| `DJDHeuristicConfig.sip` | Wraps `DJDHeuristic::Config` -- configuration for the DJD selector |
| `ItemGroup.sip` | Mapped type for collections of items |
| `String.sip` | Mapped type for `std::string` <-> Python `str` conversion |

### Build System

The build pipeline is **Conan 2.x + CMake + SIP**:

1. **Conan** (`conanfile.py`) manages all dependencies: `nest2d` (libnest2d), `cpython`, `nlopt`, `clipper`, plus build tools (`sipbuildtool`, `standardprojectsettings`).
2. During `conan generate`, SIP processes the `.sip` files to produce C++ source code.
3. **CMake** (`CMakeLists.txt`) compiles the generated C++ into a shared library (`pynest2d.so` / `pynest2d.pyd`).
4. The CMake file is minimal (17 lines): it finds Python and nest2d, creates an interface library linked against `nest2d::nest2d` and `cpython::cpython`, then calls `add_sip_module` and `install_sip_module`.

**C++ standard:** C++17 minimum.

**Key dependencies:**
- libnest2d (the actual nesting engine)
- Clipper 6.4.2 (polygon clipping backend for libnest2d)
- NLopt (optimization library used by libnest2d's subplex optimizer)
- CPython 3.12.2

### How Bindings Work

SIP generates C/C++ wrapper code from `.sip` files. Each `.sip` file declares a class interface that mirrors the C++ API. Some methods require custom `%MethodCode` blocks for translation -- for example, `Point.x()` maps to Clipper's `sipCpp->X` field, and `Item.__init__` manually converts a Python list of Point objects into a `std::vector<Point>`.

Two `%MappedType` blocks handle collection conversions:
- `std::vector<Item*>`: converts Python lists of Items to/from C++ vectors (used as input to `nest()`)
- `ItemGroup`: similar mapping for the placer's internal item group type

## 3. API

### Core Function

```python
nest(items: list[Item], bin: Box, distance: int = 1, config: NfpConfig = NfpConfig()) -> int
```

The primary function. Arranges items into copies of `bin`, returns the number of bins required. Results are stored as transformations on the input Item objects -- retrieve them via `item.translation()`, `item.rotation()`, `item.transformedShape()`, and `item.binId()`.

Uses the **FirstFit selector** and **NfpPPlacer** (No-Fit Polygon placer) with Clipper as the geometry backend.

### Classes

**Point(x: int, y: int)**
- `.x()`, `.y()` -- integer coordinates

**Item(vertex_list: list[Point])**
- Construction from a list of `Point` vertices (polygon contour)
- Geometric queries: `area()`, `vertexCount()`, `holeCount()`, `boundingBox()`, `isContourConvex()`, `isInside(...)`, `vertex(idx)`, `setVertex(idx, v)`
- Transformation: `translate(delta)`, `rotate(radians)`, `translation()`, `rotation()`, `transformedShape()`, `resetTransformation()`, `inflation(distance)`
- Bin assignment: `binId()`, `binId(idx)`, `markAsFixedInBin(binid)`, `markAsDisallowedAreaInBin(binid)`, `isFixed()`, `isDisallowedArea()`
- Priority: `priority()`, `priority(value)`
- Static: `Item.intersects(sh1, sh2)`, `Item.touches(sh1, sh2)`

**Box** (bin shape)
- Multiple constructors: `Box(center)`, `Box(min, max)`, `Box(width, height)`, `Box(width, height, center)`
- `Box.infinite(center)` -- an unbounded bin
- `minCorner()`, `maxCorner()`, `width()`, `height()`, `area()`, `center()`

**Circle(center: Point, r: float)**
- `center()`, `radius()`, `area()`

**Rectangle(width: int, height: int)**
- Inherits most Item methods (duplicated in the SIP file since SIP does not handle C++ inheritance from templated classes transparently)

**NfpConfig** (NFP placer configuration)
- `alignment`: enum `CENTER, BOTTOM_LEFT, BOTTOM_RIGHT, TOP_LEFT, TOP_RIGHT, DONT_ALIGN`
- `starting_point`: same enum
- `rotations`: list of float angles to try (default `[0, pi/2, pi, 3pi/2]`)
- `accuracy`: float 0.0-1.0 (default 0.65)
- `explore_holes`: bool (default False; noted as non-functional)
- `parallel`: bool (default True)

**BottomLeftConfig** (BL placer configuration)
- `min_obj_distance`, `epsilon`, `allow_rotations`

**DJDHeuristicConfig** (DJD selector configuration)
- `try_reverse_order`, `try_pairs`, `try_triplets`, `initial_fill_proportion`, `waste_increment`, `allow_parallel`, `force_parallel`

### Usage Example (from docs)

```python
from pynest2d import *

volume = Box(1000, 1000)
triangle = Item([Point(0, 0), Point(100, 100), Point(50, 100)])
square = Item([Point(0, 0), Point(100, 0), Point(100, 100), Point(0, 100)])

num_bins = nest([triangle, square], volume)
# num_bins == 1

transformed = triangle.transformedShape()
print(transformed.toString())
print(triangle.rotation())
```

## 4. Strengths and Weaknesses

### Strengths

- **Exposes extensive configurability** from libnest2d -- multiple placer strategies (NFP, Bottom-Left), selector strategies (FirstFit, DJD Heuristic), alignment options, rotation sets, accuracy tuning, and parallelism control.
- **Well-documented SIP files** -- every class and method has Python docstrings embedded in the `.sip` definitions, which propagate to the compiled module.
- **Integer coordinates** -- uses Clipper's integer-based geometry, avoiding floating-point precision issues in polygon operations.
- **Production-tested** -- used in Cura, a widely deployed 3D printing slicer.
- **Supports fixed items and disallowed areas** -- items can be pinned in place, and disallowed regions can be defined.
- **Priority system** for controlling placement order.

### Weaknesses

- **Complex build system** -- requires Conan 2.7+, SIP 6, CMake, and several Ultimaker-specific Conan packages (`sipbuildtool`, `standardprojectsettings`, `pyprojecttoolchain`). Not pip-installable; no PyPI package.
- **Tightly coupled to Ultimaker ecosystem** -- uses Ultimaker's fork of libnest2d, Ultimaker's Conan config, and Ultimaker's CI workflows. Community developers must remove the proprietary Conan remote.
- **SIP is an unusual binding technology** -- less familiar than pybind11 or ctypes, making contribution harder.
- **No holes support in practice** -- although Item has `holeCount()` and `isHoleConvex()`, there is no exposed way to create items with holes.
- **Only Box bins** -- the `nest()` function only accepts `Box` as a bin type, despite libnest2d supporting arbitrary polygon bins.
- **Coordinates are integer-only** (Clipper's `long`) -- limits precision for applications needing sub-unit accuracy.
- **`explore_holes` is documented as non-functional** in libnest2d.
- **Rectangle duplicates Item's API** in the SIP file rather than inheriting, creating maintenance overhead.
- **Minimal test** -- `test_package/test.py` only checks that `SIP_VERSION` exists in the module; no functional tests of nesting.
- **Bug in MappedType code** -- both `pynest2d.sip` and `ItemGroup.sip` use `sipCpp->at(0)` in their `%ConvertFromTypeCode` loop (always index 0 instead of `i`), meaning the conversion from C++ vector to Python list would produce a list of identical items.

## 5. Key Code Paths

### Nesting Entry Point
`/Users/finn.carlsvi/thetta/research/repos/pynest2d/python/pynest2d.sip` -- lines 106-154. The `nest()` function:
1. Enforces minimum distance of 1.
2. Copies `Item*` pointers from the Python list into a `std::vector<Item>`.
3. Wraps the NfpConfig into a `NestConfig<>`.
4. Calls libnest2d's `nest()`.
5. Copies transformations back to the original Item pointers.

### Item Construction
`/Users/finn.carlsvi/thetta/research/repos/pynest2d/python/Item.sip` -- lines 30-59. The `%MethodCode` block iterates a Python list, calls `.x()` and `.y()` on each element, builds a `std::vector<Point>`, and constructs a C++ `Item`.

### Type Conversions
- `std::vector<Item*>` mapping: `/Users/finn.carlsvi/thetta/research/repos/pynest2d/python/pynest2d.sip` lines 31-103
- `std::string` mapping: `/Users/finn.carlsvi/thetta/research/repos/pynest2d/python/String.sip`
- NfpConfig rotations list: `/Users/finn.carlsvi/thetta/research/repos/pynest2d/python/NfpConfig.sip` lines 42-68

### Build Pipeline
- `/Users/finn.carlsvi/thetta/research/repos/pynest2d/conanfile.py` -- Conan recipe orchestrating SIP code generation and CMake build.
- `/Users/finn.carlsvi/thetta/research/repos/pynest2d/CMakeLists.txt` -- 17-line CMake file linking nest2d and cpython, invoking SIP module build.
