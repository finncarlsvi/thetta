# Material-aware automatic CAM

Select **CAM → Automatic tools and settings**, choose the project material, and
use part material overrides in Nesting when necessary. Material changes invalidate
the layout. Panelize again: each material/thickness pair receives separate panels.
The original <=30 mm eligibility limit remains in force.

Open **Machine and installed tools** once to confirm actual spindle range,
practical cutting/entry feeds, stepdown, workholding and chip removal. Meitill's
configured ceilings are 24,000 RPM and 423.333 mm/s (25,399.98 mm/min); these are
limits, not recommended cutting settings. Confirm each usable cutter's material,
geometry, available cutting reach, maximum RPM, and compression transition where
applicable. Source tool comments do not establish usable cutting reach. Unnamed,
probe and unidentified tools remain in inventory without invented cutting data.

**Calculate cutting plan** selects compatible cutters and shows per-panel,
per-operation RPM, feed, entry feed, pass depth, pass count, stepover and evidence.
Manufacturer chip data and engineering starting assumptions are distinguished.
Review and accept starting estimates before G-code export. Local overrides can be
saved per material/operation; calculate and review again after changing them.
The automatic tool library displays rates as per-operation values, not permanent
properties of a physical cutter.

Wood, plywood, MDF, softwood, hardwood, acrylic and HDPE are covered by the catalog.
Plastic family estimates apply only to the documented single O-flute, carbide
cutter family and listed diameter. There is no invented universal drill recipe:
add a tested recipe for the actual installed drill and material. Tool families do
not transfer between carbide and HSS automatically. Ball-nose, V-bit and planer
entries are retained but are not selected for 2D flat-bottom operations.

## Evidence and calculation

The shipped revision is `2026-11-10.1`; facts and source provenance are in
`crates/thetta-web/data/cam-catalog-v1.json`. Cutting data comes from manufacturer documents; engineering starting assumptions are recorded separately:

- [Amana spiral chart v40](https://www.amanatool.com/pub/media/productattachments/Solid-Carbide-Spektra-Spiral-Plunge-2-3-Flute-v40.pdf): T5 / 46216-K and two-flute flat family starting data.
- [Amana compression chart v6](https://www.amanatool.com/pub/media/productattachments/Solid-Carbide-Spektra-Compression-Spirals-v6.pdf): T7 / 46010-K; wood/veneered plywood and MDF have different rows.
- [Amana ball-nose chart v12](https://www.amanatool.com/pub/media/productattachments/Spektra-Coated-3D-Profiling-Feed-Chip-Load-Chart-v12.pdf): T11 / 46294-K, catalog evidence for future 3D operations.
- [Amana plastic O-flute chart v6](https://www.amanatool.com/pub/media/productattachments/Plastic-O-Flute-Speed-Chart-v6.pdf): reviewed plastic family transfers at listed diameters.

Sources retain URL, document revision/page, retrieval date, original units and
applicability notes. Speeds at the reference RPM are not tool maximum ratings.
Milling feed = RPM × flute count × chip load (mm/tooth). Drilling uses mm/revolution.
Machine/feed ceilings reduce RPM consistently with chip load; configurations that
cannot maintain the supported operating range remain unresolved. Manufacturer
rounded feed caps are respected. Default 0.5D stepdown, 0.35D stepover and 2° entry
are unproven engineering assumptions and require acceptance, including when the
chip-load source is manufacturer-backed. Reviewed shop recipes bind to an owner,
machine, installed tool and material, and take precedence over catalog recipes.

Profiles and pockets use bounded multiple depth passes. Holding tabs retain their
physical top Z rather than being raised on every shallow pass. Flat-cutter entries
ramp along the offset contour; compression profiles first descend through a waste
helix, subject to a swept-tool clearance check, before cutting the finished edge.
Inadequate entry/neighbor clearance is reported rather than exported. Drills must
match the hole diameter within 0.02 mm. Remaining enabled hole features need a
compatible pocket cutter. Different operations using the same tool still update
RPM; LinuxCNC tool changes apply `G43 Hn` from its existing machine tool table.

## Persistence and interfaces

`/data/cam.sqlite` is separate from authentication and uses schema version 1 with
WAL journaling. `catalog_records` stores immutable typed material, model, source,
and recipe revisions. `machines` and `installed_tools` are account-owned;
`shop_recipes` are append-only private revisions. `cutting_plans` stores the
account/workspace-bound calculation and acceptance record. Unique keys prevent
duplicate tool numbers within a machine. Future schema versions are rejected.
Legacy workspace JSON remains readable; manual mode is the default and existing
settings and placements are retained. Imported libraries migrate idempotently
at startup from authenticated workspace ownership, and on subsequent catalog access. Existing manual per-project tool edits are
retained; automatic inventory/geometry is maintained in the machine setup screen.

Authenticated endpoints (mutations enforce the existing origin policy):

- `GET /api/cam/catalog`: shared evidence plus the caller's current machine/tools/recipes.
- `PUT /api/cam/machine`: validate practical limits and save a versioned machine profile.
- `PUT /api/cam/tools/{number}`: update installed cutter facts and availability.
- `POST /api/cam/recipes`: append a shop-tested recipe revision for a specific machine/tool.
- `POST /api/cam/resolve`: calculate and store a plan outside the HTTP executor.
- `POST /api/cam/accept`: accept the displayed fingerprint, provided it is ready and current.

Workspace settings add `cam` (mode, material default, part overrides, exclusions,
machine and operation overrides); snapshots add `sheet_materials` and `cam_plan`.
The existing material-group toggle accepts an optional `material` to distinguish
equal-thickness materials while retaining legacy thickness-wide requests.

Fingerprints cover source geometry, selected parts, quantities, materials,
placements, tabs, settings, machine/tool revisions and cutting evidence. Export
checks the fingerprint and acceptance again and uses the stored resolved values.
Automatic G-code ZIPs include `cutting-plan.json` and material/thickness folders.
Catalog edits never silently change already generated cutting values; resolve
again when a plan becomes stale.

## Checks and deployment

Run `cargo test -p thetta-web -p thetta-cam -p thetta-geometry` and
`npm --prefix web run build`. Tests exercise ownership, migration, recipe revisions,
unit conversion, chip-load-preserving limits, compression geometry, swept clearance,
pass schedules, physical tab heights, exact drilling, material grouping, review,
authenticated API export and stale-plan rejection. The opt-in 37-part STEP
regression described in WEB.md remains available.

Back up the persistent volume with the service stopped before deployment; retain
both `auth.sqlite` and `cam.sqlite`, workspace JSON and STEP sources, and the
host's existing token-encryption key separately. Back up SQLite WAL files with the
database when not checkpointed. For rollback, stop the new image, restore the
pre-deployment data copy, and start the retained previous image. No CNC controller
configuration is modified, and generated programs are not sent to the machine.
