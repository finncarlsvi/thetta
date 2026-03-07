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

## Tech stack

- **Language:** Rust
- **GUI:** egui/eframe (immediate mode, renders via wgpu/Metal)
- **2D viewport:** Custom renderer with zoom/pan via RectTransform
- **STEP handling:** OpenCASCADE via opencascade-sys FFI
- **Nesting engine:** jagua-rs collision detection + custom optimizer (inspired by sparrow)
- **Target platform:** macOS on Apple Silicon
