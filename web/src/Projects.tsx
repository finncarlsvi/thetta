import { useEffect, useState } from "react";
import {
  ArrowRight,
  Box,
  Cloud,
  FolderOpen,
  LayoutGrid,
  List,
  LoaderCircle,
  Plus,
  Search,
  X,
} from "lucide-react";
import { requestJson } from "./api";
import { NewWorkspace } from "./NewWorkspace";
import type { Point } from "./types";

type Project = {
  id: string;
  filename: string | null;
  parts: number;
  active: boolean;
  updated_at: number | null;
  source: "onshape" | "step" | "empty";
  preview: { outer: Point[]; holes: Point[][] }[];
};
type User = { name: string; email: string };
const title = (project: Project) =>
  project.filename?.replace(/\.(step|stp)$/i, "") || "Untitled workspace";
const sourceName = (project: Project) =>
  project.source === "onshape"
    ? "Onshape"
    : project.source === "step"
      ? "STEP upload"
      : "Empty workspace";
const date = (time: number | null) =>
  time
    ? new Date(time * 1000).toLocaleString(undefined, {
        month: "short",
        day: "numeric",
        year: "numeric",
        hour: "2-digit",
        minute: "2-digit",
      })
    : "Not saved yet";

function Preview({ project }: { project: Project }) {
  if (!project.preview.length)
    return (
      <div className="project-preview empty-preview">
        <Box size={36} strokeWidth={1} />
        <span>Ready for your parts</span>
      </div>
    );
  const count = project.preview.length;
  const columns = count === 1 ? 1 : count <= 4 ? 2 : 3;
  const rows = Math.ceil(count / columns);
  const cellW = 280 / columns,
    cellH = 140 / rows;
  return (
    <div className="project-preview">
      <svg viewBox="0 0 320 180" aria-hidden="true">
        {project.preview.map((polygon, index) => {
          const xs = polygon.outer.map((p) => p.x),
            ys = polygon.outer.map((p) => p.y);
          if (!xs.length) return null;
          const minX = Math.min(...xs),
            minY = Math.min(...ys);
          const width = Math.max(...xs) - minX,
            height = Math.max(...ys) - minY;
          const scale = Math.min(
            (cellW - 20) / Math.max(width, 1),
            (cellH - 20) / Math.max(height, 1),
          );
          const x =
            20 + (index % columns) * cellW + (cellW - width * scale) / 2;
          const y =
            20 +
            Math.floor(index / columns) * cellH +
            (cellH - height * scale) / 2;
          const path = [polygon.outer, ...polygon.holes]
            .filter((r) => r.length)
            .map(
              (ring) =>
                ring
                  .map(
                    (p, i) =>
                      `${i ? "L" : "M"}${(x + (p.x - minX) * scale).toFixed(2)},${(y + height * scale - (p.y - minY) * scale).toFixed(2)}`,
                  )
                  .join(" ") + "Z",
            )
            .join(" ");
          return <path key={index} d={path} fillRule="evenodd" />;
        })}
      </svg>
    </div>
  );
}

