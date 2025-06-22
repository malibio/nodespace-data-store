use nodespace_core_types::NodeId;
use surrealdb::sql::Thing;

// Conversion functions to avoid orphan rule violations
pub fn node_id_to_thing(node_id: &NodeId) -> Thing {
    // SurrealDB expects a valid record ID format
    let clean_id = node_id.as_str().replace("-", "_");
    Thing::from(("nodes", clean_id.as_str()))
}

#[allow(dead_code)]
pub fn thing_to_node_id(thing: &Thing) -> NodeId {
    NodeId::from_string(thing.id.to_string())
}

#[allow(dead_code)]
pub fn string_to_node_id(s: &str) -> NodeId {
    NodeId::from_string(s.to_string())
}
