use crate::entities::{Instance, Layout, PItemKey};
use crate::geometry::DTransformation;
use crate::probs::spp::entities::SPInstance;
use crate::probs::spp::entities::strip::Strip;

#[derive(Clone)]
pub struct SPProblem {
    pub instance: SPInstance,
    pub strip: Strip,
    pub layout: Layout,
    pub item_demand_qtys: Vec<usize>,
}

impl SPProblem {
    pub fn new(instance: SPInstance) -> Self {
        let item_demand_qtys = instance.items.iter().map(|(_, qty)| *qty).collect();
        let strip = instance.base_strip;
        let layout = Layout::new(strip.into());

        Self {
            instance,
            strip,
            layout,
            item_demand_qtys,
        }
    }

    pub fn change_strip_width(&mut self, new_width: f32) {
        self.strip.set_width(new_width);
        self.layout.swap_container(self.strip.into());
    }

    pub fn fit_strip(&mut self) {
        let item_x_max = self
            .layout
            .placed_items
            .values()
            .map(|pi| pi.shape.bbox.x_max)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();

        let fitted_width = item_x_max + self.strip.shape_modify_config.offset.unwrap_or(0.0);
        self.change_strip_width(fitted_width.max(1.0));
    }

    pub fn place_item(&mut self, placement: SPPlacement) -> PItemKey {
        self.register_included_item(placement.item_id);
        let item = self.instance.item(placement.item_id);
        self.layout.place_item(item, placement.d_transf)
    }

    pub fn remove_item(&mut self, pkey: PItemKey) -> SPPlacement {
        let pi = self.layout.remove_item(pkey);
        self.deregister_included_item(pi.item_id);

        SPPlacement {
            item_id: pi.item_id,
            d_transf: pi.d_transf,
        }
    }

    fn register_included_item(&mut self, item_id: usize) {
        self.item_demand_qtys[item_id] -= 1;
    }

    fn deregister_included_item(&mut self, item_id: usize) {
        self.item_demand_qtys[item_id] += 1;
    }

    pub fn strip_width(&self) -> f32 {
        self.strip.width
    }

    pub fn n_placed_items(&self) -> usize {
        self.layout.placed_items.len()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SPPlacement {
    pub item_id: usize,
    pub d_transf: DTransformation,
}
