//! Pipeline Element
//!
//! Define custom processing for `Parser` output.
use crate::crawler::Item;
pub use crate::pipeline::elements::timestamping::{TimeOffset, Timestamping};

mod timestamping;

pub trait PipelineElement {
    /// Exposes a way to implement custom logic for processing `Parser` output.
    /// Accepts an `Item` and returns a new `Item`.
    fn process_item(&self, item: Item) -> Item;
}
