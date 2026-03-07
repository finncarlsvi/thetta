use std::collections::BTreeMap;

use crate::collision_detection::hazards::filter::HazardFilter;
use crate::collision_detection::hazards::{HazKey, HazardEntity};

pub trait HazardCollector: HazardFilter {
    fn contains_key(&self, hkey: HazKey) -> bool;
    fn insert(&mut self, hkey: HazKey, entity: HazardEntity);
    fn remove_by_key(&mut self, hkey: HazKey);
    fn len(&self) -> usize;
    fn iter(&self) -> impl Iterator<Item = (HazKey, &HazardEntity)>;
}

pub type BasicHazardCollector = BTreeMap<HazKey, HazardEntity>;

impl HazardCollector for BasicHazardCollector {
    fn contains_key(&self, hkey: HazKey) -> bool {
        self.contains_key(&hkey)
    }

    fn insert(&mut self, hkey: HazKey, entity: HazardEntity) {
        self.insert(hkey, entity);
    }

    fn remove_by_key(&mut self, hkey: HazKey) {
        self.remove(&hkey);
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn iter(&self) -> impl Iterator<Item = (HazKey, &HazardEntity)> {
        self.iter().map(|(k, v)| (*k, v))
    }
}
