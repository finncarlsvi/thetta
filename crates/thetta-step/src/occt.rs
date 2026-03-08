//! Safe OCCT FFI wrappers.
//!
//! All `unsafe` / raw OCCT calls are confined to this module.
//! When the `occt` feature is disabled the public items are simply absent
//! and the rest of the crate falls back to JSON-based simulation.

#[cfg(feature = "occt")]
mod inner {
    use std::path::Path;

    use opencascade_sys::ffi;
    use thetta_geometry::types::Point2D;

    use crate::export::StepExportError;
    use crate::import::StepImportError;

    // ── STEP Reader ──────────────────────────────────────────────────

    pub struct OcctStepReader {
        shape: cxx::UniquePtr<ffi::TopoDS_Shape>,
    }

    impl OcctStepReader {
        pub fn read_file(path: &Path) -> Result<Self, StepImportError> {
            let path_str = path
                .to_str()
                .ok_or_else(|| StepImportError::ReadFailed("non-UTF-8 path".into()))?
                .to_string();

            let mut reader = ffi::STEPControl_Reader_ctor();
            let status = ffi::read_step(reader.pin_mut(), path_str);
            if status != ffi::IFSelect_ReturnStatus::IFSelect_RetDone {
                return Err(StepImportError::OcctError(format!(
                    "STEPControl_Reader::ReadFile returned {:?}",
                    status
                )));
            }

            let progress = ffi::Message_ProgressRange_ctor();
            let n_roots = reader.pin_mut().TransferRoots(&progress);
            if n_roots == 0 {
                return Err(StepImportError::NoSolidsFound);
            }

            let shape = ffi::one_shape_step(&reader);
            if shape.is_null() {
                return Err(StepImportError::NoSolidsFound);
            }

            Ok(Self { shape })
        }

        pub fn root_shape(&self) -> &ffi::TopoDS_Shape {
            &self.shape
        }
    }

    // ── Solid enumeration ────────────────────────────────────────────

    pub struct SolidInfo {
        pub solid: cxx::UniquePtr<ffi::TopoDS_Shape>,
        pub thickness_mm: f64,
    }

    pub fn enumerate_solids(root: &ffi::TopoDS_Shape) -> Vec<SolidInfo> {
        let mut solids = Vec::new();
        let mut explorer =
            ffi::TopExp_Explorer_ctor(root, ffi::TopAbs_ShapeEnum::TopAbs_SOLID);

        while explorer.More() {
            let shape = ffi::ExplorerCurrentShape(&explorer);
            let thickness = estimate_thickness(&shape);
            solids.push(SolidInfo {
                solid: shape,
                thickness_mm: thickness,
            });
            explorer.pin_mut().Next();
        }
        solids
    }

    /// Estimate sheet-metal thickness from a solid.
    ///
    /// Strategy: find pairs of parallel planar faces and take the minimum
    /// distance between them. Falls back to volume / max-face-area.
    fn estimate_thickness(solid: &ffi::TopoDS_Shape) -> f64 {
        let faces = collect_planar_faces(solid);

        // Try parallel-face-pair distance
        let mut min_thickness = f64::MAX;
        for i in 0..faces.len() {
            for j in (i + 1)..faces.len() {
                let (n1, c1) = &faces[i];
                let (n2, c2) = &faces[j];
                // Check if normals are anti-parallel (dot ≈ -1)
                let dot = n1.0 * n2.0 + n1.1 * n2.1 + n1.2 * n2.2;
                if dot < -0.95 {
                    let dx = c2.0 - c1.0;
                    let dy = c2.1 - c1.1;
                    let dz = c2.2 - c1.2;
                    let dist = (dx * n1.0 + dy * n1.1 + dz * n1.2).abs();
                    if dist > 1e-6 && dist < min_thickness {
                        min_thickness = dist;
                    }
                }
            }
        }

        if min_thickness < f64::MAX {
            return min_thickness;
        }

        // Fallback: volume / largest face area
        let mut vol_props = ffi::GProp_GProps_ctor();
        ffi::BRepGProp_VolumeProperties(solid, vol_props.pin_mut());
        let volume = vol_props.Mass();

        let max_area = largest_face_area(solid);
        if max_area > 1e-12 {
            (volume / max_area).abs()
        } else {
            0.0
        }
    }

