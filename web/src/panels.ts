import type { Part, Snapshot } from "./types";

export const materialColors = [
  "#badb98",
  "#8fc5d4",
  "#d6b18a",
  "#c3a6d9",
  "#99c7b9",
  "#d59caa",
];
export function panelEligible(part: Part, data: Snapshot) {
  const value = part.thickness_mm;
  return (
    value != null &&
    Number.isFinite(value) &&
    value > 0 &&
    value <= (data.max_thickness_mm ?? 30) + 1e-6
  );
}
export function materialThickness(part: Part, data: Snapshot) {
  const value = part.thickness_mm;
  const bucket = data.thickness_bucket_mm ?? 0.25;
  return value == null
    ? null
    : Math.max(1, Math.round(value / bucket)) * bucket;
}
export function materialGroups(data: Snapshot) {
  const groups = new Map<
    string,
    { thickness: number | null; material: string; parts: Part[] }
  >();
  for (const part of data.workspace.parts) {
    const thickness = materialThickness(part, data);
    const material =
      data.workspace.settings.cam.part_materials[String(part.piece.id)] ??
      data.workspace.settings.cam.material;
    const key = `${material}:${thickness}`;
    if (!groups.has(key)) groups.set(key, { thickness, material, parts: [] });
    groups.get(key)!.parts.push(part);
  }
  return [...groups]
    .sort(
      ([a, x], [b, y]) =>
        (x.thickness ?? Infinity) - (y.thickness ?? Infinity) ||
        a.localeCompare(b),
    )
    .map(([key, { thickness, material, parts }], index) => ({
      key,
      material,
      thickness,
      parts,
      color: materialColors[index % materialColors.length],
      eligible: parts.some((part) => panelEligible(part, data)),
      included:
        thickness != null &&
        !data.workspace.excluded_thicknesses?.includes(thickness) &&
        !data.workspace.settings.cam.excluded_groups.includes(key),
    }));
}
export function panelIncluded(part: Part, data: Snapshot) {
  const thickness = materialThickness(part, data);
  return (
    panelEligible(part, data) &&
    thickness != null &&
    !data.workspace.excluded_thicknesses?.includes(thickness) &&
    !data.workspace.settings.cam.excluded_groups.includes(
      `${data.workspace.settings.cam.part_materials[String(part.piece.id)] ?? data.workspace.settings.cam.material}:${thickness}`,
    )
  );
}
