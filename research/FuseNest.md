# FuseNest

## Overview

A 2D nesting add-in for Autodesk Fusion 360 that wraps the SVGnest JavaScript library. FuseNest converts selected Fusion 360 bodies into SVG representations, sends them to SVGnest running in an embedded HTML palette, receives optimized placements, and applies the resulting transforms back to the 3D bodies. It supports rectangular sheets, configurable spacing, multiple sheets, and discrete rotation counts.

## Architecture

```
.
+-- FuseNest.py                # Main add-in: Fusion 360 API handlers (~952 lines)
+-- FuseNest.manifest          # Fusion 360 add-in manifest
+-- SVGnest/
|   +-- svgnest.js             # Core nesting engine (GA + NFP-based placement)
|   +-- svgparser.js           # SVG parsing utilities
|   +-- geometryutil.js        # Geometry helpers (polygon operations)
|   +-- placementworker.js     # Web Worker for parallel placement evaluation
|   +-- background.js          # Background processing
|   +-- util/                  # Additional utilities (geometric functions, etc.)
|   +-- index.html             # Nesting UI loaded in Fusion palette
+-- resources/                 # Icons and UI assets
```

### Key Files

- **`FuseNest.py`** (~952 lines): The entire add-in logic. Contains Fusion 360 command handlers (`CommandCreatedHandler`, `CommandExecuteHandler`, `CommandInputChangedHandler`), SVG generation from 3D bodies, palette event handling (`PaletteHTMLEventHandler`), and transform application. This single file handles the full round-trip: body selection -> SVG export -> nesting -> transform import -> body repositioning.
- **`SVGnest/svgnest.js`**: The JavaScript nesting engine, based on the open-source SVGnest project. Uses a Genetic Algorithm to optimize placement order and discrete rotations, with NFP-based placement evaluation. Configurable parameters include `spacing`, `rotations`, `populationSize`, `mutationRate`, and `binWidth`/`binHeight`.

## Algorithm Details

### SVG Generation from Fusion 360 Bodies

FuseNest projects each selected body onto the XY plane and converts the resulting sketch profiles into SVG paths. It handles multiple curve types:

- **Line3D**: Direct line segment conversion to SVG `L` commands.
- **Arc3D**: Converted to SVG arc commands (`A`) with proper radius, rotation, and sweep flag computation. Handles the large-arc-flag by checking the arc's sweep angle against pi.
- **Circle3D**: Rendered as two semicircular SVG arcs (since SVG cannot represent a full circle with a single arc command).
- **Ellipse3D**: Converted to SVG elliptical arc with major/minor radii and rotation angle.
- **EllipticalArc3D**: Similar to Arc3D but with elliptical parameters.
- **NurbsCurve3D**: Approximated by evaluating points along the curve at regular parameter intervals and connecting with line segments.

### Nesting (SVGnest)

