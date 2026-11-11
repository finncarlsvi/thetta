import { useEffect, useState } from "react";
import type { Snapshot, Settings, Tool } from "./types";
import { requestJson } from "./api";
export type Machine = {
  id: string;
  name: string;
  revision: number;
  confirmed: boolean;
  rpm_min: number;
  rpm_max: number;
  feed_max: number;
  plunge_max: number;
  step_down_max: number;
  workholding: string;
  chip_removal: string;
  notes: string;
};
export type Installed = {
  definition: Tool;
  model: string | null;
  substrate: string | null;
  kind: string;
  cutting_length: number | null;
  compression_min: number | null;
  rpm_max: number | null;
  geometry_confirmed: boolean;
  available: boolean;
  revision: number;
};
export type CuttingPlan = {
  fingerprint: string;
  ready: boolean;
  accepted: boolean;
  requires_review: boolean;
  issues: string[];
  catalog_revision: string;
  panels: {
    index: number;
    material: string;
    thickness: number;
    issues: string[];
    cuts: {
      operation: string;
      tool: Tool;
      step_down: number;
      depths: number[];
      step_over: number;
      entry: string;
      ramp_angle: number;
      evidence: string;
      estimated: boolean;
      recipe: { source: string; notes: string; id: string };
    }[];
  }[];
};
export type CamSettings = {
  automatic: boolean;
  machine_id: string;
  material: string;
  part_materials: Record<string, string>;
  excluded_groups: string[];
  overrides: Record<
    string,
    {
      tool_number?: number;
      rpm?: number;
      feed?: number;
      ramp_feed?: number;
      step_down?: number;
      step_over?: number;
    }
  >;
};
type Catalog = {
  revision: string;
  materials: { id: string; name: string }[];
  models: {
    id: string;
    name: string;
    kind: string;
    diameter: number;
    flutes: number;
  }[];
  sources: { id: string; url: string; notes: string }[];
  machine: Machine;
  tools: Installed[];
  recipes: Record<string, unknown>[];
};
export const materials = [
  ["plywood", "Plywood"],
  ["mdf", "MDF"],
  ["softwood", "Softwood"],
  ["hardwood", "Hardwood"],
  ["acrylic", "Acrylic (PMMA)"],
  ["hdpe", "HDPE"],
];
export const materialName = (id: string) =>
  materials.find(([key]) => key === id)?.[1] || "Material not selected";
