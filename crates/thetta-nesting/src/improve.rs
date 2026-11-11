//! Anytime, contour-aware improvement of a complete fixed-sheet layout.
//! Candidate geometry is simplified only for search; acceptance uses original contours.
use jagua_rs::geometry::geo_enums::RotationRange;
use rand::{SeedableRng, seq::SliceRandom};
use std::{
    collections::{HashMap, HashSet},
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};
use thetta_geometry::{polygon_ops, types::*};

type P = [f64; 2];
#[derive(Clone, Copy)]
struct Rect {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}
const EPS: f64 = 0.002; // CAD/export roundoff, millimetres.
// Use the full stock height and preserve a continuous offcut on the right.
// Width is measured from x=0, so moving an unchanged cluster left also wins.
type Cost = (f64, f64); // rightmost occupied x, occupied bounding-box area
fn placement_cost(b: [f64; 4]) -> Cost {
    (b[2], area(b))
}
fn cost_cmp(a: Cost, b: Cost) -> std::cmp::Ordering {
    a.0.total_cmp(&b.0).then_with(|| a.1.total_cmp(&b.1))
}
fn cost_better(a: Cost, b: Cost) -> bool {
    a.0 < b.0 - 0.01 || ((a.0 - b.0).abs() <= 0.01 && a.1 < b.1 - 0.01)
}
fn layout_better(a: (usize, Cost), b: (usize, Cost)) -> bool {
    a.0 < b.0 || (a.0 == b.0 && cost_better(a.1, b.1))
}

