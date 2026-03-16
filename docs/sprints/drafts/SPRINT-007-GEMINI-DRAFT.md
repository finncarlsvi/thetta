# Sprint 007: G-code Profile Cuts with Tool Database

## Overview

Implement G-code generation for profile cuts with tab-aware toolpaths, plus a tool database compatible with LinuxCNC's tool table format. This is the core CAM feature that makes Thetta usable for actual CNC production: after nesting and tab placement, the user exports G-code that a CNC router can physically cut.

The sprint introduces a new `thetta-cam` workspace crate to isolate toolpath generation and G-code formatting from the pure geometry and step I/O logic. It consumes the `NestingResult` and `TabState` to produce safe, tab-aware profile toolpaths.

The implementation includes a persistent tool database, G-code configuration parameters (safe-Z, cut depth, feed rates), and an intuitive UI to manage tools and trigger the export. One G-code file is generated per sheet, strictly adhering to LinuxCNC conventions.

## Use Cases

1. **Tool Database Management**: User can create, edit, and delete tools in a persistent tool database UI. Fields include tool number, diameter, flute count, feed rate, plunge rate, and spindle speed.
2. **Assign Tool to Operation**: User can select a tool from the database specifically for profile cuts.
3. **Configure G-code Parameters**: User can configure global G-code settings such as Safe-Z height, material cut depth, and coordinate system.
4. **Tab-Aware Toolpaths**: When generating G-code, the toolpath automatically lifts the tool over previously generated holding tabs (ramping up to `cut_depth - tab_height`) and plunges back down, ensuring parts remain attached to the stock sheet.
5. **G-code Export per Sheet**: User clicks "Generate G-code", producing one `.ngc` (or `.nc`) file per sheet formatted for LinuxCNC, including standard preambles and tool change (T/M6) commands.
6. **Tool Table Persistence**: Tool database is saved to disk across sessions automatically as JSON in the application's config/data directory.

## Architecture

### Component Overview

```text
+-----------------------------------------------------------+
|                      thetta-app                            |
|  +----------+  +--------------------------+  +---------+   |
|  | panels.rs|  |       state.rs           |  | io.rs   |   |
|  | Tool UI  |  | ToolTable, GcodeConfig,  |  | Save DB |   |
|  | controls |  | CamState                 |  | Export  |   |
|  +----+-----+  +-----------+--------------+  +----+----+   |
+-------+--------------------+----------------------+--------+
        |                    |                      |
        |                    |                      v
        |                    |             +-----------------+
        |                    |             |  thetta-cam     |
        |                    |             | +-------------+ |
        |                    |             | | toolpath.rs | |
        |                    +------------>| | tab lifts,  | |
        |                                  | | profile cuts| |
        |                                  | +------+------+ |
        |                                  |        |        |
        |                                  | +------v------+ |
        v                                  | |  gcode.rs   | |
+---------------------+                    | | LinuxCNC    | |
|   thetta-geometry   |                    | | formatter   | |
| +-----------------+ |                    | +-------------+ |
| |    types.rs     | |                    +-----------------+
| | Tool, ToolTable,| |
| | GcodeConfig,    | |
| | Toolpath        | |
| +-----------------+ |
+---------------------+
```

### Data Flow

```text
NestingResult + TabState + ToolTable + GcodeConfig
        |
        v
thetta-cam::generate_toolpaths(sheet_placements, tabs, tool, config)
        |
        +-- For each piece:
        |    +-- Extract world-space outer contour
        |    +-- Apply tool radius compensation (offset by tool diameter / 2)
        |    +-- Identify tab positions along offset contour
        |    +-- Build Toolpath (linear moves, Z-lifts over tabs at tab positions)
        v
Vec<Toolpath>
        |
        v
thetta-cam::export_linuxcnc_gcode(toolpaths, config)
        |
        +-- Emit Preamble (G21, G90, G54)
        +-- Emit Tool Change (T# M6, S# M3)
        +-- Emit Moves (G0 for rapid, G1 with F# for cut/plunge)
        +-- Emit Postamble (M5, M30)
        v
G-code String (written to file per sheet)
```

## Implementation

### Phase 1: Core Types & Tool Database UI
**Goal:** Define types and build the UI to manage and persist tools.
- **`thetta-geometry/src/types.rs`**: Add `Tool`, `ToolTable`, `GcodeConfig`, `Toolpath`, `CamOperation` structs with `Serialize`/`Deserialize`.
- **`thetta-app/src/state.rs`**: Add `CamState` to hold the tool table and G-code config. Implement load/save to disk for `ToolTable`.
- **`thetta-app/src/panels.rs`**: Add "CAM / Tool Database" tab. Implement CRUD UI for tools and inputs for G-code config (Safe-Z, cut depth).

### Phase 2: Toolpath Generation (`thetta-cam` crate)
**Goal:** Convert geometry and tabs into 3D XYZ toolpaths.
- **`crates/thetta-cam`**: Initialize new workspace crate. Add it to workspace `Cargo.toml`.
- **`thetta-cam/src/toolpath.rs`**: Implement `generate_profile_toolpath`. This takes a piece's polygon, applies an outward offset of `tool.diameter / 2`, matches tab positions to the offset path, and generates a sequence of 3D waypoints. Implement ramp-up/ramp-down logic for tabs based on `tab.height`.

