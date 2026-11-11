# Thetta website

Open **https://thetta.finn.xyz** and sign in with the approved Onshape account. Import a STEP assembly,
set quantities and rotations, then panelize all thicknesses. Each import
replaces the current source assembly. Stop nesting before changing parts or
settings. Unplaced parts are reported separately and are not included in exports.

Use the prominent **Nesting / CAM** switch below the project title to change the
workspace. Nesting shows the part library and layout overview. CAM shows the tool
library on the left and operation/tool assignments, holding tabs, and G-code
export on the right. **Add tool** opens an unsaved tool definition; enter your
machine’s actual tool values and save it, then assign tools to the enabled
operations. Switching modes preserves the layout.

Imported tool definitions may be incomplete. Missing dimensions, flute counts,
feeds, plunge rates, and spindle speeds appear as **Not set** / **Setup needed**.
These definitions can be saved and edited. Complete every selected operation's
tool before exporting G-code; unused incomplete tools may remain in the library.

The canvas supports scroll to zoom, drag to pan, and click to select. Double-click
a panel (including its empty space) to zoom and center that panel. Double-click
the background or use **Fit all panels** to return to the overview. Every enabled material at or below 30 mm is nested separately;
panels appear together in labeled thickness arrays. The parts list is grouped
by thickness and starts collapsed. Expand a header to see its parts; use its
**Place** checkbox to include or exclude the entire material group. Group choices
are saved per workspace and preserve individual part selections. Changing a
group invalidates the layout; panelize again to apply the new selection.
Thicker parts remain listed as excluded. Legacy thickness
filters are restored to the full assembly while preserving edits to existing
parts. The total optimization time budget is shared between material groups.
A completion pass places remaining fitting parts into free space or additional
panels, respecting spacing and allowed rotations. This includes long pieces
that need diagonal placement.
Every ordinary **Panelize** run in sheet mode automatically performs contour-aware
refinement and coordinated sheet consolidation within its existing time budget.
No separate click or saved project-specific placement is required.
**Improve layout** optionally searches the saved layout for
up to three minutes. It uses actual part contours to interlock concave pieces
and move parts between panels. Coordinated cluster moves can combine sparse
panels while preserving already interlocked groups. The search prioritizes fewer panels, then the shortest
occupied width from the left edge, then a smaller occupied footprint. Parts can
use the full panel height, leaving a continuous offcut on the right. Thicknesses, spacing, and rotation restrictions are preserved. The
best complete, validated layout stays visible and saves automatically. **Stop
nesting** retains that best result. If no improvement is found, the original
layout and tabs remain; accepting changed placements clears tabs for regeneration.
Older non-overlapping layouts with insufficient clearance are repaired first;
this can require extra panels to restore the configured spacing before repacking.
If stopped before that repair completes, the original layout remains unchanged.
The initial panelization time setting does not change this improvement budget.

STEP and G-code downloads are ZIP archives with one folder per thickness and one
file per panel. Cutting depth and tab material thickness follow each panel's
actual material thickness.
Holding tabs affect G-code; the existing STEP exporter preserves the original
solid geometry. Configure the tool library and assign tools to enabled CAM
operations before exporting G-code.

Workspaces save automatically, including the original source, configuration,
latest layout, and generated tabs. A Secure HttpOnly cookie identifies the authenticated session;
workspaces remain saved when the browser session ends. After sign-in, the
project explorer lists the user's saved workspaces with profile previews,
source, and modification time. Search, filter, sort, or use grid/list views to
find a project. **New workspace** imports a STEP file or Onshape model into an
isolated project; opening an empty workspace offers the same import dialog. Studio
mode contains parts and layout controls, with linked Onshape source/refresh when
available. **Workspaces**
in the editor returns to the explorer. Workspace links and browser back/forward
navigation reopen the selected project. There is no public sharing or machine control.
A restart restores saved work and stops interrupted optimizations; it does not
resume them automatically. Reconnecting browsers restore their workspace before
resubscribing to updates.

See [automatic material-aware CAM and cutting database](AUTOMATIC_CAM.md) for
material selection, machine setup, reviewed recipes, multi-pass machining, and
the database/API architecture. Existing projects retain manual CAM mode.

## Local development

```sh
npm --prefix web ci
npm --prefix web run build
THETTA_TOKEN_KEY_FILE=/path/to/32-byte-key cargo run -p thetta-web
```

The server requires OAuth configuration; see [Onshape setup](ONSHAPE.md). Environment variables:

