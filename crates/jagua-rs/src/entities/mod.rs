mod container;
mod item;
mod layout;
mod placed_item;

pub use container::Container;
pub use item::Item;
pub use layout::{Layout, LayoutSnapshot};
pub use placed_item::{PItemKey, PlacedItem};

pub trait Instance {
    fn item(&self, id: usize) -> &Item;
    fn items(&self) -> impl Iterator<Item = &Item>;
}
