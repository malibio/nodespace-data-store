// Conversion functions for SurrealDB 2.x compatibility
// Only compiled when migration feature is enabled
#[cfg(feature = "migration")]
use nodespace_core_types::NodeId;
#[cfg(feature = "migration")]
use surrealdb::{sql::Thing, RecordId};

#[cfg(feature = "migration")]
pub fn node_id_to_record_id(node_id: &NodeId) -> RecordId {
    // SurrealDB 2.x expects RecordId format
    let clean_id = node_id.as_str().replace("-", "_");
    RecordId::from(("nodes", clean_id.as_str()))
}

#[cfg(feature = "migration")]
// Legacy function for backward compatibility with queries
pub fn node_id_to_thing(node_id: &NodeId) -> Thing {
    // Still needed for some query contexts
    let clean_id = node_id.as_str().replace("-", "_");
    Thing::from(("nodes", clean_id.as_str()))
}

#[cfg(feature = "migration")]
#[allow(dead_code)]
pub fn thing_to_node_id(thing: &Thing) -> NodeId {
    NodeId::from_string(thing.id.to_string())
}

#[cfg(feature = "migration")]
#[allow(dead_code)]
pub fn record_id_to_node_id(record_id: &RecordId) -> NodeId {
    NodeId::from_string(record_id.key().to_string())
}

#[cfg(feature = "migration")]
#[allow(dead_code)]
pub fn string_to_node_id(s: &str) -> NodeId {
    NodeId::from_string(s.to_string())
}