    /// Collect planar faces as (normal, center_point) tuples.
    fn collect_planar_faces(
        solid: &ffi::TopoDS_Shape,
    ) -> Vec<((f64, f64, f64), (f64, f64, f64))> {
        let mut result = Vec::new();
        let mut explorer =
            ffi::TopExp_Explorer_ctor(solid, ffi::TopAbs_ShapeEnum::TopAbs_FACE);

        while explorer.More() {
            let shape = explorer.Current();
            let face = ffi::TopoDS_cast_to_face(shape);
            let surface = ffi::BRep_Tool_Surface(face);
            let type_handle = ffi::DynamicType(&surface);
            let type_name = ffi::type_name(type_handle);

            if type_name.contains("Geom_Plane") {
                let gprop_face = ffi::BRepGProp_Face_ctor(face);
                let mut point = ffi::new_point(0.0, 0.0, 0.0);
                let mut normal = ffi::new_vec(0.0, 0.0, 0.0);
                gprop_face.Normal(0.5, 0.5, point.pin_mut(), normal.pin_mut());
                result.push((
                    (normal.X(), normal.Y(), normal.Z()),
                    (point.X(), point.Y(), point.Z()),
                ));
            }
            explorer.pin_mut().Next();
        }
        result
    }

    fn largest_face_area(solid: &ffi::TopoDS_Shape) -> f64 {
        let mut max_area = 0.0_f64;
        let mut explorer =
            ffi::TopExp_Explorer_ctor(solid, ffi::TopAbs_ShapeEnum::TopAbs_FACE);

        while explorer.More() {
            let shape = ffi::ExplorerCurrentShape(&explorer);
            let mut props = ffi::GProp_GProps_ctor();
            ffi::BRepGProp_SurfaceProperties(&shape, props.pin_mut());
            let area = props.Mass();
            if area > max_area {
                max_area = area;
            }
            explorer.pin_mut().Next();
        }
        max_area
    }

    // ── Profile extraction ───────────────────────────────────────────

    pub struct FaceProfile {
        pub outer: Vec<Point2D>,
        pub holes: Vec<Vec<Point2D>>,
    }

    /// Extract a 2D profile from a solid by finding its largest planar face
    /// and discretizing the wires.
    pub fn extract_profile(
        solid: &ffi::TopoDS_Shape,
        angular_deflection: f64,
        curvature_deflection: f64,
    ) -> Option<FaceProfile> {
        // Find the largest planar face (returned as owned TopoDS_Shape)
        let face_shape = find_largest_planar_face(solid)?;
        let face = ffi::TopoDS_cast_to_face(&face_shape);

        // Get the face plane's coordinate system for 3D -> 2D projection
        let surface = ffi::BRep_Tool_Surface(face);
        let plane_handle = ffi::new_HandleGeomPlane_from_HandleGeomSurface(&surface);
        if plane_handle.is_null() {
            return None;
        }
        let plane_origin = ffi::handle_geom_plane_location(&plane_handle);

        // Get face normal for projection axis
        let gprop_face = ffi::BRepGProp_Face_ctor(face);
        let mut norm_point = ffi::new_point(0.0, 0.0, 0.0);
        let mut face_normal = ffi::new_vec(0.0, 0.0, 0.0);
        gprop_face.Normal(0.5, 0.5, norm_point.pin_mut(), face_normal.pin_mut());

        // Build local coordinate axes (u, v) on the plane
        let nz = normalize_vec3(face_normal.X(), face_normal.Y(), face_normal.Z())?;

        // Choose u-axis: cross with world-Z unless parallel
        let world_up = if nz.2.abs() > 0.9 {
            (1.0, 0.0, 0.0)
        } else {
            (0.0, 0.0, 1.0)
        };
        let ux = normalize_cross(world_up, nz)?;
        let uy = cross(nz, ux);

        let origin = (plane_origin.X(), plane_origin.Y(), plane_origin.Z());

        // Extract outer wire
        let outer_wire = ffi::outer_wire(face);
        let outer = discretize_wire_2d(
            &outer_wire,
            origin,
            ux,
            uy,
            angular_deflection,
            curvature_deflection,
        );

        // Extract hole wires
        let mut holes = Vec::new();
        let face_as_shape = ffi::cast_face_to_shape(face);
        let mut wire_explorer =
            ffi::TopExp_Explorer_ctor(face_as_shape, ffi::TopAbs_ShapeEnum::TopAbs_WIRE);
        while wire_explorer.More() {
            let wire_shape = wire_explorer.Current();
            let wire = ffi::TopoDS_cast_to_wire(wire_shape);
            let pts = discretize_wire_2d(
                wire,
                origin,
                ux,
                uy,
                angular_deflection,
                curvature_deflection,
            );
            if !rings_similar(&pts, &outer) && pts.len() >= 3 {
                holes.push(pts);
            }
            wire_explorer.pin_mut().Next();
        }

        if outer.len() < 3 {
            return None;
        }

        Some(FaceProfile { outer, holes })
    }

