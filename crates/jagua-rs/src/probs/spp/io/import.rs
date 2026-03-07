use anyhow::{Result, ensure};

use crate::entities::Item;
use crate::geometry::shape_modification::ShapeModifyConfig;
use crate::io::import::Importer;
use crate::probs::spp::entities::{SPInstance, Strip};
use crate::probs::spp::io::ext_repr::ExtSPInstance;

pub fn import_instance(importer: &Importer, ext_instance: &ExtSPInstance) -> Result<SPInstance> {
    let mut items: Vec<(Item, usize)> = ext_instance
        .items
        .iter()
        .map(|ext_item| {
            let item = importer.import_item(&ext_item.base)?;
            let demand = ext_item.demand as usize;
            Ok((item, demand))
        })
        .collect::<Result<Vec<(Item, usize)>>>()?;

    items.sort_by_key(|(item, _)| item.id);
    items.retain(|(_, demand)| *demand > 0);

    ensure!(
        items.iter().enumerate().all(|(i, (item, _))| item.id == i),
        "All items should have consecutive IDs starting from 0"
    );
    ensure!(
        !items.is_empty(),
        "ExtSPInstance must have at least one item with positive demand"
    );

    let total_item_area = items
        .iter()
        .map(|(item, demand)| item.area() * *demand as f32)
        .sum::<f32>();

    let fixed_height = ext_instance.strip_height;
    let width = total_item_area / fixed_height;

    let base_strip = Strip::new(
        fixed_height,
        importer.cde_config,
        ShapeModifyConfig {
            offset: importer.shape_modify_config.offset,
            simplify_tolerance: None,
            narrow_concavity_cutoff: None,
        },
        width,
    )?;

    Ok(SPInstance::new(items, base_strip))
}
