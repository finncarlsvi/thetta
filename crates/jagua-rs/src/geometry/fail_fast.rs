use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SPSurrogateConfig {
    pub n_pole_limits: [(usize, f32); 3],
    pub n_ff_poles: usize,
    pub n_ff_piers: usize,
}
