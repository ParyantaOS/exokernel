//! Object Store — "Everything is a Database"
//!
//! Replaces traditional filesystem with tagged, queryable objects.
//! All access is capability-gated.

pub mod store;
pub mod gated;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;

// ─── Core types ──────────────────────────────────────────────────

/// Content-addressed object identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ObjId(u64);

impl ObjId {
    pub fn raw(&self) -> u64 {
        self.0
    }

    /// Compute the ObjId for given content (same hash as Object::new).
    pub fn from_content(data: &[u8]) -> Self {
        ObjId(hash_content(data))
    }
}

impl core::fmt::Display for ObjId {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "Obj#{:04x}", self.0 & 0xFFFF) // short display
    }
}

/// FNV-1a hash for content addressing.
fn hash_content(data: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325; // FNV offset basis
    for &byte in data {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3); // FNV prime
    }
    hash
}

/// An object in the store.
#[derive(Debug, Clone)]
pub struct Object {
    pub id: ObjId,
    pub content: Vec<u8>,
    pub tags: Vec<String>,
    pub metadata: BTreeMap<String, String>,
}

impl Object {
    /// Create a new object from raw content.
    pub fn new(content: &[u8]) -> Self {
        let id = ObjId(hash_content(content));
        Object {
            id,
            content: content.to_vec(),
            tags: Vec::new(),
            metadata: BTreeMap::new(),
        }
    }

    /// Builder: add a tag.
    pub fn with_tag(mut self, tag: &str) -> Self {
        self.tags.push(String::from(tag));
        self
    }

    /// Builder: add metadata.
    pub fn with_meta(mut self, key: &str, val: &str) -> Self {
        self.metadata.insert(String::from(key), String::from(val));
        self
    }
}

/// Object Store errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjError {
    NotFound,
    AlreadyExists,
}

impl core::fmt::Display for ObjError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            ObjError::NotFound => write!(f, "not found"),
            ObjError::AlreadyExists => write!(f, "already exists"),
        }
    }
}