### Phase 3: G-Code Emitter & Export
**Goal:** Format toolpaths to LinuxCNC dialect and write to files.
- **`thetta-cam/src/gcode.rs`**: Implement `format_gcode_sheet`. Generates preamble, loops through toolpaths creating `G0`/`G1` blocks, and adds postamble.
- **`thetta-app/src/main.rs` & `state.rs`**: Wire up "Generate G-code" action. Trigger native save dialog to select output directory, and iterate over sheets to save `.ngc` files.

## Files Summary

| File | Purpose |
|------|---------|
| `Cargo.toml` | Add `thetta-cam` to workspace members |
| `crates/thetta-geometry/src/types.rs` | Add `Tool`, `ToolTable`, `GcodeConfig`, `Toolpath` |
| `crates/thetta-cam/Cargo.toml` | New crate definition |
| `crates/thetta-cam/src/lib.rs` | Crate exports |
| `crates/thetta-cam/src/toolpath.rs` | Tab-aware contour-to-toolpath conversion |
| `crates/thetta-cam/src/gcode.rs` | LinuxCNC string emission |
| `crates/thetta-app/src/state.rs` | Add `CamState`, persist tool DB, trigger export |
| `crates/thetta-app/src/panels.rs` | Add CAM panel, tool DB UI, export button |

## Definition of Done

### P0 — Must Ship
- [ ] `thetta-cam` crate created and integrated into workspace.
- [ ] `Tool`, `ToolTable`, `GcodeConfig` types exist in `thetta-geometry`.
- [ ] Tool database UI allows creating, editing, and deleting tools.
- [ ] Tool table persists to disk automatically across app restarts.
- [ ] G-code configuration UI exists (safe-Z, target cut depth, etc.).
- [ ] "Generate G-code" outputs one `.ngc` file per sheet.
- [ ] Exported G-code includes correct LinuxCNC preamble (units, absolute mode) and postamble (spindle off, rewind).
- [ ] Exported G-code contains valid tool change blocks (`T# M6`).
- [ ] Profile cuts trace the outer contour of parts with outward tool radius compensation.
- [ ] Profile cuts lift Z to `cut_depth - tab_height` at tab locations.
- [ ] Unit tests for G-code formatting and toolpath tab lifting.
- [ ] `cargo test --workspace` passes without errors.

### P1 — Should Ship
- [ ] Ramp lead-in/lead-out for tab transitions (instead of vertical plunges).
- [ ] Spindle speed (`S# M3`) and Feed rates (`F#`) properly output in G-code.
- [ ] Warning if attempting to export without tabs configured or with tool diameter larger than part features.

### P2 — Stretch
- [ ] Multi-pass step-down cuts (taking multiple passes to reach final depth).
- [ ] In-app 3D toolpath preview overlay in the viewport.
- [ ] Toolpath climb vs conventional milling toggle.

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Tool radius compensation (offsetting) failing on complex/tight corners | High | High | Use the robust `offset` approximation in `polygon_ops`, fallback to zero offset and warn user if topology changes unexpectedly, or keep it simple for V1. |
| Tab positions desyncing after tool radius compensation | Medium | High | Map tab parameter `t` from the original contour to the offset contour proportionally. |
| G-code dialect incompatibilities | Low | High | Strictly follow the LinuxCNC documentation. Verify with simulators like CAMotics. |
| Z-axis crashes | Medium | High | Ensure all inter-part rapid moves use `G0 Z[Safe_Z]`. Add unit tests explicitly checking Z height between piece boundaries. |

## Security Considerations

- **File System Access**: G-code export and Tool Database persistence require file writing. These should use standard, safe Rust I/O and respect the user's chosen directories via native dialogs.
- **G-code Safety**: Ensure output numbers do not result in NaN or unparseable formats that could cause unpredictable CNC machine behavior. Always format floats to fixed decimal precision (e.g., `%.4f`).

## Dependencies

- No new external crates are strictly required. G-code generation is pure text formatting. `thetta-cam` relies only on standard library, `thetta-geometry`, and `serde`.

## Open Questions

1. Should G-code generation live in `thetta-step` or a new `thetta-cam` crate? -> Resolved: New `thetta-cam` crate.
2. Should there be an intermediate `Toolpath` representation, or emit G-code directly from contour + tabs? -> Resolved: Use intermediate `Toolpath` to separate geometric interpolation from string formatting.
3. How should lead-in/lead-out be handled? -> Direct plunge for P0, ramp for P1.
4. Should step-down (multiple depth passes) be supported in this sprint or deferred? -> Deferred to P2 Stretch.
5. What's the right tool table persistence format? -> JSON in application data directory.
6. Should the G-code viewer/preview be in-app or external only? -> External only for P0, in-app for P2 Stretch.
7. How to handle cutting direction? -> Conventional milling default, toggle deferred to P2.