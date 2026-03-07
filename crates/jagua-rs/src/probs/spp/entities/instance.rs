use crate::entities::{Container, Instance, Item};
use crate::probs::spp::entities::Strip;

#[derive(Debug, Clone)]
pub struct SPInstance {
    pub items: Vec<(Item, usize)>,
    pub base_strip: Strip,
}

impl SPInstance {
    pub fn new(items: Vec<(Item, usize)>, base_strip: Strip) -> Self {
        assert!(
            items.iter().enumerate().all(|(i, (item, _))| item.id == i),
            "All items should have consecutive IDs starting from 0"
        );

        Self { items, base_strip }
    }

    pub fn item_area(&self) -> f32 {
        self.items
            .iter()
            .map(|(item, qty)| item.shape_orig.area() * *qty as f32)
            .sum()
    }

    pub fn item_qty(&self, id: usize) -> usize {
        self.items[id].1
    }

    pub fn total_item_qty(&self) -> usize {
        self.items.iter().map(|(_, qty)| *qty).sum()
    }
}

impl Instance for SPInstance {
    fn item(&self, id: usize) -> &Item {
        &self.items[id].0
    }

    fn items(&self) -> impl Iterator<Item = &Item> {
        self.items.iter().map(|(item, _)| item)
    }
}

#[allow(dead_code)]
fn _dummy(_c: Option<&Container>) {}
