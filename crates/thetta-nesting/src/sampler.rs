use jagua_rs::geometry::geo_enums::RotationRange;
use rand::Rng;
use rand::rngs::StdRng;
use thetta_geometry::types::SheetConfig;

#[derive(Debug, Clone, Copy)]
pub struct Sample {
    pub x: f64,
    pub y: f64,
    pub rotation: f64,
}

pub fn uniform_sample(
    rng: &mut StdRng,
    sheet: &SheetConfig,
    rotation_range: Option<&RotationRange>,
) -> Sample {
    let x = rng.gen_range(0.0..sheet.width.max(0.001));
    let y = rng.gen_range(0.0..sheet.height.max(0.001));

    let rotation = match rotation_range {
        Some(RotationRange::None) => 0.0,
        Some(RotationRange::Continuous) | None => rng.gen_range(0.0..std::f64::consts::TAU),
        Some(RotationRange::Discrete(angles)) => {
            if angles.is_empty() {
                0.0
            } else {
                let idx = rng.gen_range(0..angles.len());
                normalize_angle(angles[idx] as f64)
            }
        }
    };

    Sample { x, y, rotation }
}

pub fn discrete_rotation_sample(rng: &mut StdRng, orientations: &[f64]) -> f64 {
    if orientations.is_empty() {
        0.0
    } else {
        let idx = rng.gen_range(0..orientations.len());
        normalize_angle(orientations[idx])
    }
}

pub fn coordinate_descent_refine<F>(
    seed: Sample,
    mut score: F,
    step: f64,
    iters: usize,
    rotation_range: Option<&RotationRange>,
) -> Sample
where
    F: FnMut(Sample) -> f64,
{
    let mut best = snap_sample(seed, rotation_range);
    let mut best_score = score(best);
    let mut delta = step;

    for _ in 0..iters {
        let candidates = [
            Sample {
                x: best.x + delta,
                ..best
            },
            Sample {
                x: best.x - delta,
                ..best
            },
            Sample {
                y: best.y + delta,
                ..best
            },
            Sample {
                y: best.y - delta,
                ..best
            },
            Sample {
                rotation: best.rotation + delta.to_radians(),
                ..best
            },
            Sample {
                rotation: best.rotation - delta.to_radians(),
                ..best
            },
        ];

        let mut improved = false;
        for cand in candidates {
            let c = snap_sample(cand, rotation_range);
            let s = score(c);
            if s < best_score {
                best = c;
                best_score = s;
                improved = true;
            }
        }

        if !improved {
            delta *= 0.5;
        }
    }

    snap_sample(best, rotation_range)
}

fn snap_sample(sample: Sample, rotation_range: Option<&RotationRange>) -> Sample {
    let rotation = match rotation_range {
        Some(RotationRange::None) => 0.0,
        Some(RotationRange::Continuous) | None => normalize_angle(sample.rotation),
        Some(RotationRange::Discrete(angles)) => {
            if angles.is_empty() {
                0.0
            } else {
                let mut best = angles[0] as f64;
                let mut best_dist = circular_distance(sample.rotation, best);
                for angle in angles.iter().skip(1) {
                    let candidate = *angle as f64;
                    let dist = circular_distance(sample.rotation, candidate);
                    if dist < best_dist {
                        best = candidate;
                        best_dist = dist;
                    }
                }
                normalize_angle(best)
            }
        }
    };

    Sample {
        rotation,
        ..sample
    }
}

fn circular_distance(a: f64, b: f64) -> f64 {
    let tau = std::f64::consts::TAU;
    let diff = (normalize_angle(a) - normalize_angle(b)).abs();
    diff.min(tau - diff)
}

fn normalize_angle(mut angle: f64) -> f64 {
    let tau = std::f64::consts::TAU;
    angle %= tau;
    if angle < 0.0 {
        angle += tau;
    }
    angle
}
