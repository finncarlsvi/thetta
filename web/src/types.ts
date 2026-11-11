import type { CamSettings, CuttingPlan } from "./AutomaticCam";
export type Point = { x: number; y: number };
export type Piece = {
  id: number;
  polygon: { outer: Point[]; holes: Point[][] };
  quantity: number;
};
export type Part = {
  piece: Piece;
  enabled: boolean;
  rotation: string;
  thickness_mm: number | null;
};
export type Tool = {
  number: number;
  name: string;
  diameter: number;
  flute_count: number;
  feed_rate: number;
  plunge_rate: number;
  spindle_speed: number;
};
export type Settings = {
  cam: CamSettings;
  sheet: { width: number; height: number; separation: number };
  optimizer: {
    samples_per_move: number;
    refinement_depth: number;
    penalty_increment: number;
    compression_step: number;
    time_limit_secs: number;
  };
  mode: string;
  tabs: {
    tab_distance: number;
    min_tabs: number;
    max_tabs: number;
    tab_width: number;
    tab_height: number;
    size_threshold: number;
    corner_exclusion: number;
    material_thickness: number;
    material_type: string;
    auto_size: boolean;
  };
  tools: { tools: Tool[] };
  gcode: {
    safe_z: number;
    cut_depth: number;
    tool_number: number | null;
    coordinate_system: string;
  };
  profile: { lead_in_radius: number; cutting_direction: string };
  drill: {
    peck_depth: number;
    tool_number: number | null;
    enabled: boolean;
    circularity_threshold: number;
  };
  pocket: {
    step_over: number;
    tool_number: number | null;
    enabled: boolean;
    finish_pass: boolean;
  };
};
export type Placement = {
  piece_id: number;
  x: number;
  y: number;
  rotation: number;
};
export type Snapshot = {
  workspace: {
    filename: string | null;
    thickness: number | null;
    groups: [number, number][];
    parts: Part[];
    settings: Settings;
    result: {
      sheets: {
        placements: Placement[];
        utilization: number;
        strip_length: number;
      }[];
      unplaced: number[];
    } | null;
    sheet_thicknesses: number[];
    sheet_materials: string[];
    cam_plan: CuttingPlan | null;
    excluded_thicknesses: number[];
    tabs: {
      sheet: number;
      placement: number;
      tabs: { position: Point; width: number; height: number }[];
    }[];
    revision: number;
    onshape: { url: string; name: string; imported_at: number } | null;
  };
  pieces: Piece[];
  max_thickness_mm: number;
  thickness_bucket_mm: number;
  running: boolean;
  busy: boolean;
  status: string;
  iteration: number;
  elapsed: number;
};
