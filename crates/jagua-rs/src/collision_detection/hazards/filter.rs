use crate::collision_detection::hazards::HazKey;
use crate::collision_detection::hazards::collector::HazardCollector;

pub trait HazardFilter {
    fn is_irrelevant(&self, haz_key: HazKey) -> bool;
}

#[derive(Clone, Debug)]
pub struct NoFilter;

impl HazardFilter for NoFilter {
    fn is_irrelevant(&self, _haz_key: HazKey) -> bool {
        false
    }
}

impl HazardFilter for HazKey {
    fn is_irrelevant(&self, hk: HazKey) -> bool {
        *self == hk
    }
}

impl<T> HazardFilter for T
where
    T: HazardCollector,
{
    fn is_irrelevant(&self, hkey: HazKey) -> bool {
        self.contains_key(hkey)
    }
}
