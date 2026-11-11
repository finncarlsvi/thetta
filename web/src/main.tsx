import React, { useEffect, useState } from "react";
import { createRoot } from "react-dom/client";
import {
  ArrowDownToLine,
  Box,
  Check,
  Layers,
  LoaderCircle,
  Play,
  Plus,
  Settings2,
  Square,
  Trash2,
  Undo2,
  X,
  Wrench,
  PanelLeftClose,
  FolderOpen,
  ChevronRight,
} from "lucide-react";
import { Viewport } from "./Viewport";
import { materials, materialName } from "./AutomaticCam";
import { ToolLibrary, CamInspector } from "./CamPanels";
import { Projects } from "./Projects";
import { requestJson } from "./api";
import { materialGroups, panelIncluded } from "./panels";
import type { Snapshot, Settings, Part, Tool } from "./types";
import "./style.css";
type User = { id: string; email: string; name: string };
type AuthStatus = { authenticated: boolean; configured: boolean; user?: User };
const api = (path: string, method = "POST", body?: unknown) =>
  requestJson<Snapshot>(path, method, body);
function AuthShell() {
  const [auth, setAuth] = useState<AuthStatus | null>(null);
  const [error, setError] = useState("");
  const signInReason = new URLSearchParams(location.search).get("signin");
  const signInMessage =
    signInReason === "expired"
      ? "Your sign-in link expired. Sign in again to continue."
      : signInReason === "not-approved"
        ? "This Onshape account is not approved for Thetta. Use your approved account."
        : signInReason === "denied"
          ? "Onshape sign-in was cancelled. You can try again."
          : signInReason
            ? "Unable to complete Onshape sign-in. Please try again."
            : "";
  useEffect(() => {
    const check = () =>
      requestJson<AuthStatus>("auth/status", "GET")
        .then(setAuth)
        .catch((e) => setError(e.message));
    void check();
    const logout = () => {
      setAuth((a) => (a ? { ...a, authenticated: false } : a));
    };
    window.addEventListener("thetta-signed-out", logout);
    return () => window.removeEventListener("thetta-signed-out", logout);
  }, []);
  if (auth?.authenticated && auth.user)
    return <ProjectShell user={auth.user} />;
  return (
    <div className="login-page">
      <div className="login-card">
        <img src="/thetta-rune.svg" className="login-logo" alt="Thetta rune" />
        <h1>thetta</h1>
        <span className="eyebrow">NESTING & CAM</span>
        <p>Sign in with your approved Onshape account.</p>
        {auth?.configured ? (
          <a className="primary" href="/api/auth/onshape/start">
            Sign in with Onshape
          </a>
        ) : (
          <p className="muted">
            {auth
              ? "Onshape sign-in is awaiting app registration."
              : "Connecting…"}
          </p>
        )}
        {signInMessage && <p role="alert">{signInMessage}</p>}
        {error && <p role="alert">{error}</p>}
      </div>
    </div>
  );
}
function ProjectShell({ user }: { user: User }) {
  const readWorkspace = () =>
    /^#\/workspace\/([a-f0-9-]{36})$/i.exec(location.hash)?.[1] || null;
  const [workspace, setWorkspace] = useState(readWorkspace);
  useEffect(() => {
    const changed = () => setWorkspace(readWorkspace());
    window.addEventListener("hashchange", changed);
    return () => window.removeEventListener("hashchange", changed);
  }, []);
  return workspace ? (
    <WorkspacePage key={workspace} user={user} id={workspace} />
  ) : (
    <Projects
      user={user}
      onOpen={(id) => {
        location.hash = `/workspace/${id}`;
      }}
    />
  );
}
function WorkspacePage({ user, id }: { user: User; id: string }) {
  const [ready, setReady] = useState(false);
  const [error, setError] = useState("");
  useEffect(() => {
    let alive = true;
    requestJson("workspaces", "POST", { id })
      .then(() => {
        if (alive) setReady(true);
      })
      .catch((e) => {
        if (alive) setError(e.message);
      });
    return () => {
      alive = false;
    };
  }, [id]);
  if (ready) return <App user={user} />;
  return (
    <div className="loading">
      <img className="brand-mark" src="/thetta-rune.svg" alt="Thetta" />
      <p role={error ? "alert" : "status"}>
        {error || "Opening your workspace…"}
      </p>
      {error && (
        <a className="secondary" href="#/">
          All workspaces
        </a>
      )}
    </div>
  );
}
function App({ user }: { user: User }) {
  const [data, setData] = useState<Snapshot | null>(null),
    [error, setError] = useState(""),
    [connected, setConnected] = useState(false),
    [tab, setTab] = useState("nesting"),
    [selected, setSelected] = useState<number | null>(null),
    [pending, setPending] = useState(false),
    [draft, setDraft] = useState<Settings | null>(null),
    [modal, setModal] = useState(false),
    [settingsTab, setSettingsTab] = useState("sheet"),
    [dirty, setDirty] = useState(false),
    [mobileParts, setMobileParts] = useState(false);
  const [expandedGroups, setExpandedGroups] = useState<Set<string>>(
    () => new Set(),
  );
  useEffect(() => {
    let stream: EventSource | undefined;
    let retry: ReturnType<typeof setTimeout> | undefined;
    let alive = true;
    const connect = async () => {
      try {
        const snapshot = await api("workspace");
        if (!alive) return;
        setData(snapshot);
        stream = new EventSource("/api/events");
        stream.onopen = () => setConnected(true);
        stream.onmessage = (event) => {
          setData(JSON.parse(event.data));
          setConnected(true);
        };
        stream.onerror = () => {
          setConnected(false);
          stream?.close();
          if (alive) retry = setTimeout(connect, 2000);
        };
      } catch (e) {
        if (alive) {
          setError((e as Error).message);
          retry = setTimeout(connect, 3000);
        }
      }
    };
    void connect();
    return () => {
      alive = false;
      stream?.close();
      if (retry) clearTimeout(retry);
    };
  }, []);
  async function act(f: () => Promise<Snapshot>) {
    setError("");
    setPending(true);
    try {
      setData(await f());
    } catch (e) {
      setError((e as Error).message);
    } finally {
      setPending(false);
    }
  }
  async function download(kind: string) {
    setPending(true);
    setError("");
    try {
      const r = await fetch("/api/export/" + kind, { method: "POST" });
      if (r.status === 401)
        window.dispatchEvent(new Event("thetta-signed-out"));
      if (!r.ok) throw new Error((await r.json()).error);
      const url = URL.createObjectURL(await r.blob());
      const a = document.createElement("a");
      a.href = url;
      a.download = `thetta-${kind}.zip`;
      a.click();
      setTimeout(() => URL.revokeObjectURL(url), 10000);
    } catch (e) {
      setError((e as Error).message);
    } finally {
      setPending(false);
    }
  }
  if (!data)
    return (
      <div className="loading">
        <img
          className="brand-mark"
          src="/thetta-rune.svg"
          alt="Thetta — ᚦ rune"
        />
        <p>{error || "Opening your workspace…"}</p>
        {error && <button onClick={() => location.reload()}>Retry</button>}
      </div>
    );
  const w = data.workspace,
    s = w.settings,
    locked = data.busy || data.running || pending,
    parts = w.parts,
    active = parts.filter((p) => p.enabled && panelIncluded(p, data)),
    groupedParts = materialGroups(data),
    eligibleGroups = groupedParts.filter(
      (group) =>
        group.eligible && group.included && group.parts.some((p) => p.enabled),
    ),
    result = w.result,
    allSheets = result?.sheets || [];
  const placed = allSheets.reduce((n, s) => n + s.placements.length, 0),
    total = active.reduce((n, p) => n + p.piece.quantity, 0);
  const area = (ring: { x: number; y: number }[]) =>
    Math.abs(
      ring.reduce(
        (a, p, i) =>
          a +
          p.x * ring[(i + 1) % ring.length].y -
          ring[(i + 1) % ring.length].x * p.y,
        0,
      ),
    ) / 2;
  const usedArea = allSheets
    .flatMap((s) => s.placements)
    .reduce((n, p) => {
      const piece = data.pieces.find((x) => x.id === p.piece_id);
      return (
        n +
        (piece
          ? area(piece.polygon.outer) -
            piece.polygon.holes.reduce((n, r) => n + area(r), 0)
          : 0)
      );
    }, 0);
  const denominator = allSheets.reduce(
    (n, sh) =>
      n +
      (s.mode === "strip" ? sh.strip_length : s.sheet.width) * s.sheet.height,
    0,
  );
  const utilization = denominator ? (100 * usedArea) / denominator : 0;
  const editPart = (p: Part, change: Record<string, unknown>) =>
    act(() =>
      api("parts/" + p.piece.id, "PUT", {
        quantity: p.piece.quantity,
        enabled: p.enabled,
        rotation: p.rotation,
        ...change,
      }),
    );
  const openSettings = (name: string) => {
    setDraft(structuredClone(s));
    setDirty(false);
    setSettingsTab(name);
    setModal(true);
  };
  const update = (group: keyof Settings, key: string, value: unknown) => {
    setDraft((d) =>
      d ? { ...d, [group]: { ...(d[group] as object), [key]: value } } : d,
    );
    setDirty(true);
  };
  const saveSettings = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!draft) return;
    setPending(true);
    setError("");
    try {
      setData(await api("settings", "PUT", draft));
      setModal(false);
      setDirty(false);
    } catch (e) {
      setError((e as Error).message);
    } finally {
      setPending(false);
    }
  };
  const toolSelect = (group: "gcode" | "drill" | "pocket") => (
    <select
      aria-label={group + " tool"}
      value={draft![group].tool_number ?? ""}
      onChange={(e) =>
        update(
          group,
          "tool_number",
          e.target.value ? Number(e.target.value) : null,
        )
      }
    >
      <option value="">Select tool</option>
      {draft!.tools.tools.map((t) => (
        <option key={t.number} value={t.number}>
          T{t.number} · {t.name}
        </option>
      ))}
    </select>
  );
  const enums: Record<string, string[]> = {
    material_type: ["WoodSoft", "WoodHard", "Plastic", "Composite"],
    coordinate_system: ["G54", "G55", "G56", "G57", "G58", "G59"],
    cutting_direction: ["Conventional", "Climb"],
  };
  const order: Record<string, string[]> = {
    sheet: ["width", "height", "separation"],
    optimizer: [
      "time_limit_secs",
      "samples_per_move",
      "refinement_depth",
      "compression_step",
      "penalty_increment",
    ],
    tabs: [
      "auto_size",
      "material_type",
      "material_thickness",
      "tab_distance",
      "min_tabs",
      "max_tabs",
      "tab_width",
      "tab_height",
      "size_threshold",
      "corner_exclusion",
    ],
  };
  const labels: Record<string, string> = {
    time_limit_secs: "Time limit (seconds)",
    separation: "Part spacing",
    tool_number: "Tool",
    safe_z: "Safe Z height",
    step_over: "Step-over (diameter fraction)",
    auto_size: "Automatic tab sizing",
  };
  const fields = (group: Exclude<keyof Settings, "mode" | "tools">) => (
    <div className="form-grid">
      {Object.entries(draft![group])
        .filter(
          ([key]) =>
            !(group === "gcode" && key === "cut_depth") &&
            !(group === "tabs" && key === "material_thickness"),
        )
        .sort(
          ([a], [b]) =>
            (order[group]?.indexOf(a) ?? 0) - (order[group]?.indexOf(b) ?? 0),
        )
        .map(([key, v]) => (
          <label
            className={"field " + (typeof v === "boolean" ? "boolean" : "")}
            key={key}
          >
            <span>{labels[key] || key.replaceAll("_", " ")}</span>
            {key === "tool_number" ? (
              toolSelect(group as "gcode" | "drill" | "pocket")
            ) : enums[key] ? (
              <select
                value={v as string}
                onChange={(e) => update(group, key, e.target.value)}
              >
                {enums[key].map((v) => (
                  <option key={v} value={v}>
                    {v === "WoodSoft"
                      ? "Softwood"
                      : v === "WoodHard"
                        ? "Hardwood"
                        : v}
                  </option>
                ))}
              </select>
            ) : typeof v === "boolean" ? (
              <input
                type="checkbox"
                checked={v}
                onChange={(e) => update(group, key, e.target.checked)}
              />
            ) : (
              <input
                type="number"
                required
                step={
                  [
                    "samples_per_move",
                    "refinement_depth",
                    "min_tabs",
                    "max_tabs",
                  ].includes(key)
                    ? 1
                    : "any"
                }
                value={v as number}
                onChange={(e) =>
                  update(group, key, e.target.valueAsNumber || 0)
                }
              />
            )}
          </label>
        ))}
    </div>
  );
  return (
    <div className="app">
      <header>
        <a className="brand" href="/">
          <img
            className="brand-mark"
            src="/thetta-rune.svg"
            alt=""
            aria-hidden="true"
          />
          <strong>thetta</strong>
        </a>
        <div className="header-right">
          <a className="project-back" href="#/">
            <FolderOpen size={16} />
            Workspaces
          </a>
          <span className="account-name" title={user.email}>
            {user.name}
          </span>
          <button
            className="text-button"
            onClick={() =>
              void requestJson("auth/logout")
                .then(() => location.reload())
                .catch((e) => setError(e.message))
            }
          >
            Sign out
          </button>
          <span className={"connection " + (connected ? "" : "offline")}>
            <i />
            {connected ? "Autosave enabled" : "Reconnecting…"}
          </span>
          <button
            className="icon-button"
            title="Workspace settings"
            aria-label="Workspace settings"
            onClick={() => openSettings("sheet")}
          >
            <Settings2 size={18} />
          </button>
        </div>
      </header>
      <div className="projectbar">
        <div>
          <Box size={18} />
          <strong>{w.filename || "Untitled workspace"}</strong>
          <span className="tag">{w.filename ? "STEP" : "NEW"}</span>
        </div>
        <nav className="workspace-mode" aria-label="Workspace mode">
          <button
            type="button"
            aria-pressed={tab === "nesting"}
            className={tab === "nesting" ? "active" : ""}
            onClick={() => {
              setTab("nesting");
              setMobileParts(false);
            }}
          >
            <Layers size={20} /> Nesting
          </button>
          <button
            type="button"
            aria-pressed={tab === "cam"}
            className={tab === "cam" ? "active" : ""}
            onClick={() => {
              setTab("cam");
              setMobileParts(false);
            }}
          >
            <Wrench size={20} /> CAM
          </button>
        </nav>
        {w.filename && (
          <span className="muted">Your work is saved automatically</span>
        )}
      </div>
      <div className="workspace">
        <aside className={"parts-panel " + (mobileParts ? "mobile-open" : "")}>
          {tab === "nesting" ? (
            <>
              <div className="panel-title">
                <span>
                  PART LIBRARY{" "}
                  <small>{parts.length.toString().padStart(2, "0")}</small>
                </span>
                <button
                  className="icon-button mobile-toggle"
                  title="Close parts"
                  onClick={() => setMobileParts(false)}
                >
                  <PanelLeftClose size={16} />
                </button>
              </div>
              {w.onshape && (
                <div className="onshape-source">
                  <a
                    href={w.onshape.url}
                    target="_blank"
                    rel="noreferrer"
                    title={w.onshape.name}
                  >
                    Source: {w.onshape.name}
                  </a>
                  <button
                    className="text-button full"
                    type="button"
                    disabled={locked}
                    onClick={() => void act(() => api("onshape/refresh"))}
                  >
                    Refresh from Onshape
                  </button>
                </div>
              )}
              <div className="material-limit">
                All materials up to {data.max_thickness_mm} mm
              </div>
              <div className="parts-scroll">
                {parts.length === 0 ? (
                  <div className="parts-empty">
                    <Layers size={28} />
                    <h3>Your parts will appear here</h3>
                    <p>Choose a project from your workspaces to get started.</p>
                    <a className="text-button" href="#/">
                      Back to workspaces
                    </a>
                  </div>
                ) : (
                  groupedParts.map((group) => (
                    <section className="material-group" key={group.key}>
                      <h3
                        className="material-group-title"
                        style={{
                          borderLeftColor: group.eligible
                            ? group.color
                            : "#7c756b",
                        }}
                      >
                        <button
                          type="button"
                          className="material-group-toggle"
                          aria-expanded={expandedGroups.has(group.key)}
                          aria-controls={`material-parts-${group.key}`}
                          onClick={() =>
                            setExpandedGroups((previous) => {
                              const next = new Set(previous);
                              if (next.has(group.key)) next.delete(group.key);
                              else next.add(group.key);
                              return next;
                            })
                          }
                        >
                          <ChevronRight
                            size={14}
                            className={
                              expandedGroups.has(group.key) ? "expanded" : ""
                            }
                          />
                          <span>
                            {group.thickness == null
                              ? "Unknown thickness"
                              : `${group.thickness} mm · ${materialName(group.material)}`}
                          </span>
                          <small>
                            {group.parts.length} profiles
                            {group.eligible && !group.included
                              ? " · excluded"
                              : ""}
                          </small>
                        </button>
                        <label
                          className="material-place"
                          title={
                            group.eligible
                              ? "Include this thickness in panelization; individual part selections are preserved."
                              : "Only materials up to 30 mm can be placed."
                          }
                        >
                          <input
                            type="checkbox"
                            aria-label={`Place ${group.thickness ?? "unknown"} mm thickness`}
                            checked={group.eligible && group.included}
                            disabled={locked || !group.eligible}
                            onChange={(e) => {
                              const enabled = e.currentTarget.checked;
                              void act(() =>
                                api(`materials/${group.thickness}`, "PUT", {
                                  enabled,
                                  material: group.material,
                                }),
                              );
                            }}
                          />
                          Place
                        </label>
                      </h3>
                      <div
                        id={`material-parts-${group.key}`}
                        hidden={!expandedGroups.has(group.key)}
                      >
                        {!group.eligible && (
                          <p className="material-excluded">
                            Not panelized ·{" "}
                            {group.thickness == null
                              ? "thickness unavailable"
                              : "over 30 mm"}
                          </p>
                        )}
                        {group.eligible && !group.included && (
                          <p className="material-excluded">
                            Excluded from panelization
                          </p>
                        )}
                        {group.parts.map((p) => {
                          const i = p.piece.id;
                          return (
                            <div
                              key={p.piece.id}
                              className={
                                "part " +
                                (selected === p.piece.id ? "selected " : "") +
                                (!p.enabled ? "removed " : "") +
                                (!panelIncluded(p, data) ? "out-of-range" : "")
                              }
                              onClick={() => setSelected(p.piece.id)}
                            >
                              <div className="part-top">
                                <span
                                  className="part-swatch"
                                  style={{
                                    background: [
                                      "#E67E22",
                                      "#2E86AB",
                                      "#2A9D8F",
                                      "#BC4749",
                                      "#8D6E63",
                                      "#3A86FF",
                                    ][p.piece.id % 6],
                                  }}
                                />
                                <b>
                                  Part{" "}
                                  {(p.piece.id + 1).toString().padStart(2, "0")}
                                </b>
                                <button
                                  className="icon-button"
                                  title={
                                    p.enabled ? "Remove part" : "Restore part"
                                  }
                                  aria-label={
                                    (p.enabled ? "Remove" : "Restore") +
                                    " part " +
                                    (i + 1)
                                  }
                                  disabled={locked || !panelIncluded(p, data)}
                                  onClick={(e) => {
                                    e.stopPropagation();
                                    void editPart(p, { enabled: !p.enabled });
                                  }}
                                >
                                  {p.enabled ? (
                                    <Trash2 size={13} />
                                  ) : (
                                    <Undo2 size={14} />
                                  )}
                                </button>
                              </div>
                              <div className="part-controls">
                                <label>
                                  Qty
                                  <input
                                    aria-label={"Quantity for part " + (i + 1)}
                                    key={p.piece.quantity}
                                    type="number"
                                    min="1"
                                    max="1000"
                                    defaultValue={p.piece.quantity}
                                    disabled={
                                      locked ||
                                      !p.enabled ||
                                      !panelIncluded(p, data)
                                    }
                                    onBlur={(e) => {
                                      const quantity = Number(e.target.value);
                                      if (quantity !== p.piece.quantity)
                                        void editPart(p, { quantity });
                                    }}
                                    onKeyDown={(e) => {
                                      if (e.key === "Enter")
                                        e.currentTarget.blur();
                                    }}
                                  />
                                </label>
                                <select
                                  aria-label={"Rotation for part " + (i + 1)}
                                  disabled={
                                    locked ||
                                    !p.enabled ||
                                    !panelIncluded(p, data)
                                  }
                                  value={p.rotation}
                                  onChange={(e) =>
                                    void editPart(p, {
                                      rotation: e.target.value,
                                    })
                                  }
                                >
                                  <option value="continuous">
                                    Free rotation
                                  </option>
                                  <option value="quarter">
                                    90° increments
                                  </option>
                                  <option value="fixed">Fixed · 0°</option>
                                </select>
                                <select
                                  aria-label={`Material for part ${p.piece.id}`}
                                  disabled={locked}
                                  value={
                                    s.cam.part_materials[String(p.piece.id)] ??
                                    ""
                                  }
                                  onChange={(e) => {
                                    const overrides = {
                                      ...s.cam.part_materials,
                                    };
                                    if (e.target.value)
                                      overrides[String(p.piece.id)] =
                                        e.target.value;
                                    else delete overrides[String(p.piece.id)];
                                    void act(() =>
                                      api("settings", "PUT", {
                                        ...s,
                                        cam: {
                                          ...s.cam,
                                          part_materials: overrides,
                                        },
                                      }),
                                    );
                                  }}
                                >
                                  <option value="">Project material</option>
                                  {materials.map(([id, name]) => (
                                    <option key={id} value={id}>
                                      {name}
                                    </option>
                                  ))}
                                </select>
                              </div>
                            </div>
                          );
                        })}
                      </div>
                    </section>
                  ))
                )}
              </div>
              <div className="parts-footer">
                <span>{active.length} active profiles</span>
                <b>{total} parts</b>
              </div>
            </>
          ) : (
            <ToolLibrary
              automatic={s.cam.automatic}
              tools={s.tools.tools}
              locked={locked}
              onAdd={() => {
                const next = structuredClone(s);
                next.tools.tools.push({
                  number:
                    Math.max(0, ...next.tools.tools.map((t) => t.number)) + 1,
                  name: "6 mm end mill",
                  diameter: 6,
                  flute_count: 2,
                  feed_rate: 1500,
                  plunge_rate: 500,
                  spindle_speed: 18000,
                });
                setDraft(next);
                setDirty(true);
                setSettingsTab("tools");
                setModal(true);
              }}
              onEdit={() => openSettings("tools")}
              onClose={() => setMobileParts(false)}
            />
          )}
        </aside>
        <main>
          <div className="main-toolbar">
            <div className="stage-context">
              <button
                className="mobile-toggle"
                title={tab === "cam" ? "Show tools" : "Show parts"}
                onClick={() => setMobileParts(true)}
              >
                {tab === "cam" ? <Wrench size={16} /> : <Layers size={16} />}
              </button>
              <span>
                {tab === "cam" ? "Tooling & operations" : "Panel layout"}
              </span>
            </div>
            <button
              className="text-button"
              onClick={() => openSettings(tab === "cam" ? "cam" : "sheet")}
            >
              <Settings2 size={15} />
              <span>{tab === "cam" ? "CAM settings" : "Sheet settings"}</span>
            </button>
          </div>
          <div className="canvas-bar">
            <div>
              <span className="sheet-badge">ALL PANELS</span>
              <span>
                {eligibleGroups.length} thickness arrays · {allSheets.length}{" "}
                panels
              </span>
            </div>
            <span>
              {s.sheet.width} × {s.sheet.height} mm · {s.sheet.separation} mm
              spacing
            </span>
          </div>
          <Viewport data={data} selected={selected} onSelect={setSelected} />
          <div className="sheet-strip material-legend">
            <div>
              {eligibleGroups.map((group) => (
                <span key={group.key} style={{ color: group.color }}>
                  <i style={{ background: group.color }} />
                  {materialName(group.material)} · {group.thickness} mm ·{" "}
                  {
                    w.sheet_thicknesses.filter(
                      (t, i) =>
                        t === group.thickness &&
                        (w.sheet_materials?.[i] ?? s.cam.material) ===
                          group.material,
                    ).length
                  }{" "}
                  panels
                </span>
              ))}
            </div>
            <span className="muted">
              Scroll to zoom · drag to pan · double-click a panel to zoom
            </span>
          </div>
        </main>
        {tab === "nesting" ? (
          <aside className="inspector">
            <div className="panel-title">
              LAYOUT OVERVIEW
              <span className="tag">mm</span>
            </div>
            <div className="util">
              <div className="eyebrow">MATERIAL UTILIZATION</div>
              <div className="big-number">
                {utilization.toFixed(1)}
                <span>%</span>
              </div>
              <div className="meter">
                <i style={{ width: `${Math.min(100, utilization)}%` }} />
              </div>
              <span className="muted">
                {result
                  ? "Across all thicknesses"
                  : "Waiting for your first layout"}
              </span>
            </div>
            <div className="stats">
              <div>
                <span>Parts placed</span>
                <b>
                  {placed}
                  <small> / {total}</small>
                </b>
              </div>
              <div>
                <span>Panels used</span>
                <b>{allSheets.length || "—"}</b>
              </div>
              <div>
                <span>Unplaced parts</span>
                <b className={result?.unplaced.length ? "warning" : ""}>
                  {result?.unplaced.length ?? "—"}
                </b>
              </div>
              <div>
                <span>Elapsed time</span>
                <b>
                  {data.elapsed.toFixed(1)}
                  <small> s</small>
                </b>
              </div>
            </div>
            <div className="inspector-section">
              <div className="section-heading">
                <span>NESTING PARAMETERS</span>
                <button
                  title="Edit nesting parameters"
                  aria-label="Edit nesting parameters"
                  onClick={() => openSettings("sheet")}
                >
                  <Settings2 size={14} />
                </button>
              </div>
              <dl>
                <div>
                  <dt>Mode</dt>
                  <dd>
                    {s.mode === "sheet" ? "Multi-sheet" : "Strip packing"}
                  </dd>
                </div>
                <div>
                  <dt>Time budget</dt>
                  <dd>{s.optimizer.time_limit_secs} s</dd>
                </div>
                <div>
                  <dt>Search samples</dt>
                  <dd>{s.optimizer.samples_per_move}</dd>
                </div>
              </dl>
            </div>
            <div className="run-area">
              <div className="status-line">
                {locked ? (
                  <LoaderCircle size={13} className="spin" />
                ) : (
                  <Check size={13} />
                )}
                <span>{pending ? "Processing…" : data.status}</span>
              </div>
              <button
                className={"primary full " + (data.running ? "stop" : "")}
                disabled={
                  pending || data.busy || (!data.running && !active.length)
                }
                onClick={() =>
                  void act(() =>
                    api("nesting/" + (data.running ? "stop" : "start")),
                  )
                }
              >
                {data.running ? <Square size={15} /> : <Play size={15} />}
                {data.running
                  ? "Stop nesting"
                  : result
                    ? "Panelize again"
                    : "Panelize selected thicknesses"}
              </button>
              <button
                className="full"
                style={{ marginTop: 8 }}
                disabled={
                  locked ||
                  !result ||
                  result.unplaced.length > 0 ||
                  data.workspace.settings.mode !== "sheet"
                }
                title="Pack toward the left edge for up to 3 minutes, leaving free space on the right. Stop keeps the best layout found."
                onClick={() => void act(() => api("nesting/improve"))}
              >
                Improve layout
              </button>
              <div className="export-label">EXPORT LAYOUT</div>
              <div className="exports">
                <button
                  disabled={locked || !result}
                  onClick={() => void download("step")}
                >
                  <ArrowDownToLine size={15} />
                  STEP
                </button>
              </div>
            </div>
          </aside>
        ) : (
          <CamInspector
            onSnapshot={setData}
            data={data}
            locked={locked}
            pending={pending}
            onSettings={openSettings}
            onChange={(settings) =>
              void act(() => api("settings", "PUT", settings))
            }
            onTabs={() => void act(() => api("tabs"))}
            onExport={() => void download("gcode")}
          />
        )}
      </div>
      <footer>
        <span>
          <i />
          {data.running ? "Optimizer running" : "Ready"}
          <span className="footer-sep">/</span>
          {parts.length} profiles loaded
        </span>
      </footer>
      {error && (
        <div className="toast" role="alert">
          <span>{error}</span>
          <button aria-label="Dismiss error" onClick={() => setError("")}>
            <X size={16} />
          </button>
        </div>
      )}
      {modal && draft && (
        <div className="modal-backdrop">
          <form
            className="modal"
            role="dialog"
            aria-modal="true"
            aria-label="Workspace settings"
            onKeyDown={(e) => {
              if (e.key === "Escape") setModal(false);
            }}
            onSubmit={saveSettings}
          >
            <div className="modal-head">
              <div>
                <span className="eyebrow">WORKSPACE SETTINGS</span>
                <h2>Make it fit your workflow.</h2>
              </div>
              <button
                type="button"
                title="Close settings"
                aria-label="Close settings"
                onClick={() => setModal(false)}
              >
                <X size={20} />
              </button>
            </div>
            <div className="modal-tabs">
              {["sheet", "tabs", "tools", "cam"].map((t) => (
                <button
                  type="button"
                  key={t}
                  className={settingsTab === t ? "active" : ""}
                  onClick={() => setSettingsTab(t)}
                >
                  {t === "sheet"
                    ? "Sheet & nesting"
                    : t === "cam"
                      ? "CAM operations"
                      : t === "tabs"
                        ? "Holding tabs"
                        : "Tool library"}
                </button>
              ))}
            </div>
            <fieldset disabled={locked} className="modal-body">
              {(settingsTab === "cam" || settingsTab === "tabs") && (
                <p className="help">
                  Cutting depth and tab material thickness follow each panel's
                  material automatically.
                </p>
              )}
              {settingsTab === "sheet" && (
                <>
                  <h3>Material sheet · mm</h3>
                  {fields("sheet")}
                  <label className="field mode">
                    <span>Packing mode</span>
                    <select
                      value={draft.mode}
                      onChange={(e) => {
                        setDraft({ ...draft, mode: e.target.value });
                        setDirty(true);
                      }}
                    >
                      <option value="sheet">Multi-sheet</option>
                      <option value="strip">Strip packing</option>
                    </select>
                  </label>
                  <h3>Optimizer</h3>
                  {fields("optimizer")}
                </>
              )}
              {settingsTab === "tabs" && (
                <>
                  <h3>Holding tabs · mm</h3>
                  {fields("tabs")}
                </>
              )}
              {settingsTab === "tools" && (
                <>
                  <p className="help">
                    Tool numbers match your machine’s tool table. Dimensions are
                    in mm; feed and plunge rates in mm/min. Imported tools may
                    have unset values. Complete diameter, flutes, feeds, and
                    spindle speed before using a tool for G-code.
                  </p>
                  {draft.tools.tools.map((tool, index) => (
                    <div className="tool-card" key={index}>
                      <div className="section-heading">
                        <strong>
                          T{tool.number} · {tool.name}
                        </strong>
                        <button
                          type="button"
                          title="Delete tool"
                          onClick={() =>
                            update(
                              "tools",
                              "tools",
                              draft.tools.tools.filter((_, i) => i !== index),
                            )
                          }
                        >
                          <Trash2 size={15} />
                        </button>
                      </div>
                      <div className="form-grid">
                        {Object.entries(tool).map(([key, v]) => (
                          <label className="field" key={key}>
                            <span>{key.replaceAll("_", " ")}</span>
                            <input
                              required={key === "name" || key === "number"}
                              type={key === "name" ? "text" : "number"}
                              min={
                                key === "name"
                                  ? undefined
                                  : key === "number"
                                    ? 1
                                    : 0
                              }
                              placeholder={
                                key === "name" || key === "number"
                                  ? undefined
                                  : "Not set"
                              }
                              step={
                                [
                                  "number",
                                  "flute_count",
                                  "spindle_speed",
                                ].includes(key)
                                  ? 1
                                  : "any"
                              }
                              value={
                                key !== "name" && key !== "number" && v === 0
                                  ? ""
                                  : v
                              }
                              onChange={(e) =>
                                update(
                                  "tools",
                                  "tools",
                                  draft.tools.tools.map((t, i) =>
                                    i === index
                                      ? {
                                          ...t,
                                          [key]:
                                            key === "name"
                                              ? e.target.value
                                              : e.target.valueAsNumber || 0,
                                        }
                                      : t,
                                  ),
                                )
                              }
                            />
                          </label>
                        ))}
                      </div>
                    </div>
                  ))}
                  <button
                    type="button"
                    className="secondary"
                    onClick={() => {
                      const tool: Tool = {
                        number:
                          Math.max(
                            0,
                            ...draft.tools.tools.map((t) => t.number),
                          ) + 1,
                        name: "6 mm end mill",
                        diameter: 6,
                        flute_count: 2,
                        feed_rate: 1500,
                        plunge_rate: 500,
                        spindle_speed: 18000,
                      };
                      update("tools", "tools", [...draft.tools.tools, tool]);
                    }}
                  >
                    <Plus size={15} />
                    Add tool
                  </button>
                </>
              )}
              {settingsTab === "cam" && (
                <>
                  <h3>Cutting parameters · mm</h3>
                  {fields("gcode")}
                  <h3>Profile strategy</h3>
                  {fields("profile")}
                  <h3>Drilling</h3>
                  {fields("drill")}
                  <h3>Pocket clearing</h3>
                  {fields("pocket")}
                </>
              )}
            </fieldset>
            <div className="modal-footer">
              <span>
                {dirty ? "Unsaved changes" : "Saved to this workspace"}
              </span>
              <button
                type="button"
                className="secondary"
                onClick={() => setModal(false)}
              >
                Cancel
              </button>
              <button
                className="primary"
                disabled={locked || !dirty}
                type="submit"
              >
                {pending ? (
                  <LoaderCircle className="spin" size={15} />
                ) : (
                  <Check size={15} />
                )}
                Save settings
              </button>
            </div>
          </form>
        </div>
      )}
    </div>
  );
}
createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <AuthShell />
  </React.StrictMode>,
);
