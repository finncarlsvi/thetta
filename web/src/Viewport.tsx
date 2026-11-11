import { useEffect, useMemo, useRef, useState } from "react";
import { Maximize, Minus, Plus } from "lucide-react";
import { materialName } from "./AutomaticCam";
import type { Snapshot, Point } from "./types";
import { materialGroups } from "./panels";
const colors = [
  "#E67E22",
  "#2E86AB",
  "#2A9D8F",
  "#BC4749",
  "#8D6E63",
  "#3A86FF",
];

export function Viewport({
  data,
  selected,
  onSelect,
}: {
  data: Snapshot;
  selected: number | null;
  onSelect: (id: number | null) => void;
}) {
  const ref = useRef<HTMLCanvasElement>(null),
    wrap = useRef<HTMLDivElement>(null);
  const hit = useRef<{ path: Path2D; id: number }[]>([]);
  const panelHit = useRef<{ x: number; y: number; w: number; h: number }[]>([]);
  const zoomMax = useRef(20);
  const [size, setSize] = useState({ w: 800, h: 600 });
  const [view, setView] = useState({ zoom: 1, x: 0, y: 0 });
  const drag = useRef<{
    x: number;
    y: number;
    vx: number;
    vy: number;
    moved: boolean;
  } | null>(null);
  const config = data.workspace.settings.sheet;
  const materials = useMemo(
    () =>
      materialGroups(data).filter(
        (group) =>
          group.eligible &&
          group.included &&
          group.parts.some((p) => p.enabled),
      ),
    [data],
  );
  const materialKey = materials.map((group) => group.key).join(",");
  useEffect(() => {
    const ob = new ResizeObserver(([e]) =>
      setSize({ w: e.contentRect.width, h: e.contentRect.height }),
    );
    if (wrap.current) ob.observe(wrap.current);
    return () => ob.disconnect();
  }, []);
  const fit = () => {
    zoomMax.current = 20;
    setView({ zoom: 1, x: 0, y: 0 });
  };
  useEffect(fit, [config.width, config.height, materialKey]);
  useEffect(() => {
    const canvas = ref.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d")!;
    const dpr = window.devicePixelRatio || 1;
    canvas.width = size.w * dpr;
    canvas.height = size.h * dpr;
    ctx.scale(dpr, dpr);
    ctx.clearRect(0, 0, size.w, size.h);
    hit.current = [];
    panelHit.current = [];
    if (!materials.length) return;
    const sheets = data.workspace.result?.sheets || [];
    const strip = data.workspace.settings.mode === "strip";
    const boardWidth = strip
      ? Math.max(config.width, ...sheets.map((sheet) => sheet.strip_length))
      : config.width;
    const panelGap = config.height * 0.15,
      padding = boardWidth * 0.07;
    const header = config.height * 0.27,
      caption = config.height * 0.15;
    const packs = materials.map((group) => {
      const indices = sheets
        .map((_, i) => i)
        .filter(
          (i) =>
            data.workspace.sheet_thicknesses[i] === group.thickness &&
            (data.workspace.sheet_materials?.[i] ?? configMaterial(data)) ===
              group.material,
        );
      const panels = indices.length ? indices : [null];
      const columns = Math.min(3, Math.ceil(Math.sqrt(panels.length)));
      const rows = Math.ceil(panels.length / columns);
      return {
        ...group,
        panels,
        columns,
        width: 2 * padding + columns * boardWidth + (columns - 1) * panelGap,
        height:
          header +
          rows * (config.height + caption) +
          (rows - 1) * panelGap +
          padding,
      };
    });
    const columns = packs.length > 1 && size.w > 500 ? 2 : 1;
    const rows = Math.ceil(packs.length / columns),
      gap = boardWidth * 0.14;
    const widths = Array.from({ length: columns }, (_, column) =>
      Math.max(
        ...packs.filter((_, i) => i % columns === column).map((p) => p.width),
      ),
    );
    const heights = Array.from({ length: rows }, (_, row) =>
      Math.max(
        ...packs
          .filter((_, i) => Math.floor(i / columns) === row)
          .map((p) => p.height),
      ),
    );
    const worldW = widths.reduce((a, b) => a + b, 0) + (columns - 1) * gap;
    const worldH = heights.reduce((a, b) => a + b, 0) + (rows - 1) * gap;
    const base = Math.min(
      Math.max(100, size.w - 64) / worldW,
      Math.max(100, size.h - 100) / worldH,
    );
    const scale = base * view.zoom;
    const origin = {
      x: (size.w - worldW * scale) / 2 + view.x,
      y: 45 + (size.h - 85 - worldH * scale) / 2 + view.y,
    };
    const at = (x: number, y: number) => ({
      x: origin.x + x * scale,
      y: origin.y + y * scale,
    });
    const pieceMap = new Map(data.pieces.map((piece) => [piece.id, piece]));
    for (let gi = 0; gi < packs.length; gi++) {
      const group = packs[gi],
        column = gi % columns,
        row = Math.floor(gi / columns);
      const gx = widths.slice(0, column).reduce((a, b) => a + b + gap, 0);
      const gy = heights.slice(0, row).reduce((a, b) => a + b + gap, 0);
      const top = at(gx, gy);
      ctx.fillStyle = "#20302455";
      ctx.fillRect(top.x, top.y, group.width * scale, group.height * scale);
      ctx.strokeStyle = group.color + "65";
      ctx.lineWidth = 1;
      ctx.strokeRect(top.x, top.y, group.width * scale, group.height * scale);
      ctx.fillStyle = group.color;
      ctx.textAlign = "left";
      ctx.font = `600 ${Math.max(11, Math.min(18, header * scale * 0.45))}px sans-serif`;
      const title = at(gx + padding, gy + header * 0.62);
      ctx.fillText(
        `${group.thickness} mm · ${materialName(group.material)}`,
        title.x,
        title.y,
      );
      ctx.font = "10px sans-serif";
      ctx.fillStyle = "#9dab97";
      const actualCount = group.panels.filter((i) => i != null).length;
      ctx.textAlign = "right";
      ctx.fillText(
        `${actualCount} ${actualCount === 1 ? "panel" : "panels"}`,
        top.x + (group.width - padding) * scale,
        title.y,
      );
      for (let local = 0; local < group.panels.length; local++) {
        const index = group.panels[local];
        const bx =
          gx + padding + (local % group.columns) * (boardWidth + panelGap);
        const by =
          gy +
          header +
          Math.floor(local / group.columns) *
            (config.height + caption + panelGap);
        const actualWidth =
          strip && index != null
            ? Math.max(1, sheets[index].strip_length)
            : config.width;
        const board = at(bx, by),
          width = actualWidth * scale,
          height = config.height * scale;
        panelHit.current.push({ x: board.x, y: board.y, w: width, h: height });
        const screen = (p: Point) => at(bx + p.x, by + config.height - p.y);
        ctx.fillStyle = "#f4f5ee";
        ctx.fillRect(board.x, board.y, width, height);
        ctx.save();
        ctx.beginPath();
        ctx.rect(board.x, board.y, width, height);
        ctx.clip();
        const step = Math.pow(10, Math.ceil(Math.log10(45 / scale)));
        ctx.beginPath();
        ctx.strokeStyle = "#dde2d6";
        ctx.lineWidth = 0.6;
        for (let x = step; x < actualWidth; x += step) {
          const p = screen({ x, y: 0 });
          ctx.moveTo(p.x, board.y);
          ctx.lineTo(p.x, board.y + height);
        }
        for (let y = step; y < config.height; y += step) {
          const p = screen({ x: 0, y });
          ctx.moveTo(board.x, p.y);
          ctx.lineTo(board.x + width, p.y);
        }
        ctx.stroke();
        for (const placement of index == null ? [] : sheets[index].placements) {
          const piece = pieceMap.get(placement.piece_id);
          if (!piece) continue;
          const path = new Path2D();
          const c = Math.cos(placement.rotation),
            s = Math.sin(placement.rotation);
          for (const ring of [piece.polygon.outer, ...piece.polygon.holes]) {
            ring.forEach((p, i) => {
              const q = screen({
                x: p.x * c - p.y * s + placement.x,
                y: p.x * s + p.y * c + placement.y,
              });
              if (!i) path.moveTo(q.x, q.y);
              else path.lineTo(q.x, q.y);
            });
            path.closePath();
          }
          const parent = Math.floor((piece.id - 1) / 1001);
          ctx.fillStyle = colors[parent % colors.length] + "b5";
          ctx.fill(path, "evenodd");
          ctx.strokeStyle =
            parent === selected ? "#102b1e" : colors[parent % colors.length];
          ctx.lineWidth = parent === selected ? 2.5 : 0.8;
          ctx.stroke(path);
          hit.current.push({ path, id: parent });
        }
        for (const set of data.workspace.tabs.filter((t) => t.sheet === index))
          for (const tab of set.tabs) {
            const q = screen(tab.position);
            ctx.beginPath();
            ctx.arc(
              q.x,
              q.y,
              Math.max(2, Math.min(5, (tab.width * scale) / 2)),
              0,
              Math.PI * 2,
            );
            ctx.fillStyle = "#e6b64a";
            ctx.fill();
            ctx.strokeStyle = "#6d4f19";
            ctx.stroke();
          }
        if (index == null) {
          ctx.fillStyle = "#7c8e73";
          ctx.font = "11px sans-serif";
          ctx.textAlign = "center";
          ctx.fillText(
            data.running
              ? "Waiting for this material…"
              : data.workspace.result
                ? "Parts not placed"
                : "Ready to panelize",
            board.x + width / 2,
            board.y + height / 2,
          );
        }
        ctx.restore();
        ctx.fillStyle = "#9eac96";
        ctx.font = "9px sans-serif";
        ctx.textAlign = "left";
        const label =
          index == null
            ? `${group.thickness} mm material`
            : `PANEL ${String(index + 1).padStart(2, "0")}`;
        const detailedLabel =
          index == null
            ? label
            : `${label} · ${Number(actualWidth.toFixed(1))} × ${config.height} mm`;
        ctx.fillText(
          ctx.measureText(detailedLabel).width <= width ? detailedLabel : label,
          board.x,
          board.y + height + Math.max(12, caption * scale * 0.55),
        );
      }
    }
  }, [data, selected, size, view, config, materials]);
  return (
    <div className="viewport" ref={wrap}>
      <div className="view-caption">
        <i /> ALL PANELS <span>XY / mm</span>
      </div>
      <canvas
        ref={ref}
        aria-label="All panels grouped into arrays by material thickness"
        style={{ width: "100%", height: "100%" }}
        onWheel={(e) =>
          setView((v) => ({
            ...v,
            zoom: Math.min(
              zoomMax.current,
              Math.max(0.2, v.zoom * Math.exp(-e.deltaY * 0.001)),
            ),
          }))
        }
        onPointerDown={(e) => {
          e.currentTarget.setPointerCapture(e.pointerId);
          drag.current = {
            x: e.clientX,
            y: e.clientY,
            vx: view.x,
            vy: view.y,
            moved: false,
          };
        }}
        onPointerMove={(e) => {
          const d = drag.current;
          if (!d) return;
          if (Math.abs(e.clientX - d.x) + Math.abs(e.clientY - d.y) > 4)
            d.moved = true;
          if (d.moved)
            setView((v) => ({
              ...v,
              x: d.vx + e.clientX - d.x,
              y: d.vy + e.clientY - d.y,
            }));
        }}
        onPointerUp={(e) => {
          if (drag.current && !drag.current.moved) {
            const r = e.currentTarget.getBoundingClientRect(),
              ctx = e.currentTarget.getContext("2d")!;
            ctx.save();
            ctx.setTransform(1, 0, 0, 1, 0, 0);
            const found = [...hit.current]
              .reverse()
              .find((p) =>
                ctx.isPointInPath(
                  p.path,
                  e.clientX - r.left,
                  e.clientY - r.top,
                  "evenodd",
                ),
              );
            ctx.restore();
            onSelect(found?.id ?? null);
          }
          drag.current = null;
        }}
        onDoubleClick={(e) => {
          const rect = e.currentTarget.getBoundingClientRect();
          const x = e.clientX - rect.left,
            y = e.clientY - rect.top;
          const panel = panelHit.current.find(
            (p) => x >= p.x && x <= p.x + p.w && y >= p.y && y <= p.y + p.h,
          );
          if (!panel) {
            fit();
            return;
          }
          const ratio = Math.min(
            Math.max(100, size.w - 64) / panel.w,
            Math.max(100, size.h - 100) / panel.h,
          );
          const zoom = view.zoom * ratio;
          // Keep fitting reliable even with hundreds of panels in the overview.
          zoomMax.current = Math.max(20, zoom * 4);
          setView({
            zoom,
            x: ratio * (size.w / 2 + view.x - panel.x - panel.w / 2),
            y: ratio * ((size.h + 5) / 2 + view.y - panel.y - panel.h / 2),
          });
        }}
      />
      {!materials.length && (
        <div className="canvas-empty">
          <h2>No parts selected</h2>
          <p>
            Enable a thickness group up to {data.max_thickness_mm} mm in the
            parts list.
          </p>
        </div>
      )}
      <div className="view-tools">
        <button
          title="Zoom out"
          aria-label="Zoom out"
          onClick={() =>
            setView((v) => ({ ...v, zoom: Math.max(0.2, v.zoom / 1.25) }))
          }
        >
          <Minus size={16} />
        </button>
        <span>{Math.round(view.zoom * 100)}%</span>
        <button
          title="Zoom in"
          aria-label="Zoom in"
          onClick={() =>
            setView((v) => ({
              ...v,
              zoom: Math.min(zoomMax.current, v.zoom * 1.25),
            }))
          }
        >
          <Plus size={16} />
        </button>
        <i />
        <button
          title="Fit all panels"
          aria-label="Fit all panels"
          onClick={fit}
        >
          <Maximize size={16} />
        </button>
      </div>
    </div>
  );
}

function configMaterial(data: Snapshot) {
  return data.workspace.settings.cam.material;
}
