use rand::Rng;
use rand::rngs::StdRng;
use thetta_geometry::types::SheetConfig;

#[derive(Debug, Clone, Copy)]
pub struct Sample {
    pub x: f64,
    pub y: f64,
    pub rotation: f64,
}

pub fn uniform_sample(rng: &mut StdRng, sheet: &SheetConfig) -> Sample {
    Sample {
        x: rng.gen_range(0.0..sheet.width.max(0.001)),
        y: rng.gen_range(0.0..sheet.height.max(0.001)),
        rotation: rng.gen_range(0.0..std::f64::consts::TAU),
    }
}

pub fn coordinate_descent_refine<F>(seed: Sample, mut score: F, step: f64, iters: usize) -> Sample
where
    F: FnMut(Sample) -> f64,
{
    let mut best = seed;
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
        for c in candidates {
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

    best
}
