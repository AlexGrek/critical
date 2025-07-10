use serde::Deserialize;

pub mod entities;
pub mod requests;
pub mod state_entities;

// Common enum to match on kind
#[derive(Debug, Deserialize)]
pub struct KindOnly {
    pub kind: String,
}

pub mod prelude {
    pub use crate::entities;
    pub use crate::state_entities;
    pub use crate::requests;
    pub use gitops_lib::store;
    pub use gitops_lib;
    pub use crate::KindOnly;
}
