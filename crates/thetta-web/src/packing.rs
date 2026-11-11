use jagua_rs::geometry::geo_enums::RotationRange;
use std::{
    collections::{HashMap, HashSet},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use thetta_geometry::{polygon_ops, types::*};

const EPS: f64 = 1e-6;
#[derive(Clone, Copy, Debug)]
struct Rect {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}
impl Rect {
    fn area(self) -> f64 {
        self.w * self.h
    }
    fn contains(self, b: Self) -> bool {
        self.x <= b.x + EPS
            && self.y <= b.y + EPS
            && self.x + self.w >= b.x + b.w - EPS
            && self.y + self.h >= b.y + b.h - EPS
    }
}
fn bounds(piece: &Piece, angle: f64) -> Rect {
    let (c, s) = (angle.cos(), angle.sin());
    let mut minx = f64::INFINITY;
    let mut miny = f64::INFINITY;
    let mut maxx = f64::NEG_INFINITY;
    let mut maxy = f64::NEG_INFINITY;
    for p in &piece.polygon.outer {
        let x = p.x * c - p.y * s;
        let y = p.x * s + p.y * c;
        minx = minx.min(x);
        miny = miny.min(y);
        maxx = maxx.max(x);
        maxy = maxy.max(y);
    }
    Rect {
        x: minx,
        y: miny,
        w: maxx - minx,
        h: maxy - miny,
    }
}
fn reserve(free: &mut Vec<Rect>, used: Rect, spacing: f64) {
    let u = Rect {
        x: used.x - spacing,
        y: used.y - spacing,
        w: used.w + 2. * spacing,
        h: used.h + 2. * spacing,
    };
    let mut next = vec![];
    for r in free.drain(..) {
        if u.x >= r.x + r.w - EPS
            || u.x + u.w <= r.x + EPS
            || u.y >= r.y + r.h - EPS
            || u.y + u.h <= r.y + EPS
        {
            next.push(r);
            continue;
        }
        if u.x > r.x + EPS {
            next.push(Rect { w: u.x - r.x, ..r });
        }
        if u.x + u.w < r.x + r.w - EPS {
            next.push(Rect {
                x: u.x + u.w,
                w: r.x + r.w - u.x - u.w,
                ..r
            });
        }
        if u.y > r.y + EPS {
            next.push(Rect { h: u.y - r.y, ..r });
        }
        if u.y + u.h < r.y + r.h - EPS {
            next.push(Rect {
                y: u.y + u.h,
                h: r.y + r.h - u.y - u.h,
                ..r
            });
        }
    }
    *free = next
        .iter()
        .enumerate()
        .filter(|(i, r)| {
            !next.iter().enumerate().any(|(j, other)| {
                i != &j && other.contains(**r) && (other.area() > r.area() + EPS || j < *i)
            })
        })
        .map(|(_, r)| *r)
        .collect();
}
fn angles(rotation: &RotationRange, fine: bool) -> Vec<f64> {
    match rotation {
        RotationRange::None => vec![0.],
        RotationRange::Discrete(values) => values
            .iter()
            .map(|v| {
                let a = *v as f64;
                let cardinal =
                    (a / std::f64::consts::FRAC_PI_2).round() * std::f64::consts::FRAC_PI_2;
                if (a - cardinal).abs() < 1e-6 {
                    cardinal
                } else {
                    a
                }
            })
            .collect(),
        RotationRange::Continuous => {
            if fine {
                (0..720)
                    .map(|i| i as f64 * std::f64::consts::PI / 360.)
                    .collect()
            } else {
                vec![0., std::f64::consts::FRAC_PI_2]
            }
        }
    }
}
fn find(piece: &Piece, rotation: &RotationRange, free: &[Vec<Rect>]) -> Option<(usize, Rect, f64)> {
    for fine in [false, true] {
        if fine && !matches!(rotation, RotationRange::Continuous) {
            break;
        }
        let mut best = None;
        let mut score = (f64::INFINITY, f64::INFINITY);
        for angle in angles(rotation, fine) {
            let b = bounds(piece, angle);
            for (si, spaces) in free.iter().enumerate() {
                for r in spaces {
                    let dw = r.w - b.w;
                    let dh = r.h - b.h;
                    let candidate = (r.area() - b.area(), dw.min(dh));
                    if b.w <= r.w + EPS && b.h <= r.h + EPS && candidate < score {
                        score = candidate;
                        best = Some((
                            si,
                            Rect {
                                x: r.x - b.x,
                                y: r.y - b.y,
                                w: b.w,
                                h: b.h,
                            },
                            angle,
                        ));
                    }
                }
            }
        }
        if best.is_some() {
            return best;
        }
    }
    None
}
/// Complete partial layouts using conservative bounding rectangles. Existing
/// contours are never moved; spacing is reserved before filling any gap.
pub fn complete(
    result: &mut NestingResult,
    pieces: &[Piece],
    rotations: &[RotationRange],
    stock: &SheetConfig,
    cancel: &Arc<AtomicBool>,
) {
    let by_id: HashMap<_, _> = pieces.iter().map(|p| (p.id, p)).collect();
    result.sheets.retain(|sheet| !sheet.placements.is_empty());
    let board = Rect {
        x: 0.,
        y: 0.,
        w: stock.width,
        h: stock.height,
    };
    let mut free = vec![vec![board]; result.sheets.len()];
    let mut placed = HashSet::new();
    for (si, sheet) in result.sheets.iter().enumerate() {
        for placement in &sheet.placements {
            if let Some(piece) = by_id.get(&placement.piece_id) {
                placed.insert(piece.id);
                let mut b = bounds(piece, placement.rotation);
                b.x += placement.x;
                b.y += placement.y;
                reserve(&mut free[si], b, stock.separation);
            }
        }
    }
    let mut remaining: Vec<_> = pieces
        .iter()
        .enumerate()
        .filter(|(_, p)| !placed.contains(&p.id))
        .collect();
    remaining.sort_by(|(_, a), (_, b)| {
        // Quantization absorbs STEP roundoff while retaining a transitive sort.
        // Equal-area parts always use their stable identity as a tie breaker.
        let x = (polygon_ops::area(&a.polygon) * 1e3).round();
        let y = (polygon_ops::area(&b.polygon) * 1e3).round();
        y.total_cmp(&x).then_with(|| a.id.0.cmp(&b.id.0))
    });
    for (index, piece) in remaining {
        if cancel.load(Ordering::Relaxed) {
            break;
        }
        let rotation = &rotations[index];
        let candidate = find(piece, rotation, &free).or_else(|| {
            find(piece, rotation, &[vec![board]]).map(|(_, b, angle)| {
                free.push(vec![board]);
                result.sheets.push(Sheet::default());
                (free.len() - 1, b, angle)
            })
        });
        if let Some((si, position, angle)) = candidate {
            result.sheets[si].placements.push(Placement {
                piece_id: piece.id,
                x: position.x,
                y: position.y,
                rotation: angle,
            });
            let mut used = bounds(piece, angle);
            used.x += position.x;
            used.y += position.y;
            reserve(&mut free[si], used, stock.separation);
            placed.insert(piece.id);
        }
    }
    result.unplaced = pieces
        .iter()
        .filter(|p| !placed.contains(&p.id))
        .map(|p| p.id)
        .collect();
    for sheet in &mut result.sheets {
        sheet.utilization = sheet
            .placements
            .iter()
            .filter_map(|p| by_id.get(&p.piece_id))
            .map(|p| polygon_ops::area(&p.polygon))
            .sum::<f64>()
            / (stock.width * stock.height);
        sheet.strip_length = sheet
            .placements
            .iter()
            .filter_map(|p| {
                by_id
                    .get(&p.piece_id)
                    .map(|piece| bounds(piece, p.rotation).x + p.x + bounds(piece, p.rotation).w)
            })
            .fold(0., f64::max);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn rectangle(id: u64, w: f64, h: f64) -> Piece {
        Piece {
            id: PieceId(id),
            quantity: 1,
            source_file: Default::default(),
            polygon: Polygon {
                outer: vec![
                    Point2D { x: 0., y: 0. },
                    Point2D { x: w, y: 0. },
                    Point2D { x: w, y: h },
                    Point2D { x: 0., y: h },
                ],
                holes: vec![],
            },
        }
    }
    #[test]
    fn completion_preserves_spacing_and_fits_full_stock_and_diagonal_strips() {
        let stock = SheetConfig {
            width: 2440.,
            height: 1220.,
            separation: 12.,
        };
        let pieces = vec![
            rectangle(1, 1220., 2440.),
            rectangle(2, 2580., 101.6),
            rectangle(3, 200., 120.),
            rectangle(4, 200., 120.),
            rectangle(5, 3000., 3000.),
        ];
        let mut result = NestingResult::default();
        complete(
            &mut result,
            &pieces,
            &vec![RotationRange::Continuous; pieces.len()],
            &stock,
            &Arc::new(AtomicBool::new(false)),
        );
        assert_eq!(result.unplaced, vec![PieceId(5)]);
        assert_eq!(
            result
                .sheets
                .iter()
                .map(|s| s.placements.len())
                .sum::<usize>(),
            4
        );
        for sheet in &result.sheets {
            let boxes: Vec<_> = sheet
                .placements
                .iter()
                .map(|p| {
                    let piece = pieces.iter().find(|piece| piece.id == p.piece_id).unwrap();
                    let mut b = bounds(piece, p.rotation);
                    b.x += p.x;
                    b.y += p.y;
                    b
                })
                .collect();
            for b in &boxes {
                assert!(
                    b.x >= -EPS
                        && b.y >= -EPS
                        && b.x + b.w <= stock.width + EPS
                        && b.y + b.h <= stock.height + EPS
                );
            }
            for i in 0..boxes.len() {
                for j in i + 1..boxes.len() {
                    let (a, b) = (boxes[i], boxes[j]);
                    assert!(
                        a.x + a.w + stock.separation <= b.x + EPS
                            || b.x + b.w + stock.separation <= a.x + EPS
                            || a.y + a.h + stock.separation <= b.y + EPS
                            || b.y + b.h + stock.separation <= a.y + EPS
                    );
                }
            }
        }
        let before = result
            .sheets
            .iter()
            .map(|s| s.placements.clone())
            .collect::<Vec<_>>();
        complete(
            &mut result,
            &pieces,
            &vec![RotationRange::Continuous; pieces.len()],
            &stock,
            &Arc::new(AtomicBool::new(false)),
        );
        assert_eq!(
            result
                .sheets
                .iter()
                .map(|s| s.placements.len())
                .collect::<Vec<_>>(),
            before.iter().map(|p| p.len()).collect::<Vec<_>>()
        );
    }
    #[test]
    fn completion_respects_fixed_rotation_and_cancellation() {
        let pieces = vec![rectangle(1, 1220., 2440.)];
        let mut result = NestingResult::default();
        let stock = SheetConfig::default();
        complete(
            &mut result,
            &pieces,
            &[RotationRange::None],
            &stock,
            &Arc::new(AtomicBool::new(false)),
        );
        assert_eq!(result.unplaced, vec![PieceId(1)]);
        complete(
            &mut result,
            &pieces,
            &[RotationRange::Continuous],
            &stock,
            &Arc::new(AtomicBool::new(true)),
        );
        assert!(result.sheets.is_empty());
    }
}
