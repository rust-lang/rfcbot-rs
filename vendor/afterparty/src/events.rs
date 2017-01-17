//! Representations of Github events

extern crate case;
extern crate serde;
extern crate serde_json;
extern crate env_logger;

use case::CaseExt;
use std::collections::BTreeMap;

// generated Event enum goes here

/// Enumeration of availble Github events
include!(concat!(env!("OUT_DIR"), "/events.rs"));

/// to support enum deserialization, we need to
/// patch the raw json from github with a field for the enum
/// name
pub fn patch_payload_json(event: &str, payload: &str) -> String {
    let mut patched_payload = "{\"".to_string();
    patched_payload.push_str(event.to_camel().as_ref());
    patched_payload.push_str("\":");
    patched_payload.push_str(payload);
    patched_payload.push_str("}");
    patched_payload
}

// provide a sensible default for our serde_json::Value type wrapper
impl Default for Value {
    fn default() -> Value {
        Value { json: serde_json::Value::Object(BTreeMap::new()) }
    }
}