export function AutomaticCam({
  data,
  onChange,
  onSnapshot,
}: {
  data: Snapshot;
  onChange: (s: Settings) => void;
  onSnapshot: (s: Snapshot) => void;
}) {
  const [catalog, setCatalog] = useState<Catalog | null>(null),
    [error, setError] = useState(""),
    [busy, setBusy] = useState(false),
    [setup, setSetup] = useState(false),
    [toolEdit, setToolEdit] = useState<number | null>(null),
    [shop, setShop] = useState(false);
  const [machine, setMachine] = useState<Machine | null>(null),
    [tool, setTool] = useState<Installed | null>(null);
  const cam = data.workspace.settings.cam,
    plan = data.workspace.cam_plan;
  const reload = () =>
    requestJson<Catalog>("cam/catalog", "GET").then((c) => {
      setCatalog(c);
      setMachine(c.machine);
    });
  const inventorySignature = JSON.stringify(
    data.workspace.settings.tools.tools.map((t) => [
      t.number,
      t.name,
      t.diameter,
      t.flute_count,
    ]),
  );
  useEffect(() => {
    void reload().catch((e) => setError(e.message));
  }, [data.workspace.settings.cam.machine_id, inventorySignature]);
  const act = async (fn: () => Promise<unknown>) => {
    setBusy(true);
    setError("");
    try {
      await fn();
    } catch (e) {
      setError((e as Error).message);
    } finally {
      setBusy(false);
    }
  };
  const change = (next: Partial<CamSettings>) =>
    onChange({ ...data.workspace.settings, cam: { ...cam, ...next } });
  const locked = busy || data.running || data.busy;
  return (
    <section className="automatic-cam">
      <h3>Material-aware CAM</h3>
      <label className="field">
        <span>Cutting settings</span>
        <select
          disabled={locked}
          value={cam.automatic ? "automatic" : "manual"}
          onChange={(e) =>
            change({ automatic: e.target.value === "automatic" })
          }
        >
          <option value="manual">Manual settings</option>
          <option value="automatic">Automatic tools and settings</option>
        </select>
      </label>
      <label className="field">
        <span>Project material</span>
        <select
          disabled={locked}
          value={cam.material}
          onChange={(e) => change({ material: e.target.value })}
        >
          <option value="">Choose material</option>
          {materials.map(([id, name]) => (
            <option key={id} value={id}>
              {name}
            </option>
          ))}
        </select>
      </label>
      <p className="help">
        Part overrides are in the Nesting library. Different materials use
        separate panels even at the same thickness.
      </p>
      <button
        className="secondary full"
        disabled={locked}
        onClick={() => setSetup(!setup)}
      >
        Machine and installed tools{" "}
        {catalog?.machine.confirmed ? "· Confirmed" : "· Setup needed"}
      </button>
      {setup && catalog && machine && (
        <div className="cam-setup-editor">
          <h4>{machine.name}</h4>
          <p className="help">
            LinuxCNC command ceiling: 24,000 RPM. Configured trajectory ceiling:
            25,399.98 mm/min. Enter practical cutting limits for your spindle,
            setup and workholding.
          </p>
          {(
            [
              ["rpm_min", "Minimum operating RPM"],
              ["rpm_max", "Maximum operating RPM"],
              ["feed_max", "Maximum cutting feed · mm/min"],
              ["plunge_max", "Maximum entry/drill feed · mm/min"],
              ["step_down_max", "Maximum depth per pass · mm"],
            ] as const
          ).map(([key, label]) => (
            <label className="field" key={key}>
              <span>{label}</span>
              <input
                type="number"
                min="0"
                step="any"
                value={machine[key] || ""}
                onChange={(e) =>
                  setMachine({
                    ...machine,
                    [key]: Number(e.target.value),
                    confirmed: false,
                  })
                }
              />
            </label>
          ))}
          <label className="field">
            <span>Workholding</span>
            <input
              placeholder="e.g. vacuum table with holding tabs"
              value={machine.workholding}
              onChange={(e) =>
                setMachine({
                  ...machine,
                  workholding: e.target.value,
                  confirmed: false,
                })
              }
            />
          </label>
          <label className="field">
            <span>Chip removal / cooling</span>
            <input
              placeholder="e.g. dust extraction; air for plastics"
              value={machine.chip_removal}
              onChange={(e) =>
                setMachine({
                  ...machine,
                  chip_removal: e.target.value,
                  confirmed: false,
                })
              }
            />
          </label>
          <label>
            <input
              type="checkbox"
              checked={machine.confirmed}
              onChange={(e) =>
                setMachine({ ...machine, confirmed: e.target.checked })
              }
            />{" "}
            I have checked these operating limits and workholding.
          </label>
          <button
            disabled={locked}
            className="primary full"
            onClick={() =>
              void act(async () => {
                await requestJson("cam/machine", "PUT", machine);
                await reload();
                onSnapshot(await requestJson<Snapshot>("workspace", "GET"));
              })
            }
          >
            Save machine profile
          </button>
          <h4>Installed cutter geometry</h4>
          <button
            className="secondary full"
            disabled={locked}
            onClick={() => {
              const number =
                Math.max(0, ...catalog.tools.map((t) => t.definition.number)) +
                1;
              setToolEdit(number);
              setTool({
                definition: {
                  number,
                  name: "New cutter",
                  diameter: 0,
                  flute_count: 0,
                  feed_rate: 0,
                  plunge_rate: 0,
                  spindle_speed: 0,
                },
                model: null,
                substrate: null,
                kind: "unknown",
                cutting_length: null,
                compression_min: null,
                rpm_max: null,
                geometry_confirmed: false,
                available: true,
                revision: 0,
              });
            }}
          >
            New installed cutter
          </button>
          <select
            value={toolEdit ?? ""}
            onChange={(e) => {
              const n = Number(e.target.value);
              setToolEdit(n);
              setTool(
                structuredClone(
                  catalog.tools.find((t) => t.definition.number === n) || null,
                ),
              );
            }}
          >
            <option value="">Choose installed tool</option>
            {catalog.tools.map((t) => (
              <option key={t.definition.number} value={t.definition.number}>
                T{t.definition.number} · {t.definition.name}
                {t.geometry_confirmed ? "" : " · Geometry needed"}
              </option>
            ))}
          </select>
          {tool && (
            <>
              <label className="field">
                <span>Machine tool number</span>
                <input
                  type="number"
                  min="1"
                  step="1"
                  disabled={tool.revision > 0}
                  value={tool.definition.number}
                  onChange={(e) =>
                    setTool({
                      ...tool,
                      definition: {
                        ...tool.definition,
                        number: Number(e.target.value),
                      },
                      geometry_confirmed: false,
                    })
                  }
                />
              </label>
              <label className="field">
                <span>Tool name</span>
                <input
                  value={tool.definition.name}
                  onChange={(e) =>
                    setTool({
                      ...tool,
                      definition: { ...tool.definition, name: e.target.value },
                    })
                  }
                />
              </label>
              <label className="field">
                <span>Manufacturer model</span>
                <select
                  value={tool.model ?? ""}
                  onChange={(e) => {
                    const m = catalog.models.find(
                      (x) => x.id === e.target.value,
                    );
                    setTool({
                      ...tool,
                      model: m?.id ?? null,
                      kind: m?.kind ?? tool.kind,
                      substrate: m ? "carbide" : tool.substrate,
                      definition: {
                        ...tool.definition,
                        diameter: m?.diameter ?? tool.definition.diameter,
                        flute_count: m?.flutes ?? tool.definition.flute_count,
                      },
                      geometry_confirmed: false,
                    });
                  }}
                >
                  <option value="">Custom / not identified</option>
                  {catalog.models.map((m) => (
                    <option key={m.id} value={m.id}>
                      {m.name}
                    </option>
                  ))}
                </select>
              </label>
              <label className="field">
                <span>Cutter material</span>
                <select
                  value={tool.substrate ?? ""}
                  onChange={(e) =>
                    setTool({
                      ...tool,
                      substrate: e.target.value || null,
                      geometry_confirmed: false,
                    })
                  }
                >
                  <option value="">Unknown</option>
                  <option value="carbide">Solid carbide</option>
                  <option value="hss">High-speed steel</option>
                  <option value="other">Other · shop recipe required</option>
                </select>
              </label>
              <label className="field">
                <span>Cutter type</span>
                <select
                  value={tool.kind}
                  onChange={(e) =>
                    setTool({
                      ...tool,
                      kind: e.target.value,
                      geometry_confirmed: false,
                    })
                  }
                >
                  {[
                    "unknown",
                    "flat",
                    "compression",
                    "o_flute",
                    "drill",
                    "ball",
                    "v_bit",
                    "planer",
                    "probe",
                  ].map((k) => (
                    <option key={k} value={k}>
                      {k.replaceAll("_", " ")}
                    </option>
                  ))}
                </select>
              </label>
              {(["diameter", "flute_count"] as const).map((key) => (
                <label className="field" key={key}>
                  <span>{key === "diameter" ? "Diameter · mm" : "Flutes"}</span>
                  <input
                    type="number"
                    min="0"
                    step="any"
                    value={tool.definition[key] || ""}
                    onChange={(e) =>
                      setTool({
                        ...tool,
                        definition: {
                          ...tool.definition,
                          [key]: Number(e.target.value),
                        },
                        geometry_confirmed: false,
                      })
                    }
                  />
                </label>
              ))}
              {(
                [
                  ["cutting_length", "Usable cutting reach · mm"],
                  ["rpm_max", "Tool maximum RPM"],
                  ...(tool.kind === "compression"
                    ? [["compression_min", "Compression transition depth · mm"]]
                    : []),
                ] as [
                  keyof Pick<
                    Installed,
                    "cutting_length" | "rpm_max" | "compression_min"
                  >,
                  string,
                ][]
              ).map(([key, label]) => (
                <label className="field" key={key}>
                  <span>{label}</span>
                  <input
                    type="number"
                    min="0"
                    step="any"
                    value={tool[key] ?? ""}
                    onChange={(e) =>
                      setTool({
                        ...tool,
                        [key]: e.target.value ? Number(e.target.value) : null,
                        geometry_confirmed: false,
                      })
                    }
                  />
                </label>
              ))}
              <label>
                <input
                  type="checkbox"
                  checked={tool.available}
                  onChange={(e) =>
                    setTool({ ...tool, available: e.target.checked })
                  }
                />{" "}
                Available on this machine
              </label>
              <br />
              <label>
                <input
                  type="checkbox"
                  checked={tool.geometry_confirmed}
                  onChange={(e) =>
                    setTool({ ...tool, geometry_confirmed: e.target.checked })
                  }
                />{" "}
                Cutter geometry and usable reach checked
              </label>
              <button
                className="secondary full"
                disabled={locked}
                onClick={() =>
                  void act(async () => {
                    const saved = await requestJson<Installed>(
                      `cam/tools/${tool.definition.number}`,
                      "PUT",
                      tool,
                    );
                    setTool(saved);
                    setToolEdit(saved.definition.number);
                    await reload();
                    onSnapshot(await requestJson<Snapshot>("workspace", "GET"));
                  })
                }
              >
                Save installed tool
              </button>
            </>
          )}
          <button className="text-button full" onClick={() => setShop(!shop)}>
            Shop-tested material recipes
          </button>
          {shop && (
            <ShopRecipe
              catalog={catalog}
              onSave={(r) =>
                act(async () => {
                  await requestJson("cam/recipes", "POST", r);
                  await reload();
                  onSnapshot(await requestJson<Snapshot>("workspace", "GET"));
                })
              }
            />
          )}
        </div>
      )}
      {cam.automatic && (
        <>
          <button
            className="primary full"
            disabled={locked}
            onClick={() =>
              void act(async () =>
                onSnapshot(await requestJson<Snapshot>("cam/resolve", "POST")),
              )
            }
          >
            Calculate cutting plan
          </button>
          {plan && (
            <div className="cutting-plan">
              <p>
                {plan.ready
                  ? plan.requires_review && !plan.accepted
                    ? "Review starting estimates"
                    : "Cutting plan ready"
                  : "Setup required"}
              </p>
              {plan.issues.map((x, i) => (
                <p role="status" key={i}>
                  {x}
                </p>
              ))}
              {plan.panels.map((p) => (
                <article key={p.index}>
                  <strong>
                    Panel {p.index + 1} · {materialName(p.material)} ·{" "}
                    {p.thickness} mm
                  </strong>
                  {p.issues.map((x, i) => (
                    <p key={i}>{x}</p>
                  ))}
                  {p.cuts.map((c) => (
                    <div key={c.operation} className="cutting-operation">
                      <b>
                        {c.operation} · T{c.tool.number}
                      </b>
                      <dl>
                        <div>
                          <dt>Spindle</dt>
                          <dd>{c.tool.spindle_speed} RPM</dd>
                        </div>
                        <div>
                          <dt>Feed / entry</dt>
                          <dd>
                            {Math.round(c.tool.feed_rate)} /{" "}
                            {Math.round(c.tool.plunge_rate)} mm/min
                          </dd>
                        </div>
                        <div>
                          <dt>Depth per pass</dt>
                          <dd>
                            {c.step_down.toFixed(2)} mm · {c.depths.length}{" "}
                            passes
                          </dd>
                        </div>
                        <div>
                          <dt>Stepover</dt>
                          <dd>{Math.round(c.step_over * 100)}% D</dd>
                        </div>
                        <div>
                          <dt>Entry</dt>
                          <dd>
                            {c.entry}{" "}
                            {c.operation !== "drill" ? `${c.ramp_angle}°` : ""}
                          </dd>
                        </div>
                      </dl>
                      <small>{c.evidence}</small>
                      <p className="help">{c.recipe.notes}</p>
                      {(() => {
                        const src = catalog?.sources.find(
                          (s) => s.id === c.recipe.source,
                        );
                        return src?.url ? (
                          <a href={src.url} target="_blank" rel="noreferrer">
                            Manufacturer data
                          </a>
                        ) : null;
                      })()}
                      <OverrideEditor
                        value={
                          cam.overrides[`${p.material}:${c.operation}`] ?? {}
                        }
                        tools={catalog?.tools ?? []}
                        disabled={locked}
                        onSave={(o) => {
                          const k = `${p.material}:${c.operation}`,
                            overrides = { ...cam.overrides };
                          if (Object.keys(o).length) overrides[k] = o;
                          else delete overrides[k];
                          change({ overrides });
                        }}
                      />
                    </div>
                  ))}
                </article>
              ))}
              {plan.ready && plan.requires_review && !plan.accepted && (
                <>
                  <p className="help">
                    Feeds may be manufacturer-backed; pass depth and entry
                    assumptions are starting estimates. Check the displayed plan
                    against your setup.
                  </p>
                  <button
                    className="secondary full"
                    disabled={locked}
                    onClick={() =>
                      void act(async () =>
                        onSnapshot(
                          await requestJson<Snapshot>("cam/accept", "POST", {
                            fingerprint: plan.fingerprint,
                          }),
                        ),
                      )
                    }
                  >
                    Accept this starting plan
                  </button>
                </>
              )}
            </div>
          )}
        </>
      )}
      {error && <p role="alert">{error}</p>}
    </section>
  );
}
function ShopRecipe({
  catalog,
  onSave,
}: {
  catalog: Catalog;
  onSave: (r: unknown) => Promise<unknown>;
}) {
  const [number, setNumber] = useState(0),
    [material, setMaterial] = useState("plywood"),
    [operation, setOperation] = useState("profile");
  const [values, setValues] = useState<Record<string, string>>({});
  const tool = catalog.tools.find((t) => t.definition.number === number);
  const field = (key: string, label: string) => (
    <label className="field" key={key}>
      <span>{label}</span>
      <input
        type={key === "source" || key === "notes" ? "text" : "number"}
        step="any"
        min="0"
        value={values[key] ?? ""}
        onChange={(e) => setValues({ ...values, [key]: e.target.value })}
      />
    </label>
  );
  return (
    <div>
      <p className="help">
        Record an actual shop-tested recipe. This creates a new revision; it
        does not change an already resolved plan.
      </p>
      <select
        value={number || ""}
        onChange={(e) => setNumber(Number(e.target.value))}
      >
        <option value="">Choose tool</option>
        {catalog.tools.map((t) => (
          <option key={t.definition.number} value={t.definition.number}>
            T{t.definition.number} {t.definition.name}
          </option>
        ))}
      </select>
      <select value={material} onChange={(e) => setMaterial(e.target.value)}>
        {materials.map(([id, n]) => (
          <option key={id} value={id}>
            {n}
          </option>
        ))}
      </select>
      <select value={operation} onChange={(e) => setOperation(e.target.value)}>
        {["profile", "pocket", "drill"].map((op) => (
          <option key={op}>{op}</option>
        ))}
      </select>
      {field("rpm", "Tested RPM")}
      {field("chip", "Chip load · mm/tooth (or drilling mm/rev)")}
      {field("feed", "Tested cutting feed · mm/min")}
      {field("ramp", "Entry feed / cutting feed ratio")}
      {field("depth", "Depth per pass · mm")}
      {field("stepover", "Stepover · fraction of diameter")}
      {field("angle", "Contour ramp angle · degrees")}
      {field("source", "Source or trial reference")}
      {field("notes", "Test notes: stock, workholding, finish")}
      <button
        className="secondary full"
        disabled={!tool}
        onClick={() => {
          if (!tool) return;
          void onSave({
            id: `shop-${catalog.machine.id}-T${number}-${material}-${operation}`,
            revision: 1,
            model: tool.model,
            tool_number: number,
            machine_id: catalog.machine.id,
            family: tool.kind,
            material,
            operation,
            source: values.source ?? "",
            confidence: "shop",
            diameter: tool.definition.diameter,
            reference_rpm: Number(values.rpm),
            chip_min: Number(values.chip),
            chip_max: Number(values.chip),
            feed_per_rev: operation === "drill" ? Number(values.chip) : null,
            feed_cap: Number(values.feed),
            ramp_ratio: Number(values.ramp),
            step_down_ratio: Number(values.depth) / tool.definition.diameter,
            step_over: Number(values.stepover),
            ramp_angle: Number(values.angle),
            notes: values.notes ?? "",
          });
        }}
      >
        Save shop-tested recipe
      </button>
    </div>
  );
}

