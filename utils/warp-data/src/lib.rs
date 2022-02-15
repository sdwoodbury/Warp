use warp_module::Module;

use chrono::{DateTime, Utc};
use uuid::Uuid;

// Placeholder for DataObject
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataObject {
    pub id: Uuid,
    pub version: i32,
    pub timestamp: DateTime<Utc>,
    pub size: u64,
    pub module: Module,
    pub payload: (),
}