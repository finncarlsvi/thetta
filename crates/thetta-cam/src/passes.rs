//! Bounded depth scheduling and contour ramps for automatic CAM.
use thetta_geometry::types::*;
pub fn depths(total: f64, step: f64) -> Result<Vec<f64>, String> {
    if !total.is_finite() || !step.is_finite() || total <= 0. || step <= 0. || total / step > 10000.
    {
        return Err("Invalid depth schedule or more than 10000 passes".into());
    }
    let count = (total / step).ceil() as usize;
    Ok((1..=count).map(|n| (n as f64 * step).min(total)).collect())
}
fn ramp(
    start: Point2D,
    points: &[Point3D],
    previous: f64,
    depth: f64,
    angle: f64,
    tool: &Tool,
    safe: f64,
) -> Result<Vec<ToolpathMove>, String> {
    if !(0.1..=5.).contains(&angle) || points.len() < 3 {
        return Err("Invalid contour ramp".into());
    }
    let mut last = start;
    let length: f64 = points
        .iter()
        .map(|p| {
            let d = (p.x - last.x).hypot(p.y - last.y);
            last = Point2D { x: p.x, y: p.y };
            d
        })
        .sum();
    let slope = angle.to_radians().tan();
    let laps = ((depth - previous) / (length * slope)).ceil() as usize;
    if length < 0.001 || laps > 10000 {
        return Err("Contour cannot support the requested entry ramp".into());
    }
    let mut moves = vec![
        ToolpathMove::Rapid {
            to: Point3D {
                x: start.x,
                y: start.y,
                z: safe,
            },
        },
        ToolpathMove::Plunge {
            to_z: -previous,
            feed: tool.plunge_rate,
        },
    ];
    let mut distance = 0.;
    last = start;
    for lap in 0..=laps {
        for p in points {
            distance += (p.x - last.x).hypot(p.y - last.y);
            let ramp_z = if lap == laps {
                -depth
            } else {
                (-previous - distance * slope).max(-depth)
            };
            moves.push(ToolpathMove::LinearCut {
                to: Point3D {
                    x: p.x,
                    y: p.y,
                    z: p.z.max(ramp_z),
                },
                feed: if lap == laps {
                    tool.feed_rate
                } else {
                    tool.plunge_rate
                },
            });
            last = Point2D { x: p.x, y: p.y };
        }
    }
    moves.push(ToolpathMove::Retract { to_z: safe });
    Ok(moves)
}
pub fn profile(
    polygon: &Polygon,
    tabs: &[Tab],
    tool: &Tool,
    gcode: &GcodeConfig,
    config: &ProfileCutConfig,
    step: f64,
    angle: f64,
) -> Result<Toolpath, String> {
    profile_entry(polygon, tabs, tool, gcode, config, step, angle, false)
}
pub fn profile_entry(
    polygon: &Polygon,
    tabs: &[Tab],
    tool: &Tool,
    gcode: &GcodeConfig,
    config: &ProfileCutConfig,
    step: f64,
    angle: f64,
    compression: bool,
) -> Result<Toolpath, String> {
    let mut moves = vec![];
    let mut previous = 0.;
    for depth in depths(gcode.cut_depth, step)? {
        let cfg = GcodeConfig {
            cut_depth: depth,
            ..gcode.clone()
        };
        let adjusted: Vec<_> = tabs
            .iter()
            .filter_map(|t| {
                let mut t = t.clone();
                t.height = (depth - gcode.cut_depth + t.height).max(0.);
                (t.height > 0.).then_some(t)
            })
            .collect();
        let path = crate::toolpath::plan_profile_toolpath(polygon, &adjusted, tool, &cfg, config);
        let start = path
            .moves
            .iter()
            .find_map(|m| {
                if let ToolpathMove::ArcCut { to, .. } = m {
                    Some(Point2D { x: to.x, y: to.y })
                } else {
                    None
                }
            })
            .ok_or("Profile has no valid entry")?;
        let points: Vec<_> = path
            .moves
            .iter()
            .filter_map(|m| match m {
                ToolpathMove::LinearCut { to, .. } | ToolpathMove::TabLift { to, .. } => Some(*to),
                _ => None,
            })
            .collect();
        if compression {
            let center = path
                .moves
                .iter()
                .find_map(|m| {
                    if let ToolpathMove::ArcCut { center, .. } = m {
                        Some(*center)
                    } else {
                        None
                    }
                })
                .ok_or("No compression entry direction")?;
            let dx = center.x - start.x;
            let dy = center.y - start.y;
            let norm = dx.hypot(dy);
            if norm < 1e-6 {
                return Err("Invalid compression entry".into());
            }
            let radius = tool.diameter * 0.55;
            let center = Point2D {
                x: start.x + radius * dx / norm,
                y: start.y + radius * dy / norm,
            };
            let theta = (start.y - center.y).atan2(start.x - center.x);
            let turns = ((depth - previous)
                / (std::f64::consts::TAU * radius * angle.to_radians().tan()))
            .ceil()
            .max(1.) as usize;
            if turns > 10000 {
                return Err("Too many entry turns".into());
            }
            moves.push(ToolpathMove::Rapid {
                to: Point3D {
                    x: start.x,
                    y: start.y,
                    z: gcode.safe_z,
                },
            });
            moves.push(ToolpathMove::Plunge {
                to_z: -previous,
                feed: tool.plunge_rate,
            });
            for i in 1..=turns * 64 {
                let f = i as f64 / (turns * 64) as f64;
                let a = theta + f * turns as f64 * std::f64::consts::TAU;
                moves.push(ToolpathMove::LinearCut {
                    to: Point3D {
                        x: center.x + radius * a.cos(),
                        y: center.y + radius * a.sin(),
                        z: -previous - f * (depth - previous),
                    },
                    feed: tool.plunge_rate,
                });
            }
            for point in points {
                moves.push(ToolpathMove::LinearCut {
                    to: point,
                    feed: tool.feed_rate,
                });
            }
            moves.push(ToolpathMove::Retract { to_z: gcode.safe_z });
        } else {
            moves.extend(ramp(
                start,
                &points,
                previous,
                depth,
                angle,
                tool,
                gcode.safe_z,
            )?);
        }
        previous = depth;
    }
    Ok(Toolpath {
        moves,
        config: gcode.clone(),
    })
}
pub fn pocket(
    region: &PocketRegion,
    tool: &Tool,
    gcode: &GcodeConfig,
    config: &PocketConfig,
    step: f64,
    angle: f64,
) -> Result<Toolpath, String> {
    let mut moves = vec![];
    let mut previous = 0.;
    for depth in depths(gcode.cut_depth, step)? {
        let cfg = GcodeConfig {
            cut_depth: depth,
            ..gcode.clone()
        };
        let path = crate::pocket::plan_pocket_toolpath(region, tool, config, &cfg);
        if path.moves.is_empty() {
            return Err("Pocket cutter does not fit the requested feature".into());
        }
        let mut start = None;
        let mut points = vec![];
        for m in path.moves {
            match m {
                ToolpathMove::Rapid { to } => {
                    if !points.is_empty() {
                        moves.extend(ramp(
                            start.ok_or("Missing pocket entry")?,
                            &points,
                            previous,
                            depth,
                            angle,
                            tool,
                            gcode.safe_z,
                        )?);
                        points.clear();
                    }
                    start = Some(Point2D { x: to.x, y: to.y });
                }
                ToolpathMove::LinearCut { to, .. } => points.push(to),
                _ => {}
            }
        }
        if !points.is_empty() {
            moves.extend(ramp(
                start.ok_or("Missing pocket entry")?,
                &points,
                previous,
                depth,
                angle,
                tool,
                gcode.safe_z,
            )?);
        }
        previous = depth;
    }
    Ok(Toolpath {
        moves,
        config: gcode.clone(),
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    fn tool() -> Tool {
        Tool {
            number: 1,
            name: "test".into(),
            diameter: 6.,
            flute_count: 2,
            feed_rate: 1200.,
            plunge_rate: 400.,
            spindle_speed: 18000,
        }
    }
    #[test]
    fn schedules_and_limits() {
        assert_eq!(depths(17.75, 6.).unwrap(), vec![6., 12., 17.75]);
        assert!(depths(1., 0.).is_err());
        assert!(depths(1., f64::NAN).is_err());
        assert!(depths(20., 0.00001).is_err());
    }
    #[test]
    fn tabs_stay_at_physical_height_and_entry_is_bounded() {
        let p = thetta_geometry::test_shapes::rectangle(100., 60.);
        let cfg = GcodeConfig {
            cut_depth: 12.,
            ..Default::default()
        };
        let tabs = vec![Tab {
            position: Point2D { x: 50., y: 0. },
            width: 10.,
            height: 3.,
            edge_index: 0,
            t: 0.5,
        }];
        let path = profile(
            &p,
            &tabs,
            &tool(),
            &cfg,
            &ProfileCutConfig::default(),
            4.,
            2.,
        )
        .unwrap();
        let zs: Vec<_> = path
            .moves
            .iter()
            .filter_map(|m| {
                if let ToolpathMove::LinearCut { to, .. } = m {
                    Some(to.z)
                } else {
                    None
                }
            })
            .collect();
        assert!(zs.iter().any(|z| (*z + 12.).abs() < 1e-6));
        assert!(zs.iter().any(|z| (*z + 9.).abs() < 1e-6));
        assert!(zs.iter().all(|z| *z >= -12. && *z <= 0.));
        let plunges: Vec<_> = path
            .moves
            .iter()
            .filter_map(|m| {
                if let ToolpathMove::Plunge { to_z, .. } = m {
                    Some(*to_z)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(plunges, vec![0., -4., -8.]);
    }
}

/// Check the entire swept XY cutter path against the original solid profiles.
/// This includes the waste helix, not just its endpoint.
pub fn validate_profile_clearance(
    path: &Toolpath,
    solids: &[Polygon],
    diameter: f64,
) -> Result<(), String> {
    use geo::{BoundingRect, Distance, Euclidean};
    let polygons: Vec<_> = solids
        .iter()
        .map(|p| {
            let mut pts: Vec<_> = p.outer.iter().map(|p| (p.x, p.y)).collect();
            if let Some(first) = pts.first().copied() {
                pts.push(first);
            }
            geo_types::Polygon::new(geo_types::LineString::from(pts), vec![])
        })
        .collect();
    let radius = diameter / 2.;
    let mut previous = None;
    for mv in &path.moves {
        match mv {
            ToolpathMove::Rapid { to } => previous = Some(*to),
            ToolpathMove::LinearCut { to, .. } | ToolpathMove::TabLift { to, .. } => {
                if let Some(from) = previous {
                    let line = geo_types::Line::new(
                        geo_types::Coord {
                            x: from.x,
                            y: from.y,
                        },
                        geo_types::Coord { x: to.x, y: to.y },
                    );
                    for polygon in &polygons {
                        if let Some(b) = polygon.bounding_rect() {
                            if from.x.min(to.x) > b.max().x + radius
                                || from.x.max(to.x) < b.min().x - radius
                                || from.y.min(to.y) > b.max().y + radius
                                || from.y.max(to.y) < b.min().y - radius
                            {
                                continue;
                            }
                        }
                        if Euclidean::distance(&line, polygon) < radius - 0.003 {
                            return Err("Profile cutter or entry path intersects a part. Increase spacing, change tool, or change layout.".into());
                        }
                    }
                }
                previous = Some(*to);
            }
            _ => {}
        }
    }
    Ok(())
}

#[cfg(test)]
mod entry_tests {
    use super::*;
    #[test]
    fn pocket_passes_use_ramps_and_reach_the_final_depth() {
        let tool = Tool {
            number: 1,
            name: "flat".into(),
            diameter: 6.,
            flute_count: 2,
            feed_rate: 1200.,
            plunge_rate: 400.,
            spindle_speed: 18000,
        };
        let region = PocketRegion {
            contour: thetta_geometry::test_shapes::rectangle(40., 30.).outer,
        };
        let cfg = GcodeConfig {
            cut_depth: 12.,
            ..Default::default()
        };
        let path = pocket(&region, &tool, &cfg, &PocketConfig::default(), 4., 2.).unwrap();
        for depth in [4., 8., 12.] {
            assert!(
                path.moves
                    .iter()
                    .any(|m| matches!(m,ToolpathMove::LinearCut{to,..} if (to.z+depth).abs()<1e-7))
            );
        }
        let mut previous = None;
        for mv in &path.moves {
            match mv {
                ToolpathMove::Rapid { to } => previous = Some(*to),
                ToolpathMove::Plunge { to_z, .. } => {
                    assert!([0., -4., -8.].contains(to_z));
                    if let Some(p) = previous.as_mut() {
                        p.z = *to_z;
                    }
                }
                ToolpathMove::LinearCut { to, .. } => {
                    if let Some(p) = previous {
                        assert!(
                            (p.z - to.z).max(0.)
                                <= (to.x - p.x).hypot(to.y - p.y) * 2f64.to_radians().tan() + 1e-7
                        );
                    }
                    previous = Some(*to);
                }
                _ => {}
            }
        }
    }
    #[test]
    fn swept_profile_rejects_a_neighbor_collision() {
        let cfg = GcodeConfig {
            cut_depth: 4.,
            ..Default::default()
        };
        let tool = Tool {
            number: 1,
            name: "flat".into(),
            diameter: 6.,
            flute_count: 2,
            feed_rate: 1200.,
            plunge_rate: 400.,
            spindle_speed: 18000,
        };
        let a = thetta_geometry::test_shapes::rectangle(40., 30.);
        let b = thetta_geometry::polygon_ops::translate(&a, 42., 0.);
        let path = profile(&a, &[], &tool, &cfg, &ProfileCutConfig::default(), 2., 2.).unwrap();
        assert!(validate_profile_clearance(&path, &[a.clone()], 6.).is_ok());
        assert!(validate_profile_clearance(&path, &[a, b], 6.).is_err());
    }
}