function OverrideEditor({
  value,
  tools,
  disabled,
  onSave,
}: {
  value: CamSettings["overrides"][string];
  tools: Installed[];
  disabled: boolean;
  onSave: (v: CamSettings["overrides"][string]) => void;
}) {
  const [draft, setDraft] = useState({ ...value });
  return (
    <details>
      <summary>Override this material / operation</summary>
      {(["rpm", "feed", "ramp_feed", "step_down", "step_over"] as const).map(
        (key) => (
          <label className="field" key={key}>
            <span>{key.replaceAll("_", " ")}</span>
            <input
              type="number"
              min="0"
              step="any"
              value={draft[key] ?? ""}
              onChange={(e) => {
                const n = { ...draft };
                if (e.target.value) n[key] = Number(e.target.value);
                else delete n[key];
                setDraft(n);
              }}
            />
          </label>
        ),
      )}
      <label className="field">
        <span>Tool</span>
        <select
          value={draft.tool_number ?? ""}
          onChange={(e) => {
            const n = { ...draft };
            if (e.target.value) n.tool_number = Number(e.target.value);
            else delete n.tool_number;
            setDraft(n);
          }}
        >
          <option value="">Automatic choice</option>
          {tools.map((t) => (
            <option key={t.definition.number} value={t.definition.number}>
              T{t.definition.number} {t.definition.name}
            </option>
          ))}
        </select>
      </label>
      <button
        disabled={disabled}
        className="secondary full"
        onClick={() => onSave(draft)}
      >
        Save overrides and recalculate
      </button>
    </details>
  );
}