export function Projects({
  user,
  onOpen,
}: {
  user: User;
  onOpen: (id: string) => void;
}) {
  const [projects, setProjects] = useState<Project[] | null>(null);
  const [error, setError] = useState("");
  const [query, setQuery] = useState("");
  const [filter, setFilter] = useState("all");
  const [sort, setSort] = useState("recent");
  const [view, setView] = useState<"grid" | "list">("grid");
  const [creating, setCreating] = useState<{ id: string | null } | null>(null);
  const [refresh, setRefresh] = useState(0);
  useEffect(() => {
    let alive = true;
    const load = () =>
      requestJson<Project[]>("workspaces", "GET")
        .then((data) => {
          if (alive) {
            setProjects(data);
            setError("");
          }
        })
        .catch((e) => {
          if (alive) setError(e.message);
        });
    void load();
    window.addEventListener("focus", load);
    return () => {
      alive = false;
      window.removeEventListener("focus", load);
    };
  }, [refresh]);
  const visible = (projects || [])
    .filter(
      (project) =>
        (filter === "all" || project.source === filter) &&
        title(project)
          .toLocaleLowerCase()
          .includes(query.trim().toLocaleLowerCase()),
    )
    .sort((a, b) =>
      sort === "name"
        ? title(a).localeCompare(title(b))
        : (b.updated_at || 0) - (a.updated_at || 0),
    );
  const newButton = (
    <button
      className="primary create-project"
      onClick={() => setCreating({ id: null })}
    >
      <Plus size={18} />
      New workspace
    </button>
  );
  return (
    <div className="explorer">
      <header>
        <a className="brand" href="/" aria-label="Thetta workspaces">
          <img className="brand-mark" src="/thetta-rune.svg" alt="" />
          <strong>
            thetta<span> / </span>
          </strong>
          <span className="brand-sub">NESTING & CAM</span>
        </a>
        <div className="header-right">
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
        </div>
      </header>
      <div className="explorer-body">
        <aside className="explorer-sidebar">
          {newButton}
          <span className="explorer-nav-label">YOUR PROJECTS</span>
          <nav aria-label="Workspace filters">
            {[
              { value: "all", label: "All workspaces", icon: FolderOpen },
              { value: "onshape", label: "From Onshape", icon: Cloud },
              { value: "step", label: "STEP uploads", icon: Box },
            ].map(({ value, label, icon: Icon }) => (
              <button
                key={value}
                className={filter === value ? "selected" : ""}
                aria-pressed={filter === value}
                onClick={() => setFilter(value)}
              >
                <Icon size={17} />
                <span>{label}</span>
                <small>
                  {projects?.filter(
                    (p) => value === "all" || p.source === value,
                  ).length ?? "—"}
                </small>
              </button>
            ))}
          </nav>
          <div className="explorer-sidebar-note">
            <FolderOpen size={20} strokeWidth={1.5} />
            <span>
              Each workspace keeps its own parts, settings, and saved layout.
            </span>
          </div>
        </aside>
        <section className="explorer-content" aria-labelledby="explorer-title">
          <div className="explorer-heading">
            <div>
              <span className="eyebrow">PROJECT EXPLORER</span>
              <h1 id="explorer-title">
                {filter === "all"
                  ? "Workspaces"
                  : filter === "onshape"
                    ? "From Onshape"
                    : "STEP uploads"}
                <span>{projects ? visible.length : "—"}</span>
              </h1>
              <p>Open a project to continue where you left off.</p>
            </div>
            <div className="mobile-create">{newButton}</div>
          </div>
          <div className="explorer-toolbar">
            <label className="project-search">
              <Search size={17} />
              <input
                type="search"
                placeholder="Search workspaces"
                aria-label="Search workspaces"
                value={query}
                onChange={(e) => setQuery(e.target.value)}
              />
              {query && (
                <button aria-label="Clear search" onClick={() => setQuery("")}>
                  <X size={15} />
                </button>
              )}
            </label>
            <div className="explorer-view-controls">
              <select
                aria-label="Sort workspaces"
                value={sort}
                onChange={(e) => setSort(e.target.value)}
              >
                <option value="recent">Last modified</option>
                <option value="name">Name A–Z</option>
              </select>
              <div className="view-switch" aria-label="Workspace view">
                <button
                  aria-label="Grid view"
                  aria-pressed={view === "grid"}
                  onClick={() => setView("grid")}
                >
                  <LayoutGrid size={17} />
                </button>
                <button
                  aria-label="List view"
                  aria-pressed={view === "list"}
                  onClick={() => setView("list")}
                >
                  <List size={18} />
                </button>
              </div>
            </div>
          </div>
          {error && (
            <div className="explorer-error" role="alert">
              {error}
              <button className="text-button" onClick={() => location.reload()}>
                Retry
              </button>
            </div>
          )}
          {!projects && !error && (
            <div className="explorer-empty" role="status">
              <LoaderCircle className="spin" size={26} />
              <p>Loading workspaces…</p>
            </div>
          )}
          {projects && !visible.length && (
            <div className="explorer-empty">
              <FolderOpen size={42} strokeWidth={1} />
              <h2>
                {projects.length
                  ? "No matching workspaces"
                  : "Your projects start here"}
              </h2>
              <p>
                {projects.length
                  ? "Try a different search or project filter."
                  : "Create a workspace, then import a STEP file or connect an Onshape model."}
              </p>
              {!projects.length && newButton}
            </div>
          )}
          {!!visible.length && (
            <div
              className={`project-collection ${view === "list" ? "project-list" : "project-grid"}`}
            >
              {visible.map((project) => (
                <button
                  key={project.id}
                  className="project-card"
                  onClick={() =>
                    project.source === "empty"
                      ? setCreating({ id: project.id })
                      : onOpen(project.id)
                  }
                  aria-label={`Open ${title(project)}`}
                  aria-describedby={`project-${project.id}-meta`}
                >
                  <Preview project={project} />
                  <div className="project-card-details">
                    <div className="project-card-source">
                      {project.source === "onshape" ? (
                        <Cloud size={12} />
                      ) : (
                        <Box size={12} />
                      )}
                      <span>{sourceName(project)}</span>
                      {project.active && (
                        <span className="last-opened">Last opened</span>
                      )}
                    </div>
                    <h2 title={title(project)}>{title(project)}</h2>
                    <div
                      className="project-card-meta"
                      id={`project-${project.id}-meta`}
                    >
                      <span>
                        {project.parts}{" "}
                        {project.parts === 1 ? "profile" : "profiles"}
                      </span>
                      <span>
                        {project.updated_at
                          ? `Modified ${date(project.updated_at)}`
                          : "Not saved yet"}
                      </span>
                    </div>
                  </div>
                  <ArrowRight size={18} className="project-open-arrow" />
                </button>
              ))}
            </div>
          )}
        </section>
      </div>
      {creating && (
        <NewWorkspace
          workspaceId={creating.id}
          onOpen={onOpen}
          onClose={() => {
            setCreating(null);
            setRefresh((value) => value + 1);
          }}
        />
      )}
    </div>
  );
}
