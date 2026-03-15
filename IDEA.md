# Thetta

A nesting application for packing 2D profiles from STEP files into rectangular sheets.
Named after the Old Norse word meaning "to pack tight."

## What it does

Desktop GUI application where the user can:

1. Load one or more STEP files
2. On first file load, the app guesses the thickness by finding the most common thickness value among all solids in the file. The user can change this at any time.
3. All solids matching the selected thickness are extracted and flattened to 2D profiles
4. The app packs the profiles to save as much space as possible, with configurable separation, into rectangular sheets. Sheet size defaults to 2440mm x 1220mm but can be changed by the user. There may be more than one sheet.
5. The 2D viewport shows a top-down view of the nested result
6. The user can delete individual shapes from the nest
7. The user can reset/reload all shapes from a loaded file
8. The user can export one STEP file per sheet

## 2D CAM

After nesting, the user can generate toolpaths and export G-code for cutting on a CNC router via LinuxCNC. The CAM module supports three operations:

1. **Profile cuts** — Cut along the outer contour of each nested piece. Supports holding tabs (small uncut bridges) to keep pieces attached to the sheet during machining.
2. **Hole drilling** — Drill through-holes detected in the piece geometry.
3. **Pocket clearing** — Mill out enclosed pockets detected in the piece geometry.

These are the only three operations. No 3D contouring, no engraving, no threading.

### Tool database

The app maintains a tool database compatible with LinuxCNC's tool table format. Each tool has at minimum: tool number, diameter, flute count, feed rate, plunge rate, and spindle speed. The user can create, edit, and delete tools, and assign tools to operations (e.g., a specific endmill for profile cuts, a drill bit for holes).

### Tabs

For profile cuts, the user can configure tabs — small uncut sections left along the outer contour to prevent the piece from shifting during cutting. Tab parameters: width, height (partial-depth cut), and spacing or count per piece.

### G-code export

Export G-code targeting LinuxCNC. One G-code file per sheet. The output includes:
- Tool changes with LinuxCNC conventions (T/M6)
- Operation ordering: drill holes first, then pockets, then profile cuts (profiles last so pieces stay fixed)
- Tab-aware profile cuts (ramp down, lift over tabs)
- Configurable safe-Z, feed rates per tool, and spindle speed

## Tech stack

- **Language:** Rust
- **GUI:** egui/eframe (immediate mode, renders via wgpu/Metal)
- **2D viewport:** Custom renderer with zoom/pan via RectTransform
- **STEP handling:** OpenCASCADE via opencascade-sys FFI
- **Nesting engine:** jagua-rs collision detection + custom optimizer (inspired by sparrow)
- **Target platform:** macOS on Apple Silicon
