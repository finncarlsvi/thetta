use std::collections::BTreeMap;

use crate::collision_detection::hazards::{Hazard, HazardEntity};
use crate::collision_detection::CDEngine;
use crate::entities::{Container, Item, PItemKey, PlacedItem};
use crate::geometry::DTransformation;

#[derive(Clone)]
pub struct Layout {
    pub container: Container,
    pub placed_items: BTreeMap<PItemKey, PlacedItem>,
    cde: CDEngine,
    next_key: PItemKey,
}

#[derive(Clone, Debug)]
pub struct LayoutSnapshot {
    pub container: Container,
    pub placed_items: BTreeMap<PItemKey, PlacedItem>,
}

impl Layout {
    pub fn new(container: Container) -> Self {
        let cde = container.base_cde.as_ref().clone();
        Self {
            container,
            placed_items: BTreeMap::new(),
            cde,
            next_key: 0,
        }
    }

    pub fn from_snapshot(ls: &LayoutSnapshot) -> Self {
        let mut layout = Self::new(ls.container.clone());
        for (pk, pi) in &ls.placed_items {
            layout.placed_items.insert(*pk, pi.clone());
            layout.cde.register_hazard(Hazard::new(
                HazardEntity::PlacedItem {
                    pk: *pk,
                    item_id: pi.item_id,
                },
                pi.shape.clone(),
                true,
            ));
            layout.next_key = layout.next_key.max(*pk + 1);
        }
        layout
    }

    pub fn swap_container(&mut self, container: Container) {
        self.container = container;
        self.cde = self.container.base_cde.as_ref().clone();

        for (pk, pi) in &self.placed_items {
            self.cde.register_hazard(Hazard::new(
                HazardEntity::PlacedItem {
                    pk: *pk,
                    item_id: pi.item_id,
                },
                pi.shape.clone(),
                true,
            ));
        }
    }

    pub fn save(&self) -> LayoutSnapshot {
        LayoutSnapshot {
            container: self.container.clone(),
            placed_items: self.placed_items.clone(),
        }
    }

    pub fn restore(&mut self, snapshot: &LayoutSnapshot) {
        *self = Layout::from_snapshot(snapshot);
    }

    pub fn place_item(&mut self, item: &Item, d_transformation: DTransformation) -> PItemKey {
        let pk = self.next_key;
        self.next_key += 1;
        let pi = PlacedItem::new(item, d_transformation);
        self.placed_items.insert(pk, pi.clone());
        self.cde.register_hazard(Hazard::new(
            HazardEntity::PlacedItem {
                pk,
                item_id: pi.item_id,
            },
            pi.shape,
            true,
        ));
        pk
    }

    pub fn remove_item(&mut self, pk: PItemKey) -> PlacedItem {
        let pi = self.placed_items.remove(&pk).expect("key is not valid anymore");
        self.cde.deregister_hazard_by_entity(HazardEntity::PlacedItem {
            pk,
            item_id: pi.item_id,
        });
        pi
    }

    pub fn cde(&self) -> &CDEngine {
        &self.cde
    }

    pub fn is_feasible(&self) -> bool {
        self.placed_items.iter().all(|(pk, pi)| {
            let hkey = self.cde.haz_key_from_pi_key(*pk).expect("hazard key present");
            !self.cde.detect_poly_collision(&pi.shape, &hkey)
        })
    }

    pub fn density(&self, instance: &impl crate::entities::Instance) -> f32 {
        self.placed_item_area(instance) / self.container.area().max(f32::EPSILON)
    }

    pub fn placed_item_area(&self, instance: &impl crate::entities::Instance) -> f32 {
        self.placed_items
            .iter()
            .map(|(_, pi)| instance.item(pi.item_id).area())
            .sum::<f32>()
    }
}