    fn find_largest_planar_face(
        solid: &ffi::TopoDS_Shape,
    ) -> Option<cxx::UniquePtr<ffi::TopoDS_Shape>> {
        let mut best_shape: Option<cxx::UniquePtr<ffi::TopoDS_Shape>> = None;
        let mut best_area = 0.0_f64;
        let mut explorer =
            ffi::TopExp_Explorer_ctor(solid, ffi::TopAbs_ShapeEnum::TopAbs_FACE);

        while explorer.More() {
            let shape = explorer.Current();
            let face = ffi::TopoDS_cast_to_face(shape);
            let surface = ffi::BRep_Tool_Surface(face);
            let type_handle = ffi::DynamicType(&surface);
            let type_name = ffi::type_name(type_handle);

            if type_name.contains("Geom_Plane") {
                let owned = ffi::ExplorerCurrentShape(&explorer);
                let mut props = ffi::GProp_GProps_ctor();
                ffi::BRepGProp_SurfaceProperties(&owned, props.pin_mut());
                let area = props.Mass();

                if area > best_area {
                    best_area = area;
                    best_shape = Some(ffi::ExplorerCurrentShape(&explorer));
                }
            }
            explorer.pin_mut().Next();
        }
        best_shape
    }

    fn discretize_wire_2d(
        wire: &ffi::TopoDS_Wire,
        origin: (f64, f64, f64),
        ux: (f64, f64, f64),
        uy: (f64, f64, f64),
        angular_deflection: f64,
        curvature_deflection: f64,
    ) -> Vec<Point2D> {
        // Collect edge segments (each edge → ordered list of 2D points)
        let mut segments: Vec<Vec<Point2D>> = Vec::new();
        let wire_shape = ffi::cast_wire_to_shape(wire);
        let mut edge_explorer =
            ffi::TopExp_Explorer_ctor(wire_shape, ffi::TopAbs_ShapeEnum::TopAbs_EDGE);

        while edge_explorer.More() {
            let edge_shape = edge_explorer.Current();
            let edge = ffi::TopoDS_cast_to_edge(edge_shape);
            let curve = ffi::BRepAdaptor_Curve_ctor(edge);
            let approx = ffi::GCPnts_TangentialDeflection_ctor(
                &curve,
                angular_deflection,
                curvature_deflection,
            );

            let n = approx.NbPoints();
            let mut seg = Vec::with_capacity(n as usize);
            for i in 1..=n {
                let pt = ffi::GCPnts_TangentialDeflection_Value(&approx, i);
                let dx = pt.X() - origin.0;
                let dy = pt.Y() - origin.1;
                let dz = pt.Z() - origin.2;
                let u = dx * ux.0 + dy * ux.1 + dz * ux.2;
                let v = dx * uy.0 + dy * uy.1 + dz * uy.2;
                seg.push(Point2D { x: u, y: v });
            }
            if seg.len() >= 2 {
                segments.push(seg);
            }
            edge_explorer.pin_mut().Next();
        }

        // Chain segments into a proper ring by matching endpoints
        let points = chain_segments(segments);
        let mut result = points;
        dedup_ring(&mut result);
        result
    }

    /// Chain edge segments into a closed ring by matching nearest endpoints.
    fn chain_segments(mut segments: Vec<Vec<Point2D>>) -> Vec<Point2D> {
        if segments.is_empty() {
            return Vec::new();
        }

        let mut ring = segments.swap_remove(0);

        while !segments.is_empty() {
            let tail = ring.last().copied().unwrap();
            let mut best_idx = 0;
            let mut best_dist = f64::MAX;
            let mut reversed = false;

            for (i, seg) in segments.iter().enumerate() {
                // Check if segment start matches our tail
                let d_start = pt_dist2(&tail, &seg[0]);
                if d_start < best_dist {
                    best_dist = d_start;
                    best_idx = i;
                    reversed = false;
                }
                // Check if segment end matches (need to reverse)
                let d_end = pt_dist2(&tail, seg.last().unwrap());
                if d_end < best_dist {
                    best_dist = d_end;
                    best_idx = i;
                    reversed = true;
                }
            }

            let mut seg = segments.swap_remove(best_idx);
            if reversed {
                seg.reverse();
            }
            // Skip first point (it overlaps with ring's last point)
            if !seg.is_empty() {
                ring.extend_from_slice(&seg[1..]);
            }
        }
        ring
    }

    fn pt_dist2(a: &Point2D, b: &Point2D) -> f64 {
        let dx = a.x - b.x;
        let dy = a.y - b.y;
        dx * dx + dy * dy
    }

