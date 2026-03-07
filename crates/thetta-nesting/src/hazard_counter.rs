use jagua_rs::collision_detection::hazards::collector::HazardCollector;
use jagua_rs::collision_detection::hazards::{HazKey, HazardEntity};
use std::collections::BTreeMap;

pub struct CountingCollector {
    hazards: BTreeMap<HazKey, HazardEntity>,
    excluded: Option<HazKey>,
}

impl CountingCollector {
    pub fn new(excluded: Option<HazKey>) -> Self {
        let mut hazards = BTreeMap::new();
        if let Some(hk) = excluded {
            hazards.insert(hk, HazardEntity::Exterior);
        }
        Self { hazards, excluded }
    }

    pub fn count(&self) -> usize {
        let total = self.hazards.len();
        if self.excluded.is_some() {
            total.saturating_sub(1)
        } else {
            total
        }
    }
}

impl HazardCollector for CountingCollector {
    fn contains_key(&self, hkey: HazKey) -> bool {
        self.hazards.contains_key(&hkey)
    }

    fn insert(&mut self, hkey: HazKey, entity: HazardEntity) {
        self.hazards.insert(hkey, entity);
    }

    fn remove_by_key(&mut self, hkey: HazKey) {
        self.hazards.remove(&hkey);
    }

    fn len(&self) -> usize {
        self.hazards.len()
    }

    fn iter(&self) -> impl Iterator<Item = (HazKey, &HazardEntity)> {
        self.hazards.iter().map(|(k, v)| (*k, v))
    }
}
