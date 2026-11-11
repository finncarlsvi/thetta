import {
  Check,
  LoaderCircle,
  PanelLeftClose,
  Plus,
  Settings2,
  Wrench,
} from "lucide-react";
import { AutomaticCam } from "./AutomaticCam";
import type { Settings, Snapshot, Tool } from "./types";

function toolReady(tool: Tool) {
  return [
    tool.diameter,
    tool.flute_count,
    tool.feed_rate,
    tool.plunge_rate,
    tool.spindle_speed,
  ].every((value) => Number.isFinite(value) && value > 0);
}

export function ToolLibrary({
  tools,
  locked,
  automatic = false,
  onAdd,
  onEdit,
  onClose,
}: {
  tools: Tool[];
  locked: boolean;
  automatic?: boolean;
  onAdd: () => void;
  onEdit: () => void;
  onClose: () => void;
}) {
  return (
    <>
      <div className="panel-title">
        <span>
          TOOL LIBRARY <small>{tools.length}</small>
        </span>
        <button
          className="icon-button mobile-toggle"
          title="Close tools"
          onClick={onClose}
        >
          <PanelLeftClose size={16} />
        </button>
      </div>
      <div className="tool-library-actions">
        <button className="secondary full" disabled={locked} onClick={onAdd}>
          <Plus size={16} /> Add tool
        </button>
      </div>
      <div className="parts-scroll tool-library-list">
        {tools.length === 0 ? (
          <div className="parts-empty">
            <Wrench size={30} />
            <h3>No tools configured</h3>
            <p>
              Add the cutters in your machine’s tool table, then assign them to
              CAM operations.
            </p>
            <button className="primary" disabled={locked} onClick={onAdd}>
              Add your first tool
            </button>
          </div>
        ) : (
          tools.map((tool) => (
            <button
              key={tool.number}
              className="library-tool"
              disabled={locked}
              onClick={onEdit}
              title={`Edit T${tool.number}`}
            >
              <div className="library-tool-heading">
                <span className="tool-number">T{tool.number}</span>
                <strong>{tool.name}</strong>
                <Settings2 size={14} />
              </div>
              <div className="tool-diameter">
                {tool.diameter > 0
                  ? `Ø ${tool.diameter} mm`
                  : "Diameter not set"}{" "}
                <span>
                  ·{" "}
                  {tool.flute_count > 0
                    ? `${tool.flute_count} flutes`
                    : "Flutes not set"}
                </span>
                {automatic ? (
                  <span className="tag">Rates per operation</span>
                ) : (
                  !toolReady(tool) && <span className="tag">Setup needed</span>
                )}
              </div>
              <dl>
                <div>
                  <dt>Feed</dt>
                  <dd>
                    {automatic
                      ? "Automatic"
                      : tool.feed_rate > 0
                        ? `${tool.feed_rate} mm/min`
                        : "Not set"}
                  </dd>
                </div>
                <div>
                  <dt>Plunge</dt>
                  <dd>
                    {automatic
                      ? "Automatic"
                      : tool.plunge_rate > 0
                        ? `${tool.plunge_rate} mm/min`
                        : "Not set"}
                  </dd>
                </div>
                <div>
                  <dt>Spindle</dt>
                  <dd>
                    {automatic
                      ? "Automatic"
                      : tool.spindle_speed > 0
                        ? `${tool.spindle_speed} rpm`
                        : "Not set"}
                  </dd>
                </div>
              </dl>
            </button>
          ))
        )}
      </div>
      <div className="parts-footer">
        <span>Machine tool table</span>
        <b>{tools.length} tools</b>
      </div>
    </>
  );
}