    fn dedup_ring(pts: &mut Vec<Point2D>) {
        pts.dedup_by(|a, b| (a.x - b.x).abs() < 1e-6 && (a.y - b.y).abs() < 1e-6);
        if pts.len() >= 2 {
            let first = pts[0];
            let last = pts[pts.len() - 1];
            if (first.x - last.x).abs() < 1e-6 && (first.y - last.y).abs() < 1e-6 {
                pts.pop();
            }
        }
    }

    fn rings_similar(a: &[Point2D], b: &[Point2D]) -> bool {
        if a.len() != b.len() {
            return false;
        }
        a.iter()
            .zip(b.iter())
            .all(|(p, q)| (p.x - q.x).abs() < 1e-4 && (p.y - q.y).abs() < 1e-4)
    }

    fn cross(a: (f64, f64, f64), b: (f64, f64, f64)) -> (f64, f64, f64) {
        (
            a.1 * b.2 - a.2 * b.1,
            a.2 * b.0 - a.0 * b.2,
            a.0 * b.1 - a.1 * b.0,
        )
    }

    fn normalize_vec3(x: f64, y: f64, z: f64) -> Option<(f64, f64, f64)> {
        let len = (x * x + y * y + z * z).sqrt();
        if len < 1e-12 {
            None
        } else {
            Some((x / len, y / len, z / len))
        }
    }

    fn normalize_cross(a: (f64, f64, f64), b: (f64, f64, f64)) -> Option<(f64, f64, f64)> {
        let c = cross(a, b);
        normalize_vec3(c.0, c.1, c.2)
    }

    // ── STEP Writer ──────────────────────────────────────────────────

    pub struct OcctStepWriter {
        writer: cxx::UniquePtr<ffi::STEPControl_Writer>,
    }

    impl OcctStepWriter {
        pub fn new() -> Self {
            Self {
                writer: ffi::STEPControl_Writer_ctor(),
            }
        }

        pub fn add_shape(&mut self, shape: &ffi::TopoDS_Shape) -> Result<(), StepExportError> {
            let status = ffi::transfer_shape(self.writer.pin_mut(), shape);
            if status != ffi::IFSelect_ReturnStatus::IFSelect_RetDone {
                return Err(StepExportError::OcctError(format!(
                    "Writer::Transfer returned {:?}",
                    status
                )));
            }
            Ok(())
        }

        pub fn write(&mut self, path: &Path) -> Result<(), StepExportError> {
            let path_str = path
                .to_str()
                .ok_or_else(|| StepExportError::WriteError("non-UTF-8 path".into()))?
                .to_string();

            let status = ffi::write_step(self.writer.pin_mut(), path_str);
            if status != ffi::IFSelect_ReturnStatus::IFSelect_RetDone {
                return Err(StepExportError::OcctError(format!(
                    "Writer::Write returned {:?}",
                    status
                )));
            }
            Ok(())
        }
    }

    // ── Transform ────────────────────────────────────────────────────

    /// Apply a 2D translation + rotation (on the XY plane) to a 3D shape.
    pub fn transform_shape(
        shape: &ffi::TopoDS_Shape,
        dx: f64,
        dy: f64,
        rotation_rad: f64,
    ) -> cxx::UniquePtr<ffi::TopoDS_Shape> {
        let mut current = ffi::TopoDS_Shape_to_owned(shape);

        // Rotation around Z axis
        if rotation_rad.abs() > 1e-12 {
            let mut trsf = ffi::new_transform();
            let origin = ffi::new_point(0.0, 0.0, 0.0);
            let z_dir = ffi::gp_Dir_ctor(0.0, 0.0, 1.0);
            let axis = ffi::gp_Ax1_ctor(&origin, &z_dir);
            trsf.pin_mut().SetRotation(&axis, rotation_rad);
            let mut builder = ffi::BRepBuilderAPI_Transform_ctor(&current, &trsf, true);
            current = ffi::TopoDS_Shape_to_owned(builder.pin_mut().Shape());
        }

        // Translation
        if dx.abs() > 1e-12 || dy.abs() > 1e-12 {
            let mut trsf = ffi::new_transform();
            let p1 = ffi::new_point(0.0, 0.0, 0.0);
            let p2 = ffi::new_point(dx, dy, 0.0);
            trsf.pin_mut().SetTranslation(&p1, &p2);
            let mut builder = ffi::BRepBuilderAPI_Transform_ctor(&current, &trsf, true);
            current = ffi::TopoDS_Shape_to_owned(builder.pin_mut().Shape());
        }

        current
    }
}

#[cfg(feature = "occt")]
pub use inner::*;

#[cfg(feature = "occt")]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn occt_smoke_test() {
        // Verify OCCT links and we can create basic objects
        let _writer = OcctStepWriter::new();
    }
}