The actual nesting is performed by SVGnest in a browser context (Fusion 360's CEF-based palette):

1. **Genetic Algorithm**: Optimizes the placement sequence and rotation assignments. Uses ordered crossover and mutation operators. Population and mutation rate are configurable.
2. **NFP-based Placement**: For each candidate sequence, polygons are placed using NFP computations. The feasible region is determined by subtracting NFPs from the bin's inner-fit polygon.
3. **Iterative Improvement**: The first iteration produces an initial layout; subsequent iterations attempt to improve it via the GA. The user can stop at any time and accept the current best.
4. **Multi-sheet Support**: When pieces don't fit on one sheet, additional sheets are created automatically, offset by the user-specified sheet offset distances.

### Transform Application

When the user accepts a nesting result:

1. The SVGnest output (SVG with transform attributes) is sent back to `FuseNest.py` via palette events.
2. `PaletteHTMLEventHandler` parses the SVG, extracting `transform="translate(x,y) rotate(angle)"` attributes for each polygon.
3. For each body, the corresponding transform is decomposed into a translation vector and rotation angle.
4. Fusion 360 API calls (`MoveFeature` or direct `Transform` operations) reposition the 3D bodies to match the 2D nesting layout.
5. Scale conversion handles the SVG-to-Fusion coordinate mapping (SVG uses pixels, Fusion uses the active unit system).

### Communication Architecture

The add-in uses Fusion 360's palette mechanism for bidirectional communication:

- **Python -> JavaScript**: Sends SVG polygon data and configuration parameters to the palette via `palette.sendInfoToHTML()`.
- **JavaScript -> Python**: SVGnest results are sent back via `window.fusionJavaScriptHandler.handle()`, which triggers `PaletteHTMLEventHandler.notify()` in Python.
- **UI Controls**: Start/Stop/Apply buttons in the HTML palette control the nesting process. Progress updates are displayed in the palette UI.

## Dependencies

- **Autodesk Fusion 360** -- host application, provides the API for body selection, sketch projection, and transform application
- **Fusion 360 API (Python)** -- `adsk.core`, `adsk.fusion`, `adsk.cam` modules
- **SVGnest** (bundled JavaScript) -- nesting engine, runs in the embedded Chromium palette
- **No external Python packages** -- all Python dependencies are satisfied by Fusion 360's built-in Python environment

## Strengths

- **Integrated CAD workflow**: Directly nests 3D bodies within Fusion 360 without export/import steps. Users select bodies, set parameters, nest, and apply -- all within the CAD environment.
- **Comprehensive curve handling**: Properly converts lines, arcs, circles, ellipses, elliptical arcs, and NURBS curves to SVG, covering the full range of Fusion 360 sketch geometry.
- **Interactive iteration**: Users can watch the nesting improve in real-time and stop when satisfied, balancing solution quality against computation time.
- **Multi-sheet support**: Automatically handles cases where pieces exceed one sheet's capacity, with configurable sheet offset for CNC/laser cutter workflows.
- **Zero external dependencies**: Everything needed is bundled -- SVGnest is included, and the Python code uses only Fusion 360's built-in modules.

## Weaknesses

- **Monolithic codebase**: All 952 lines are in a single file with no separation of concerns. SVG generation, event handling, transform parsing, and UI logic are interleaved.
- **SVGnest limitations inherited**: The bundled SVGnest is an older version of the open-source project. It uses a basic GA without advanced features like adaptive operators or hybridization with local search.
- **No NFP caching across sessions**: Each nesting run recomputes NFPs from scratch. There is no persistence of computed NFPs or previous solutions.
- **Fragile SVG parsing**: The transform extraction from SVGnest output uses string parsing of SVG attributes rather than a proper SVG/XML parser, making it brittle to format changes.
- **Limited to rectangular sheets**: Only rectangular bin shapes are supported. Irregular stock is not an option.
- **NURBS approximation quality**: NURBS curves are approximated with a fixed number of evaluation points, with no adaptive refinement based on curvature. Complex NURBS may lose detail.
- **No spacing guarantees**: The README notes spacing is "approximate" and "can vary by small amounts," indicating the spacing parameter is not geometrically exact.
- **Platform-locked**: Only works within Fusion 360. Cannot be used standalone or with other CAD software.

## Key Code Paths

1. **Body to SVG**: `CommandExecuteHandler.notify()` -> for each selected body: `body.createSilhouetteBoundary()` or sketch projection -> iterate over `profile.profileLoops` -> for each `profileCurve`: match type (Line3D, Arc3D, Circle3D, EllipticalArc3D, NurbsCurve3D) -> generate SVG path commands -> combine into SVG `<path>` elements -> wrap in SVG document with sheet rectangle as `<rect>`.

2. **Launch Nesting**: `CommandExecuteHandler.notify()` -> `palettes.add()` creates HTML palette loading `SVGnest/index.html` -> `palette.sendInfoToHTML()` sends SVG string and config (spacing, rotations, sheet dimensions) -> SVGnest JavaScript initializes and begins GA optimization.

3. **Receive Results**: SVGnest completes iteration -> `window.fusionJavaScriptHandler.handle(action, data)` -> `PaletteHTMLEventHandler.notify()` in Python -> parse returned SVG -> extract transform attributes per polygon -> store as pending placement data.

4. **Apply Transforms**: User clicks "Apply Nest" -> `CommandExecuteHandler` -> for each body/polygon pair: parse `translate(x,y) rotate(angle)` from SVG transform string -> compute Fusion 360 `Matrix3D` with translation and rotation -> apply via `MoveFeature` or `Transform` API -> bodies are repositioned in 3D space.

5. **UI Interaction**: `CommandInputChangedHandler.notify()` -> responds to parameter changes (sheet size, spacing, rotations) -> updates internal state -> if nesting was already computed, invalidates results and prompts re-nesting.
