//! Matrix room types.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatrixRoom {
    pub room_id: String,
    pub name: Option<String>,
    pub topic: Option<String>,
}