| Variable | Default | Purpose |
| --- | --- | --- |
| `THETTA_BIND` | `0.0.0.0:3007` | HTTP listening address |
| `THETTA_DATA_DIR` | `.thetta-data` | Persistent workspace root |
| `THETTA_STATIC_DIR` | `web/dist` | Compiled frontend directory |

The server depends on the native STEP/OpenCASCADE build toolchain. The Dockerfile
supplies this toolchain for Linux. The desktop executable remains available with
`cargo run -p thetta-app`.

## Deployment and maintenance

Run `scripts/deploy-web.sh` from the repository. It copies the working tree,
including local changes, builds on the x86_64 host, and waits for the service to
be healthy. Existing SSH access is used; no passwords are stored in the project.
The host checkout is `/home/finn/thetta`.

```sh
ssh finn@192.168.5.5 'cd /home/finn/thetta && docker compose ps'
ssh finn@192.168.5.5 'docker logs --tail 100 thetta'
```

Compose binds only `192.168.5.5:3007`. The `thetta_thetta-data` named volume holds
workspace data at `/data`; normal container replacement preserves it. Back up
that volume to retain all workspaces. Do not remove it when updating the app.
The container runs as UID 1000 and restarts unless explicitly stopped.

On subsequent deployments, the script retains the previous image as
`thetta:previous`. To roll back:

```sh
ssh finn@192.168.5.5 'docker tag thetta:previous thetta:latest && cd /home/finn/thetta && docker compose up -d --no-build --force-recreate --wait'
```

## API

`POST /api/workspace` opens the authenticated user’s selected workspace;
`GET /api/workspace` returns a snapshot. `GET /api/events` streams snapshots as
SSE, with keepalives. Snapshots contain browser-facing geometry and settings,
never internal source paths.

- `POST /api/upload`: multipart `file`, `.step` or `.stp`, maximum request 100 MiB.
- `GET /api/workspaces`: owned workspace summaries and bounded preview geometry.
- `POST /api/workspaces`: select `{id}` or create with `{id: null}`; returns the workspace ID.
- `PUT /api/settings`: complete settings object from the snapshot.
- `PUT /api/parts/{id}`: `{quantity, enabled, rotation}`; rotation is `continuous`,
  `quarter`, or `fixed`. Quantity is 1–1000; at most 2000 enabled copies.
- `PUT /api/thickness`: `{value: null}` restores the full assembly; individual thickness filters are retired.
- `PUT /api/materials/{thickness}`: `{enabled: boolean}` includes/excludes a material group without changing individual part choices.
- `POST /api/nesting/start`, `POST /api/nesting/stop`: job controls.
- `POST /api/tabs`: generate tabs for the accepted layout.
- `POST /api/export/step`, `POST /api/export/gcode`: ZIP downloads.
- `GET /api/health`: process health.

Conflicting edits during a job return 409; invalid settings return 400. Settings
that affect placements invalidate the layout. Changes to tab parameters clear
previous tabs. Exports always use current CAM settings. Native OCCT calls are
serialized, and CPU jobs run outside the HTTP request executor. A stopped job
finishes cancellation before another can start.

## Validation

```sh
npm --prefix web run build
cargo test -p thetta-web
cargo test --workspace --exclude thetta-web
python3 scripts/smoke-web.py --url http://localhost:3007
```

Web integration tests cover session isolation, invalid settings, saved settings
and layout restoration, real STEP import/export/reimport, quantities, optimizer
cancellation, mutation conflicts, tabs, and G-code generation.

An opt-in regression exercises ordinary Panelize with the original 37-part
assembly. Supply its original STEP file and saved workspace JSON (used only for
part selections, quantities, rotations, and settings):

```sh
THETTA_REGRESSION_STEP=/path/to/original.step \
THETTA_REGRESSION_WORKSPACE=/path/to/workspace.json \
cargo test --release -p thetta-web ordinary_panelize_rebuilds_project_without_saved_placements -- --ignored --nocapture
```

It imports into a temporary workspace, discards saved placements, and checks all
37 parts fit on at most seven sheets with valid clearance, material groups,
rotations, and saved results. The nesting tests also cover merging fresh sparse
panels whose matching parts were initially assigned to different groups.

The full workspace run currently reports two failures in
`thetta-nesting/tests/benchmark_smoke.rs`: `lbf_places_all_on_small_instances`
and `strip_mode_jakobs1_stays_single_sheet_with_stable_length` (nonpositive strip
length assertions). The other 112 workspace tests and the web integration tests
pass.

See [Onshape login, registration, and proxy setup](ONSHAPE.md) for the public deployment.