#[derive(Clone)]
struct Shape {
    points: Vec<P>,
    contacts: Vec<P>,
    bounds: [f64; 4],
}
fn cross(a: P, b: P, c: P) -> f64 {
    (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
}
fn bounds(p: &[P]) -> [f64; 4] {
    p.iter().fold(
        [
            f64::INFINITY,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::NEG_INFINITY,
        ],
        |mut b, p| {
            b[0] = b[0].min(p[0]);
            b[1] = b[1].min(p[1]);
            b[2] = b[2].max(p[0]);
            b[3] = b[3].max(p[1]);
            b
        },
    )
}
fn union(a: [f64; 4], b: [f64; 4]) -> [f64; 4] {
    [
        a[0].min(b[0]),
        a[1].min(b[1]),
        a[2].max(b[2]),
        a[3].max(b[3]),
    ]
}
fn area(b: [f64; 4]) -> f64 {
    (b[2] - b[0]) * (b[3] - b[1])
}
impl Shape {
    fn new(piece: &Piece, angle: f64) -> Self {
        let (s, c) = angle.sin_cos();
        let points: Vec<_> = piece
            .polygon
            .outer
            .iter()
            .map(|p| [p.x * c - p.y * s, p.x * s + p.y * c])
            .collect();
        let mut contacts = points.clone();
        // Remove least salient vertices, retaining corners and concave transitions.
        while contacts.len() > 24 {
            let n = contacts.len();
            let i = (0..n)
                .min_by(|&i, &j| {
                    cross(
                        contacts[(i + n - 1) % n],
                        contacts[i],
                        contacts[(i + 1) % n],
                    )
                    .abs()
                    .total_cmp(
                        &cross(
                            contacts[(j + n - 1) % n],
                            contacts[j],
                            contacts[(j + 1) % n],
                        )
                        .abs(),
                    )
                })
                .unwrap();
            contacts.remove(i);
        }
        Self {
            bounds: bounds(&points),
            points,
            contacts,
        }
    }
    fn moved(&self, x: f64, y: f64) -> Self {
        Self {
            points: self.points.iter().map(|p| [p[0] + x, p[1] + y]).collect(),
            contacts: self.contacts.iter().map(|p| [p[0] + x, p[1] + y]).collect(),
            bounds: [
                self.bounds[0] + x,
                self.bounds[1] + y,
                self.bounds[2] + x,
                self.bounds[3] + y,
            ],
        }
    }
}
fn point_distance(p: P, a: P, b: P) -> f64 {
    let v = [b[0] - a[0], b[1] - a[1]];
    let n = v[0] * v[0] + v[1] * v[1];
    let t = if n > 0. {
        ((p[0] - a[0]) * v[0] + (p[1] - a[1]) * v[1]) / n
    } else {
        0.
    }
    .clamp(0., 1.);
    (p[0] - a[0] - t * v[0]).hypot(p[1] - a[1] - t * v[1])
}
fn inside(p: P, poly: &[P]) -> bool {
    let mut yes = false;
    for i in 0..poly.len() {
        let a = poly[i];
        let b = poly[(i + 1) % poly.len()];
        if (a[1] > p[1]) != (b[1] > p[1])
            && p[0] < (b[0] - a[0]) * (p[1] - a[1]) / (b[1] - a[1]) + a[0]
        {
            yes = !yes
        }
    }
    yes
}
fn compatible(a: &Shape, b: &Shape, gap: f64) -> bool {
    let x = (a.bounds[0] - b.bounds[2])
        .max(b.bounds[0] - a.bounds[2])
        .max(0.);
    let y = (a.bounds[1] - b.bounds[3])
        .max(b.bounds[1] - a.bounds[3])
        .max(0.);
    if x.hypot(y) > gap + EPS {
        return true;
    }
    let min = (gap - EPS).max(0.);
    for i in 0..a.points.len() {
        let p = a.points[i];
        let q = a.points[(i + 1) % a.points.len()];
        for j in 0..b.points.len() {
            let r = b.points[j];
            let s = b.points[(j + 1) % b.points.len()];
            if cross(p, q, r) * cross(p, q, s) < 0. && cross(r, s, p) * cross(r, s, q) < 0. {
                return false;
            }
            if point_distance(p, r, s) < min
                || point_distance(q, r, s) < min
                || point_distance(r, p, q) < min
                || point_distance(s, p, q) < min
            {
                return false;
            }
        }
    }
    if gap > EPS {
        return !inside(a.points[0], &b.points) && !inside(b.points[0], &a.points);
    }
    // Zero spacing permits shared boundaries, but never shared interior.
    for shape in [a, b] {
        for i in 0..shape.points.len() {
            let p = shape.points[i];
            let q = shape.points[(i + 1) % shape.points.len()];
            let dx = q[0] - p[0];
            let dy = q[1] - p[1];
            let len = dx.hypot(dy);
            if len == 0. {
                continue;
            }
            for sign in [-1., 1.] {
                let sample = [
                    (p[0] + q[0]) * 0.5 - sign * dy / len * 1e-5,
                    (p[1] + q[1]) * 0.5 + sign * dx / len * 1e-5,
                ];
                if inside(sample, &a.points) && inside(sample, &b.points) {
                    return false;
                }
            }
        }
    }
    true
}
fn allowed(angle: f64, range: &RotationRange) -> bool {
    let delta = |b: f64| {
        ((angle - b + std::f64::consts::PI).rem_euclid(std::f64::consts::TAU)
            - std::f64::consts::PI)
            .abs()
            < 1e-5
    };
    match range {
        RotationRange::Continuous => true,
        RotationRange::None => delta(0.),
        RotationRange::Discrete(v) => v.iter().any(|&a| delta(a as f64)),
    }
}
fn in_board(s: &Shape, stock: &SheetConfig) -> bool {
    s.bounds[0] >= -EPS
        && s.bounds[1] >= -EPS
        && s.bounds[2] <= stock.width + EPS
        && s.bounds[3] <= stock.height + EPS
}

/// Validate all identities, rotations, original outer contours and stock boundaries.
/// Holes deliberately remain unavailable to nesting.
pub fn validate(
    result: &NestingResult,
    pieces: &[Piece],
    rotations: &[RotationRange],
    stock: &SheetConfig,
) -> Result<(), String> {
    if pieces.is_empty() || pieces.len() != rotations.len() || !result.unplaced.is_empty() {
        return Err("A complete layout is required".into());
    }
    let by_id: HashMap<_, _> = pieces.iter().enumerate().map(|(i, p)| (p.id, i)).collect();
    if by_id.len() != pieces.len() {
        return Err("Duplicate part identity".into());
    }
    let mut seen = HashSet::new();
    for sheet in &result.sheets {
        let mut shapes = vec![];
        for p in &sheet.placements {
            let &i = by_id.get(&p.piece_id).ok_or("Unknown part in layout")?;
            if !seen.insert(p.piece_id)
                || ![p.x, p.y, p.rotation].iter().all(|v| v.is_finite())
                || !allowed(p.rotation, &rotations[i])
            {
                return Err("Invalid or duplicate placement".into());
            }
            let s = Shape::new(&pieces[i], p.rotation).moved(p.x, p.y);
            if !in_board(&s, stock)
                || shapes
                    .iter()
                    .any(|other| !compatible(&s, other, stock.separation))
            {
                return Err("Layout violates contour spacing or stock boundaries".into());
            }
            shapes.push(s);
        }
    }
    if seen.len() != pieces.len() {
        return Err("Layout is missing selected parts".into());
    }
    Ok(())
}
struct Search<'a> {
    pieces: &'a [Piece],
    rotations: &'a [RotationRange],
    stock: &'a SheetConfig,
    cache: HashMap<(usize, i64), Shape>,
    pair_cache: HashMap<(usize, usize), Option<Vec<Placement>>>,
    radii: HashMap<usize, f64>,
    end: Instant,
    cancel: &'a AtomicBool,
}
impl Search<'_> {
    fn stopped(&self) -> bool {
        Instant::now() >= self.end || self.cancel.load(Ordering::Relaxed)
    }
    fn shape(&mut self, i: usize, a: f64) -> Shape {
        let key = (i, (a * 1e8).round() as i64);
        self.cache
            .entry(key)
            .or_insert_with(|| Shape::new(&self.pieces[i], a))
            .clone()
    }
    fn index(&self, p: &Placement) -> usize {
        self.pieces.iter().position(|x| x.id == p.piece_id).unwrap()
    }
    fn score(&mut self, r: &NestingResult) -> (usize, Cost) {
        let mut total = (0., 0.);
        let mut count = 0;
        for sheet in &r.sheets {
            if sheet.placements.is_empty() {
                continue;
            }
            count += 1;
            let mut b = None;
            for p in &sheet.placements {
                let i = self.index(p);
                let s = self.shape(i, p.rotation).moved(p.x, p.y);
                b = Some(b.map_or(s.bounds, |b| union(b, s.bounds)))
            }
            let used = placement_cost(b.unwrap());
            total.0 += used.0;
            total.1 += used.1;
        }
        (count, total)
    }
    // An inscribed circle provides an orientation-independent impossibility
    // bound. Only exact rectangular obstacles spanning the circle's full centre
    // domain qualify; concave openings and rounded corners are never boxed off.
    fn blocked_by_rectangle(&mut self, i: usize, placed: &[Placement]) -> bool {
        let r = if let Some(&r) = self.radii.get(&i) {
            r
        } else {
            let shape = self.shape(i, 0.);
            let b = shape.bounds;
            let mut radius = 0_f64;
            for x in 0..8 {
                for y in 0..8 {
                    let p = [
                        b[0] + (x as f64 + 0.5) * (b[2] - b[0]) / 8.,
                        b[1] + (y as f64 + 0.5) * (b[3] - b[1]) / 8.,
                    ];
                    if inside(p, &shape.points) {
                        let d = (0..shape.points.len())
                            .map(|k| {
                                point_distance(
                                    p,
                                    shape.points[k],
                                    shape.points[(k + 1) % shape.points.len()],
                                )
                            })
                            .fold(f64::INFINITY, f64::min);
                        radius = radius.max((d - EPS).max(0.));
                    }
                }
            }
            self.radii.insert(i, radius);
            radius
        };
        if r <= EPS {
            return false;
        }
        let expand = r + self.stock.separation - 2. * EPS;
        for p in placed {
            let j = self.index(p);
            let shape = self.shape(j, p.rotation).moved(p.x, p.y);
            let b = shape.bounds;
            if shape.points.len() != 4
                || !shape.points.iter().all(|p| {
                    ((p[0] - b[0]).abs() < 1e-7 || (p[0] - b[2]).abs() < 1e-7)
                        && ((p[1] - b[1]).abs() < 1e-7 || (p[1] - b[3]).abs() < 1e-7)
                })
            {
                continue;
            }
            let covers_y = b[1] <= r && b[3] >= self.stock.height - r;
            let covers_x = b[0] <= r && b[2] >= self.stock.width - r;
            if (covers_y && b[0] - expand < r - EPS && b[2] + expand > self.stock.width - r + EPS)
                || (covers_x
                    && b[1] - expand < r - EPS
                    && b[3] + expand > self.stock.height - r + EPS)
            {
                return true;
            }
        }
        false
    }
    fn find(&mut self, i: usize, placed: &[Placement]) -> Option<Placement> {
        self.find_with_objective(i, placed, false)
    }
    fn find_with_objective(
        &mut self,
        i: usize,
        placed: &[Placement],
        pair_seed: bool,
    ) -> Option<Placement> {
        // Build tightly interlocked units before orienting those units to the left.
        let evaluate = |b: [f64; 4]| {
            if pair_seed {
                (area(b), 0.)
            } else {
                placement_cost(b)
            }
        };
        // Impossible area combinations must not consume contact-search time.
        let used: f64 = placed
            .iter()
            .map(|p| polygon_ops::area(&self.pieces[self.index(p)].polygon))
            .sum();
        if used + polygon_ops::area(&self.pieces[i].polygon)
            > self.stock.width * self.stock.height + EPS
        {
            return None;
        }
        if self.blocked_by_rectangle(i, placed) {
            return None;
        }
        let slice = if pair_seed { 2000 } else { 700 };
        let until = (Instant::now() + Duration::from_millis(slice)).min(self.end);
        let others: Vec<_> = placed
            .iter()
            .map(|p| {
                let j = self.index(p);
                self.shape(j, p.rotation).moved(p.x, p.y)
            })
            .collect();
        let occupied = others.iter().map(|s| s.bounds).reduce(union);
        let mut angles = match &self.rotations[i] {
            RotationRange::None => vec![0.],
            RotationRange::Discrete(a) => a.iter().map(|&a| a as f64).collect(),
            RotationRange::Continuous => {
                let mut a = vec![0., 90., 180., 270.];
                a.extend((0..24).map(|v| v as f64 * 15.).filter(|a| a % 90. != 0.));
                a.into_iter().map(f64::to_radians).collect()
            }
        };
        // Try cardinal orientations first, favouring the narrowest used width.
        if others.is_empty() && matches!(self.rotations[i], RotationRange::Continuous) {
            angles.truncate(4)
        }
        let mut best: Option<(Cost, Placement)> = None;
        for a in angles {
            if self.stopped() || Instant::now() >= until {
                break;
            }
            let shape = self.shape(i, a);
            let b = shape.bounds;
            if b[2] - b[0] > self.stock.width + EPS || b[3] - b[1] > self.stock.height + EPS {
                continue;
            }
            let mut candidates = vec![
                (-b[0], -b[1]),
                (self.stock.width - b[2], -b[1]),
                (-b[0], self.stock.height - b[3]),
            ];
            let gap = self.stock.separation + 0.01;
            for other in &others {
                let o = other.bounds;
                candidates.extend([
                    (o[2] + gap - b[0], o[1] - b[1]),
                    (o[0] - b[0], o[3] + gap - b[1]),
                    (o[0] - gap - b[2], o[1] - b[1]),
                    (o[0] - b[0], o[1] - gap - b[3]),
                ]);
                for p in &shape.contacts {
                    for q in &other.contacts {
                        for k in 0..8 {
                            let t = k as f64 * std::f64::consts::FRAC_PI_4;
                            candidates
                                .push((q[0] - p[0] + gap * t.cos(), q[1] - p[1] + gap * t.sin()));
                        }
                    }
                }
                // Contact with straight edge interiors, including long edges with few vertices.
                for p in &shape.contacts {
                    for e in 0..other.contacts.len() {
                        let q = other.contacts[e];
                        let r = other.contacts[(e + 1) % other.contacts.len()];
                        let dx = r[0] - q[0];
                        let dy = r[1] - q[1];
                        let len = dx.hypot(dy);
                        if len > 0. {
                            for t in [0.25, 0.5, 0.75] {
                                for side in [-1., 1.] {
                                    candidates.push((
                                        q[0] + t * dx - p[0] - side * gap * dy / len,
                                        q[1] + t * dy - p[1] + side * gap * dx / len,
                                    ));
                                }
                            }
                        }
                    }
                }
            }
            let mut ranked: Vec<_> = candidates
                .into_iter()
                .filter_map(|(x, y)| {
                    let bb = [b[0] + x, b[1] + y, b[2] + x, b[3] + y];
                    if bb[0] < -EPS
                        || bb[1] < -EPS
                        || bb[2] > self.stock.width + EPS
                        || bb[3] > self.stock.height + EPS
                    {
                        return None;
                    }
                    let cost = evaluate(occupied.map_or(bb, |o| union(o, bb)));
                    Some((cost, x, y))
                })
                .collect();
            ranked.sort_unstable_by(|a, b| {
                cost_cmp(a.0, b.0)
                    .then_with(|| a.1.total_cmp(&b.1))
                    .then_with(|| a.2.total_cmp(&b.2))
            });
            let mut seen = HashSet::new();
            for (n, (cost, x, y)) in ranked.into_iter().enumerate() {
                if n % 64 == 0 && (self.stopped() || Instant::now() >= until) {
                    break;
                }
                if let Some((previous, _)) = &best {
                    if cost.0 > previous.0 + 0.01 {
                        break;
                    }
                    if !cost_better(cost, *previous) {
                        continue;
                    }
                }
                if !seen.insert(((x * 100.).round() as i64, (y * 100.).round() as i64)) {
                    continue;
                }
                let s = shape.moved(x, y);
                if others
                    .iter()
                    .all(|o| compatible(&s, o, self.stock.separation))
                {
                    best = Some((
                        cost,
                        Placement {
                            piece_id: self.pieces[i].id,
                            x,
                            y,
                            rotation: a,
                        },
                    ));
                    break;
                }
            }
        }
        // Coordinate refinement of promising contact placements, with 1-degree angle moves.
        if let Some((mut cost, mut p)) = best {
            for step in [10., 2., 0.5, 0.1] {
                for _ in 0..12 {
                    if self.stopped() || Instant::now() >= until {
                        break;
                    }
                    let mut improved = false;
                    for (dx, dy, da) in [
                        (step, 0., 0.),
                        (-step, 0., 0.),
                        (0., step, 0.),
                        (0., -step, 0.),
                        (0., 0., 1_f64.to_radians()),
                        (0., 0., -1_f64.to_radians()),
                    ] {
                        let a = p.rotation + da;
                        if !allowed(a, &self.rotations[i]) {
                            continue;
                        }
                        let s = self.shape(i, a).moved(p.x + dx, p.y + dy);
                        let score = evaluate(occupied.map_or(s.bounds, |o| union(o, s.bounds)));
                        if cost_better(score, cost)
                            && in_board(&s, self.stock)
                            && others
                                .iter()
                                .all(|o| compatible(&s, o, self.stock.separation))
                        {
                            p.x += dx;
                            p.y += dy;
                            p.rotation = a;
                            cost = score;
                            improved = true
                        }
                    }
                    if !improved {
                        break;
                    }
                }
            }
            Some(p)
        } else {
            None
        }
    }
    // Repair small legacy-clearance errors locally before a full placement search.
    // Keeping the sheet assignment intact preserves useful clusters for merging.
    fn nudge(&mut self, i: usize, original: &Placement, placed: &[Placement]) -> Option<Placement> {
        let base = self.shape(i, original.rotation);
        let others: Vec<_> = placed
            .iter()
            .map(|p| {
                let j = self.index(p);
                self.shape(j, p.rotation).moved(p.x, p.y)
            })
            .collect();
        for factor in [0.25, 0.5, 1., 2., 3.] {
            let step = (self.stock.separation + 0.01) * factor;
            for (dx, dy) in [
                (-step, 0.),
                (step, 0.),
                (0., step),
                (0., -step),
                (-step, step),
                (step, step),
                (-step, -step),
                (step, -step),
            ] {
                if self.stopped() {
                    return None;
                }
                let shape = base.moved(original.x + dx, original.y + dy);
                if in_board(&shape, self.stock)
                    && others
                        .iter()
                        .all(|s| compatible(&shape, s, self.stock.separation))
                {
                    return Some(Placement {
                        piece_id: original.piece_id,
                        x: original.x + dx,
                        y: original.y + dy,
                        rotation: original.rotation,
                    });
                }
            }
        }
        None
    }
    // Preserve legal placements, then relocate only legacy spacing violations.
    fn repair(&mut self, baseline: &NestingResult) -> Option<NestingResult> {
        let mut repaired = NestingResult::default();
        let mut pending = vec![];
        for sheet in &baseline.sheets {
            let mut kept = Sheet::default();
            let mut shapes = vec![];
            for p in &sheet.placements {
                if self.stopped() {
                    return None;
                }
                let i = self.index(p);
                let shape = self.shape(i, p.rotation).moved(p.x, p.y);
                if in_board(&shape, self.stock)
                    && shapes
                        .iter()
                        .all(|other| compatible(&shape, other, self.stock.separation))
                {
                    kept.placements.push(p.clone());
                    shapes.push(shape);
                } else {
                    pending.push(i)
                }
            }
            repaired.sheets.push(kept);
        }
        pending.sort_by(|&a, &b| {
            polygon_ops::area(&self.pieces[b].polygon)
                .total_cmp(&polygon_ops::area(&self.pieces[a].polygon))
        });
        for i in pending {
            if self.stopped() {
                return None;
            }
            let original = baseline.sheets.iter().enumerate().find_map(|(si, s)| {
                s.placements
                    .iter()
                    .find(|p| p.piece_id == self.pieces[i].id)
                    .map(|p| (si, p))
            });
            if let Some((si, p)) = original {
                if let Some(moved) = self.nudge(i, p, &repaired.sheets[si].placements) {
                    repaired.sheets[si].placements.push(moved);
                    continue;
                }
            }
            let mut found = false;
            for sheet in &mut repaired.sheets {
                if let Some(p) = self.find(i, &sheet.placements) {
                    sheet.placements.push(p);
                    found = true;
                    break;
                }
            }
            if !found {
                let p = self.find(i, &[])?;
                repaired.sheets.push(Sheet {
                    placements: vec![p],
                    ..Default::default()
                })
            }
        }
        validate(&repaired, self.pieces, self.rotations, self.stock).ok()?;
        refresh(&mut repaired, self.pieces, self.stock);
        Some(repaired)
    }
    fn concave(&mut self, i: usize) -> bool {
        let bounds = self.shape(i, 0.).bounds;
        polygon_ops::area(&self.pieces[i].polygon) < 0.65 * area(bounds)
    }
    // Form a compact pair on empty stock, then move both pieces as one unit.
    // This avoids choosing a locally attractive first orientation which blocks
    // the matching hook. The pair itself remains two original contours.
    fn pair(&mut self, i: usize, j: usize, placed: &[Placement]) -> Option<Vec<Placement>> {
        let used: f64 = placed
            .iter()
            .map(|p| polygon_ops::area(&self.pieces[self.index(p)].polygon))
            .sum();
        if used
            + polygon_ops::area(&self.pieces[i].polygon)
            + polygon_ops::area(&self.pieces[j].polygon)
            > self.stock.width * self.stock.height + EPS
        {
            return None;
        }
        if self.stopped() {
            return None;
        }
        let key = (i, j);
        if !self.pair_cache.contains_key(&key) {
            let arrangement = self.find_with_objective(i, &[], true).and_then(|first| {
                self.find_with_objective(j, std::slice::from_ref(&first), true)
                    .map(|second| vec![first, second])
            });
            self.pair_cache.insert(key, arrangement);
        }
        let seed = self.pair_cache.get(&key)?.as_ref()?.clone();
        let others: Vec<_> = placed
            .iter()
            .map(|p| {
                let k = self.index(p);
                self.shape(k, p.rotation).moved(p.x, p.y)
            })
            .collect();
        let occupied = others.iter().map(|s| s.bounds).reduce(union);
        let mut best: Option<(Cost, Vec<Placement>)> = None;
        for quarter in 0..4 {
            if self.stopped() {
                break;
            }
            let angle = quarter as f64 * std::f64::consts::FRAC_PI_2;
            let (sn, cs) = angle.sin_cos();
            let mut parts = vec![];
            let mut shapes = vec![];
            for p in &seed {
                let k = self.index(p);
                let a = p.rotation + angle;
                if !allowed(a, &self.rotations[k]) {
                    break;
                }
                let x = p.x * cs - p.y * sn;
                let y = p.x * sn + p.y * cs;
                shapes.push(self.shape(k, a).moved(x, y));
                parts.push(Placement {
                    piece_id: p.piece_id,
                    x,
                    y,
                    rotation: a,
                });
            }
            if parts.len() != 2 {
                continue;
            }
            let b = union(shapes[0].bounds, shapes[1].bounds);
            let width = b[2] - b[0];
            let height = b[3] - b[1];
            if width > self.stock.width + EPS || height > self.stock.height + EPS {
                continue;
            }
            let mut xs = vec![0., self.stock.width - width];
            let mut ys = vec![0., self.stock.height - height];
            let gap = self.stock.separation + 0.01;
            for o in &others {
                xs.extend([o.bounds[0], o.bounds[2] + gap, o.bounds[0] - gap - width]);
                ys.extend([o.bounds[1], o.bounds[3] + gap, o.bounds[1] - gap - height]);
            }
            let mut positions = vec![];
            for &x in &xs {
                for &y in &ys {
                    if x < -EPS
                        || y < -EPS
                        || x + width > self.stock.width + EPS
                        || y + height > self.stock.height + EPS
                    {
                        continue;
                    }
                    let bb = [x, y, x + width, y + height];
                    let cost = placement_cost(occupied.map_or(bb, |o| union(o, bb)));
                    positions.push((cost, x - b[0], y - b[1]));
                }
            }
            positions.sort_unstable_by(|a, b| cost_cmp(a.0, b.0));
            for (cost, dx, dy) in positions {
                if self.stopped() {
                    break;
                }
                if let Some((previous, _)) = &best {
                    if cost.0 > previous.0 + 0.01 {
                        break;
                    }
                    if !cost_better(cost, *previous) {
                        continue;
                    }
                }
                if shapes.iter().all(|s| {
                    let moved = s.moved(dx, dy);
                    others
                        .iter()
                        .all(|o| compatible(&moved, o, self.stock.separation))
                }) {
                    let shifted = parts
                        .iter()
                        .cloned()
                        .map(|mut p| {
                            p.x += dx;
                            p.y += dy;
                            p
                        })
                        .collect();
                    best = Some((cost, shifted));
                    break;
                }
            }
        }
        best.map(|(_, parts)| parts)
    }
    // Move already interlocked clusters together. Greedy insertion cannot find
    // this arrangement because both old sheets must change at the same time.
    fn combine_regions(&mut self, first: &Sheet, second: &Sheet) -> Option<Sheet> {
        let all: Vec<_> = first
            .placements
            .iter()
            .chain(&second.placements)
            .cloned()
            .collect();
        if all.len() > 24 || self.stopped() {
            return None;
        }
        let strips: Vec<_> = all
            .iter()
            .filter(|p| {
                let i = self.index(p);
                let b = self.shape(i, p.rotation).bounds;
                let w = b[2] - b[0];
                let h = b[3] - b[1];
                w.max(h) > 4. * w.min(h)
            })
            .cloned()
            .collect();
        if strips.len() < 2 || strips.len() > 6 {
            return None;
        }
        let gap = self.stock.separation + 0.01;
        for a in &strips {
            for b in &strips {
                if a.piece_id == b.piece_id || self.stopped() {
                    continue;
                }
                let ai = self.index(a);
                let bi = self.index(b);
                let aa = crate::conversion::rotation_range_to_samples(&self.rotations[ai], 4);
                let ba = crate::conversion::rotation_range_to_samples(&self.rotations[bi], 4);
                for ar in &aa {
                    for br in &ba {
                        if self.stopped() {
                            return None;
                        }
                        let ab = self.shape(ai, *ar).bounds;
                        let bb = self.shape(bi, *br).bounds;
                        let aw = ab[2] - ab[0];
                        let ah = ab[3] - ab[1];
                        let bw = bb[2] - bb[0];
                        let bh = bb[3] - bb[1];
                        if aw <= ah || bh <= bw || aw > self.stock.width || bh > self.stock.height {
                            continue;
                        }
                        let regions = [
                            Rect {
                                x: 0.,
                                y: ah + gap,
                                w: aw,
                                h: self.stock.height - ah - gap,
                            },
                            Rect {
                                x: aw + gap,
                                y: 0.,
                                w: self.stock.width - aw - bw - 2. * gap,
                                h: self.stock.height,
                            },
                        ];
                        if regions.iter().any(|r| r.w <= 0. || r.h <= 0.) {
                            continue;
                        }
                        let groups: Vec<Vec<_>> = [first, second]
                            .iter()
                            .map(|sheet| {
                                sheet
                                    .placements
                                    .iter()
                                    .filter(|p| {
                                        p.piece_id != a.piece_id && p.piece_id != b.piece_id
                                    })
                                    .cloned()
                                    .collect()
                            })
                            .collect();
                        if groups.iter().any(|g| g.is_empty()) {
                            continue;
                        }
                        for swap in [false, true] {
                            let Some(mut left) = self.block(&groups[usize::from(swap)], regions[0])
                            else {
                                continue;
                            };
                            let Some(middle) = self.block(&groups[usize::from(!swap)], regions[1])
                            else {
                                continue;
                            };
                            left.extend(middle);
                            left.push(Placement {
                                piece_id: a.piece_id,
                                x: -ab[0],
                                y: -ab[1],
                                rotation: *ar,
                            });
                            left.push(Placement {
                                piece_id: b.piece_id,
                                x: self.stock.width - bw - bb[0],
                                y: -bb[1],
                                rotation: *br,
                            });
                            let candidate = Sheet {
                                placements: left,
                                ..Default::default()
                            };
                            let indices: Vec<_> = all.iter().map(|p| self.index(p)).collect();
                            let pieces: Vec<_> =
                                indices.iter().map(|&i| self.pieces[i].clone()).collect();
                            let rotations: Vec<_> =
                                indices.iter().map(|&i| self.rotations[i].clone()).collect();
                            if validate(
                                &NestingResult {
                                    sheets: vec![candidate.clone()],
                                    unplaced: vec![],
                                },
                                &pieces,
                                &rotations,
                                self.stock,
                            )
                            .is_ok()
                            {
                                return Some(candidate);
                            }
                        }
                        // A new panelization need not divide matching parts into
                        // the same clusters as a previous layout. Rebuild both
                        // regions as well as trying to preserve their old blocks.
                        let mut ids: Vec<_> = all
                            .iter()
                            .filter(|p| p.piece_id != a.piece_id && p.piece_id != b.piece_id)
                            .map(|p| self.index(p))
                            .collect();
                        if ids.len() > 12 {
                            continue;
                        }
                        ids.sort_by(|&i, &j| {
                            polygon_ops::area(&self.pieces[j].polygon)
                                .total_cmp(&polygon_ops::area(&self.pieces[i].polygon))
                        });
                        let total: f64 = ids
                            .iter()
                            .map(|&i| polygon_ops::area(&self.pieces[i].polygon))
                            .sum();
                        let target = total * regions[0].w * regions[0].h
                            / (regions[0].w * regions[0].h + regions[1].w * regions[1].h);
                        let mut cuts: Vec<_> = (1..ids.len()).collect();
                        cuts.sort_by(|&i, &j| {
                            let distance = |n| {
                                (ids[..n]
                                    .iter()
                                    .map(|&k| polygon_ops::area(&self.pieces[k].polygon))
                                    .sum::<f64>()
                                    - target)
                                    .abs()
                            };
                            distance(i).total_cmp(&distance(j))
                        });
                        for cut in cuts {
                            if self.stopped() {
                                return None;
                            }
                            let Some(mut left) = self.repack_region(&ids[..cut], regions[0]) else {
                                continue;
                            };
                            let Some(middle) = self.repack_region(&ids[cut..], regions[1]) else {
                                continue;
                            };
                            left.extend(middle);
                            left.push(Placement {
                                piece_id: a.piece_id,
                                x: -ab[0],
                                y: -ab[1],
                                rotation: *ar,
                            });
                            left.push(Placement {
                                piece_id: b.piece_id,
                                x: self.stock.width - bw - bb[0],
                                y: -bb[1],
                                rotation: *br,
                            });
                            let indices: Vec<_> = all.iter().map(|p| self.index(p)).collect();
                            let pieces: Vec<_> =
                                indices.iter().map(|&i| self.pieces[i].clone()).collect();
                            let rotations: Vec<_> =
                                indices.iter().map(|&i| self.rotations[i].clone()).collect();
                            let candidate = Sheet {
                                placements: left,
                                ..Default::default()
                            };
                            if validate(
                                &NestingResult {
                                    sheets: vec![candidate.clone()],
                                    unplaced: vec![],
                                },
                                &pieces,
                                &rotations,
                                self.stock,
                            )
                            .is_ok()
                            {
                                return Some(candidate);
                            }
                        }
                    }
                }
            }
        }
        None
    }
    fn repack_region(&mut self, ids: &[usize], region: Rect) -> Option<Vec<Placement>> {
        // Form pairs on full stock first, so a narrow region can receive the
        // rotated pair instead of freezing its first part in a blocking pose.
        for pair in ids.chunks_exact(2) {
            if self.concave(pair[0]) && self.concave(pair[1]) {
                self.pair(pair[0], pair[1], &[]);
            }
        }
        for transpose in [false, true] {
            let stock = SheetConfig {
                width: if transpose { region.h } else { region.w },
                height: if transpose { region.w } else { region.h },
                separation: self.stock.separation,
            };
            let mut search = Search {
                pieces: self.pieces,
                rotations: self.rotations,
                stock: &stock,
                cache: std::mem::take(&mut self.cache),
                pair_cache: self.pair_cache.clone(),
                radii: self.radii.clone(),
                end: self.end,
                cancel: self.cancel,
            };
            let result = search.repack(ids, 1);
            self.cache = search.cache;
            if let Some(mut sheets) = result {
                if let Some(placements) = self.block(&sheets.remove(0).placements, region) {
                    return Some(placements);
                }
            }
        }
        None
    }
    fn block(&mut self, parts: &[Placement], region: Rect) -> Option<Vec<Placement>> {
        for quarter in 0..4 {
            if self.stopped() {
                return None;
            }
            let angle = quarter as f64 * std::f64::consts::FRAC_PI_2;
            let (sn, cs) = angle.sin_cos();
            let mut moved = vec![];
            let mut extent = None;
            for p in parts {
                let i = self.index(p);
                let rotation = p.rotation + angle;
                if !allowed(rotation, &self.rotations[i]) {
                    break;
                }
                let x = p.x * cs - p.y * sn;
                let y = p.x * sn + p.y * cs;
                let b = self.shape(i, rotation).moved(x, y).bounds;
                extent = Some(extent.map_or(b, |e| union(e, b)));
                moved.push(Placement {
                    piece_id: p.piece_id,
                    x,
                    y,
                    rotation,
                });
            }
            if moved.len() != parts.len() {
                continue;
            }
            let b = extent?;
            if b[2] - b[0] > region.w + EPS || b[3] - b[1] > region.h + EPS {
                continue;
            }
            for p in &mut moved {
                p.x += region.x - b[0];
                p.y += region.y - b[1]
            }
            return Some(moved);
        }
        None
    }
    fn merge_clusters(&mut self, layout: &NestingResult) -> Option<NestingResult> {
        let mut indices: Vec<_> = (0..layout.sheets.len()).collect();
        indices.sort_by(|&a, &b| {
            layout.sheets[a]
                .utilization
                .total_cmp(&layout.sheets[b].utilization)
        });
        for &source in &indices {
            for &target in &indices {
                if source == target || self.stopped() {
                    continue;
                }
                if let Some(sheet) =
                    self.combine_regions(&layout.sheets[source], &layout.sheets[target])
                {
                    let mut candidate = layout.clone();
                    candidate.sheets[source].placements.clear();
                    candidate.sheets[target] = sheet;
                    if validate(&candidate, self.pieces, self.rotations, self.stock).is_ok() {
                        refresh(&mut candidate, self.pieces, self.stock);
                        return Some(candidate);
                    }
                }
            }
        }
        None
    }
    fn repack(&mut self, ids: &[usize], max_sheets: usize) -> Option<Vec<Sheet>> {
        if ids
            .iter()
            .map(|&i| polygon_ops::area(&self.pieces[i].polygon))
            .sum::<f64>()
            > max_sheets as f64 * self.stock.width * self.stock.height + EPS
        {
            return None;
        }
        let mut sheets: Vec<Sheet> = vec![];
        let mut offset = 0;
        while offset < ids.len() {
            if self.stopped() {
                return None;
            }
            let i = ids[offset];
            if offset + 1 < ids.len() && self.concave(i) && self.concave(ids[offset + 1]) {
                let j = ids[offset + 1];
                let mut paired = false;
                for sheet in &mut sheets {
                    if let Some(parts) = self.pair(i, j, &sheet.placements) {
                        sheet.placements.extend(parts);
                        paired = true;
                        break;
                    }
                }
                if !paired && sheets.len() < max_sheets {
                    if let Some(parts) = self.pair(i, j, &[]) {
                        sheets.push(Sheet {
                            placements: parts,
                            ..Default::default()
                        });
                        paired = true
                    }
                }
                if paired {
                    offset += 2;
                    continue;
                }
            }
            let mut placed = false;
            for sheet in &mut sheets {
                if let Some(p) = self.find(i, &sheet.placements) {
                    sheet.placements.push(p);
                    placed = true;
                    break;
                }
            }
            if !placed {
                if sheets.len() >= max_sheets {
                    return None;
                }
                let p = self.find(i, &[])?;
                sheets.push(Sheet {
                    placements: vec![p],
                    ..Default::default()
                });
            }
            offset += 1;
        }
        Some(sheets)
    }
}
fn refresh(r: &mut NestingResult, pieces: &[Piece], stock: &SheetConfig) {
    r.sheets.retain(|s| !s.placements.is_empty());
    for s in &mut r.sheets {
        s.utilization = s
            .placements
            .iter()
            .map(|p| {
                polygon_ops::area(&pieces.iter().find(|x| x.id == p.piece_id).unwrap().polygon)
            })
            .sum::<f64>()
            / (stock.width * stock.height);
        s.strip_length = s
            .placements
            .iter()
            .map(|p| {
                Shape::new(
                    pieces.iter().find(|x| x.id == p.piece_id).unwrap(),
                    p.rotation,
                )
                .bounds[2]
                    + p.x
            })
            .fold(0., f64::max)
    }
}
/// Returns the best complete layout on cancellation, deadline, or exhausted search.
/// Every published result is validated against original geometry. Legacy clearance
/// violations are repaired first (possibly using extra stock); thereafter only
/// strict improvements in sheet count, then used width, then footprint are published.
pub fn improve<F: FnMut(NestingResult), P: FnMut(usize)>(
    pieces: &[Piece],
    rotations: &[RotationRange],
    stock: &SheetConfig,
    baseline: &NestingResult,
    budget: Duration,
    cancel: &AtomicBool,
    update: F,
    progress: P,
) -> Result<NestingResult, String> {
    search_layout(
        pieces, rotations, stock, baseline, budget, cancel, false, update, progress,
    )
}
/// Ordinary panelization can return unused time to other material groups once
/// a validated layout reaches the area lower bound on the number of sheets.
/// The longer `improve` action continues optimizing occupied width afterwards.
pub fn consolidate<F: FnMut(NestingResult), P: FnMut(usize)>(
    pieces: &[Piece],
    rotations: &[RotationRange],
    stock: &SheetConfig,
    baseline: &NestingResult,
    budget: Duration,
    cancel: &AtomicBool,
    update: F,
    progress: P,
) -> Result<NestingResult, String> {
    search_layout(
        pieces, rotations, stock, baseline, budget, cancel, true, update, progress,
    )
}
fn search_layout<F: FnMut(NestingResult), P: FnMut(usize)>(
    pieces: &[Piece],
    rotations: &[RotationRange],
    stock: &SheetConfig,
    baseline: &NestingResult,
    budget: Duration,
    cancel: &AtomicBool,
    finish_at_minimum: bool,
    mut update: F,
    mut progress: P,
) -> Result<NestingResult, String> {
    // Older solver versions could save non-overlapping layouts with insufficient
    // clearance. Accept their identities/rotations, then repair at full spacing.
    let spacing_valid = validate(baseline, pieces, rotations, stock).is_ok();
    let zero_gap = SheetConfig {
        separation: 0.,
        ..stock.clone()
    };
    validate(baseline, pieces, rotations, &zero_gap)?;
    let mut search = Search {
        pieces,
        rotations,
        stock,
        cache: HashMap::new(),
        pair_cache: HashMap::new(),
        radii: HashMap::new(),
        end: Instant::now() + budget,
        cancel,
    };
    let mut best = baseline.clone();
    if !spacing_valid {
        let Some(repaired) = search.repair(baseline) else {
            return Ok(best);
        };
        best = repaired;
        update(best.clone());
    }
    if !search.stopped() {
        let mut anchored = best.clone();
        let mut shifted = false;
        for sheet in &mut anchored.sheets {
            let left = sheet
                .placements
                .iter()
                .map(|p| {
                    let i = search.index(p);
                    search.shape(i, p.rotation).bounds[0] + p.x
                })
                .fold(f64::INFINITY, f64::min);
            if left > 0.01 {
                for p in &mut sheet.placements {
                    p.x -= left
                }
                shifted = true
            }
        }
        if shifted && validate(&anchored, pieces, rotations, stock).is_ok() {
            refresh(&mut anchored, pieces, stock);
            best = anchored;
            update(best.clone());
        }
    }
    let mut best_score = search.score(&best);
    let minimum_sheets = (pieces
        .iter()
        .map(|p| polygon_ops::area(&p.polygon))
        .sum::<f64>()
        / (stock.width * stock.height)
        - 1e-9)
        .ceil() as usize;
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    for pass in 0..32 {
        if search.stopped() || (finish_at_minimum && best_score.0 <= minimum_sheets) {
            break;
        }
        progress(pass + 1);
        if let Some(merged) = search.merge_clusters(&best) {
            best = merged;
            best_score = search.score(&best);
            update(best.clone());
            continue;
        }
        // Prepare sparse clusters before spending time inserting into dense sheets.
        // This exposes coordinated merges even on the short ordinary-run budget.
        if pass == 0 {
            let mut sparse: Vec<_> = (0..best.sheets.len())
                .filter(|&i| {
                    best.sheets[i].utilization < 0.65 && best.sheets[i].placements.len() > 1
                })
                .collect();
            sparse.sort_by(|&a, &b| {
                best.sheets[a]
                    .utilization
                    .total_cmp(&best.sheets[b].utilization)
            });
            for source in sparse {
                if search.stopped() {
                    break;
                }
                let mut ids: Vec<_> = best.sheets[source]
                    .placements
                    .iter()
                    .map(|p| search.index(p))
                    .collect();
                order(&mut ids, pieces, 0, &mut rng);
                if let Some(sheets) = search.repack(&ids, 1) {
                    let mut candidate = best.clone();
                    candidate.sheets[source] = sheets[0].clone();
                    let score = search.score(&candidate);
                    if layout_better(score, best_score)
                        && validate(&candidate, pieces, rotations, stock).is_ok()
                    {
                        refresh(&mut candidate, pieces, stock);
                        best = candidate;
                        best_score = score;
                        update(best.clone());
                    }
                }
            }
            if let Some(merged) = search.merge_clusters(&best) {
                best = merged;
                best_score = search.score(&best);
                update(best.clone());
                continue;
            }
        }
        let mut sources: Vec<_> = (0..best.sheets.len()).collect();
        sources.sort_by(|&a, &b| {
            best.sheets[a]
                .utilization
                .total_cmp(&best.sheets[b].utilization)
        });
        let old_count = best.sheets.len();
        for source in sources {
            if search.stopped() {
                break;
            }
            // Try coordinated cluster moves before freezing either sheet's placements.
            let mut merged = false;
            for target in 0..best.sheets.len() {
                if target == source || search.stopped() {
                    continue;
                }
                let first = best.sheets[source].clone();
                let second = best.sheets[target].clone();
                if let Some(sheet) = search.combine_regions(&first, &second) {
                    let mut candidate = best.clone();
                    candidate.sheets[source].placements.clear();
                    candidate.sheets[target] = sheet;
                    if validate(&candidate, pieces, rotations, stock).is_ok() {
                        refresh(&mut candidate, pieces, stock);
                        best_score = search.score(&candidate);
                        best = candidate;
                        update(best.clone());
                        merged = true;
                        break;
                    }
                }
            }
            if merged {
                break;
            }
            // Compact a legal sparse sheet even when no whole-sheet merge succeeds.
            let mut ids: Vec<_> = best.sheets[source]
                .placements
                .iter()
                .map(|p| search.index(p))
                .collect();
            order(&mut ids, pieces, pass, &mut rng);
            if let Some(sheets) = search.repack(&ids, 1) {
                let mut candidate = best.clone();
                candidate.sheets[source] = sheets[0].clone();
                let score = search.score(&candidate);
                if cost_better(score.1, best_score.1)
                    && validate(&candidate, pieces, rotations, stock).is_ok()
                {
                    refresh(&mut candidate, pieces, stock);
                    best = candidate;
                    best_score = score;
                    update(best.clone());
                }
            }
            // First migrate small pieces without disturbing the remaining layout.
            let mut candidate = best.clone();
            let mut incoming = candidate.sheets[source].placements.clone();
            incoming.sort_by(|a, b| {
                polygon_ops::area(&pieces[search.index(a)].polygon)
                    .total_cmp(&polygon_ops::area(&pieces[search.index(b)].polygon))
            });
            for p in incoming {
                let i = search.index(&p);
                let mut targets: Vec<_> = (0..candidate.sheets.len()).collect();
                targets.sort_by(|&a, &b| {
                    candidate.sheets[a]
                        .utilization
                        .total_cmp(&candidate.sheets[b].utilization)
                });
                for target in targets {
                    if target == source {
                        continue;
                    }
                    if let Some(p) = search.find(i, &candidate.sheets[target].placements) {
                        candidate.sheets[source]
                            .placements
                            .retain(|x| x.piece_id != p.piece_id);
                        candidate.sheets[target].placements.push(p);
                        break;
                    }
                }
                if search.stopped() {
                    break;
                }
            }
            let score = search.score(&candidate);
            if layout_better(score, best_score) {
                if validate(&candidate, pieces, rotations, stock).is_ok() {
                    refresh(&mut candidate, pieces, stock);
                    best = candidate;
                    best_score = score;
                    update(best.clone());
                    break;
                }
            }
            // Jointly repack sparse sheets first, before expensive dense targets.
            let mut targets: Vec<_> = (0..best.sheets.len()).collect();
            targets.sort_by(|&a, &b| {
                best.sheets[a]
                    .utilization
                    .total_cmp(&best.sheets[b].utilization)
            });
            for target in targets {
                if source == target || search.stopped() {
                    continue;
                }
                let mut ids: Vec<_> = best.sheets[source]
                    .placements
                    .iter()
                    .chain(&best.sheets[target].placements)
                    .map(|p| search.index(p))
                    .collect();
                order(&mut ids, pieces, pass, &mut rng);
                if let Some(sheets) = search.repack(&ids, 1) {
                    let mut candidate = best.clone();
                    candidate.sheets[source].placements.clear();
                    candidate.sheets[target] = sheets[0].clone();
                    if validate(&candidate, pieces, rotations, stock).is_ok() {
                        refresh(&mut candidate, pieces, stock);
                        best_score = search.score(&candidate);
                        best = candidate;
                        update(best.clone());
                        break;
                    }
                }
            }
            // Source indices refer to the pre-merge layout; restart after sheet removal.
            if best.sheets.len() < old_count {
                break;
            }
        }
        if search.stopped() || (finish_at_minimum && best_score.0 <= minimum_sheets) {
            break;
        }
        // Deterministic restarts permit several sheets to exchange parts at once.
        let mut ids: Vec<_> = (0..pieces.len()).collect();
        order(&mut ids, pieces, pass, &mut rng);
        if let Some(sheets) = search.repack(&ids, best.sheets.len()) {
            let mut candidate = NestingResult {
                sheets,
                unplaced: vec![],
            };
            let score = search.score(&candidate);
            if layout_better(score, best_score)
                && validate(&candidate, pieces, rotations, stock).is_ok()
            {
                refresh(&mut candidate, pieces, stock);
                best = candidate;
                best_score = score;
                update(best.clone())
            }
        }
    }
    Ok(best)
}
fn order(ids: &mut [usize], pieces: &[Piece], pass: usize, rng: &mut rand::rngs::StdRng) {
    match pass % 4 {
        0 => ids.sort_by(|&a, &b| {
            polygon_ops::area(&pieces[b].polygon).total_cmp(&polygon_ops::area(&pieces[a].polygon))
        }),
        1 => ids.sort_by(|&a, &b| {
            let x = Shape::new(&pieces[a], 0.).bounds;
            let y = Shape::new(&pieces[b], 0.).bounds;
            (y[2] - y[0])
                .max(y[3] - y[1])
                .total_cmp(&(x[2] - x[0]).max(x[3] - x[1]))
        }),
        2 => ids.sort_by(|&a, &b| {
            let ratio = |i: usize| {
                polygon_ops::area(&pieces[i].polygon) / area(Shape::new(&pieces[i], 0.).bounds)
            };
            ratio(a).total_cmp(&ratio(b))
        }),
        _ => ids.shuffle(rng),
    }
}