export function CamInspector({
  data,
  locked,
  pending,
  onSettings,
  onSnapshot,
  onChange,
  onTabs,
  onExport,
}: {
  data: Snapshot;
  locked: boolean;
  pending: boolean;
  onSettings: (tab: string) => void;
  onSnapshot: (data: Snapshot) => void;
  onChange: (settings: Settings) => void;
  onTabs: () => void;
  onExport: () => void;
}) {
  const settings = data.workspace.settings,
    tools = settings.tools.tools,
    result = data.workspace.result;
  const operations = [
    { group: "gcode" as const, name: "Profile cutting", enabled: true },
    {
      group: "drill" as const,
      name: "Drilling",
      enabled: settings.drill.enabled,
    },
    {
      group: "pocket" as const,
      name: "Pocket clearing",
      enabled: settings.pocket.enabled,
    },
  ];
  const ready = settings.cam.automatic
    ? !!data.workspace.cam_plan?.ready &&
      (!data.workspace.cam_plan.requires_review ||
        data.workspace.cam_plan.accepted)
    : operations.every(
        (op) =>
          !op.enabled ||
          tools.some(
            (t) => t.number === settings[op.group].tool_number && toolReady(t),
          ),
      );
  return (
    <aside className="inspector cam-inspector">
      <div className="panel-title">
        CAM SETUP{" "}
        <button
          className="icon-button"
          title="Edit CAM parameters"
          aria-label="Edit CAM parameters"
          onClick={() => onSettings("cam")}
        >
          <Settings2 size={16} />
        </button>
      </div>
      <div className="cam-summary">
        <strong>
          {result?.sheets.length ?? 0} panels · {tools.length} tools
        </strong>
        <span>Cutting depth follows each panel’s thickness.</span>
      </div>
      <AutomaticCam data={data} onChange={onChange} onSnapshot={onSnapshot} />
      {!settings.cam.automatic && (
        <div className="cam-operations">
          <div className="eyebrow">OPERATIONS & TOOLS</div>
          {operations.map((op) => (
            <section className="cam-operation" key={op.group}>
              <div className="cam-operation-heading">
                <strong>{op.name}</strong>
                {op.group === "gcode" ? (
                  <span className="tag">PROFILE</span>
                ) : (
                  <label>
                    <input
                      type="checkbox"
                      aria-label={`Enable ${op.name.toLowerCase()}`}
                      checked={op.enabled}
                      disabled={locked}
                      onChange={(e) =>
                        onChange({
                          ...settings,
                          [op.group]: {
                            ...settings[op.group],
                            enabled: e.target.checked,
                          },
                        })
                      }
                    />{" "}
                    Enable
                  </label>
                )}
              </div>
              <label className="field">
                <span>Tool</span>
                <select
                  aria-label={`${op.name} tool`}
                  disabled={locked || !op.enabled || tools.length === 0}
                  value={settings[op.group].tool_number ?? ""}
                  onChange={(e) =>
                    onChange({
                      ...settings,
                      [op.group]: {
                        ...settings[op.group],
                        tool_number: e.target.value
                          ? Number(e.target.value)
                          : null,
                      },
                    })
                  }
                >
                  <option value="">
                    {tools.length
                      ? "Assign a tool"
                      : "Add a tool in the library"}
                  </option>
                  {tools.map((t) => (
                    <option key={t.number} value={t.number}>
                      T{t.number} · {t.name} ·{" "}
                      {t.diameter > 0
                        ? `Ø ${t.diameter} mm`
                        : "Diameter not set"}
                      {!toolReady(t) ? " · Setup needed" : ""}
                    </option>
                  ))}
                </select>
              </label>
            </section>
          ))}
          <button
            className="text-button full"
            onClick={() => onSettings("cam")}
          >
            Feeds, depths & advanced parameters <Settings2 size={14} />
          </button>
        </div>
      )}
      <div className="inspector-section">
        <div className="section-heading">
          <span>HOLDING TABS</span>
          <button
            title="Edit tab settings"
            aria-label="Edit tab settings"
            onClick={() => onSettings("tabs")}
          >
            <Settings2 size={14} />
          </button>
        </div>
        <dl>
          <div>
            <dt>Tab sizing</dt>
            <dd>{settings.tabs.auto_size ? "Automatic" : "Manual"}</dd>
          </div>
          <div>
            <dt>Tab distance</dt>
            <dd>{settings.tabs.tab_distance} mm</dd>
          </div>
          <div>
            <dt>Tabs generated</dt>
            <dd>
              {data.workspace.tabs.reduce((n, t) => n + t.tabs.length, 0)}
            </dd>
          </div>
        </dl>
        <button
          className="secondary full"
          disabled={locked || !result}
          onClick={onTabs}
        >
          Generate tabs
        </button>
      </div>
      <div className="run-area">
        <div className="status-line">
          {pending ? (
            <LoaderCircle size={13} className="spin" />
          ) : (
            <Check size={13} />
          )}
          <span>
            {pending
              ? "Processing…"
              : !ready
                ? "Assign tools and complete their cutting parameters."
                : "CAM setup ready"}
          </span>
        </div>
        <button
          className="primary full"
          disabled={locked || !result || !ready || !!result.unplaced.length}
          onClick={onExport}
        >
          Export G-code
        </button>
      </div>
    </aside>
  );
}
