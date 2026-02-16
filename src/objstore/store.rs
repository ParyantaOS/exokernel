//! In-memory Object Store backed by BTreeMap.

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use spin::Mutex;
use super::{ObjId, Object, ObjError};

/// Global object store instance.
static STORE: Mutex<StoreInner> = Mutex::new(StoreInner::new());

struct StoreInner {
    objects: Option<BTreeMap<ObjId, Object>>,
    tag_index: Option<BTreeMap<String, Vec<ObjId>>>,
}

impl StoreInner {
    const fn new() -> Self {
        Self { objects: None, tag_index: None }
    }

    fn objects(&mut self) -> &mut BTreeMap<ObjId, Object> {
        self.objects.get_or_insert_with(BTreeMap::new)
    }

    fn tag_index(&mut self) -> &mut BTreeMap<String, Vec<ObjId>> {
        self.tag_index.get_or_insert_with(BTreeMap::new)
    }
}

/// Store an object. Returns its content-addressed ID.
pub fn create(obj: Object) -> Result<ObjId, ObjError> {
    let mut store = STORE.lock();
    let id = obj.id;

    if store.objects().contains_key(&id) {
        return Err(ObjError::AlreadyExists);
    }

    // Update tag index
    for tag in &obj.tags {
        store.tag_index()
            .entry(tag.clone())
            .or_insert_with(Vec::new)
            .push(id);
    }

    store.objects().insert(id, obj);
    Ok(id)
}

/// Read an object by ID.
pub fn read(id: ObjId) -> Result<Object, ObjError> {
    let store = STORE.lock();
    store.objects.as_ref()
        .and_then(|m| m.get(&id))
        .cloned()
        .ok_or(ObjError::NotFound)
}

/// Query objects matching a tag. Returns list of IDs.
pub fn query_by_tag(tag: &str) -> Vec<ObjId> {
    let store = STORE.lock();
    store.tag_index.as_ref()
        .and_then(|idx| idx.get(tag))
        .cloned()
        .unwrap_or_default()
}

/// Delete an object by ID.
pub fn delete(id: ObjId) -> Result<(), ObjError> {
    let mut store = STORE.lock();
    let obj = store.objects()
        .remove(&id)
        .ok_or(ObjError::NotFound)?;

    // Clean up tag index
    for tag in &obj.tags {
        if let Some(ids) = store.tag_index().get_mut(tag) {
            ids.retain(|i| *i != id);
        }
    }
    Ok(())
}

/// Count of objects in the store.
pub fn count() -> usize {
    let store = STORE.lock();
    store.objects.as_ref().map_or(0, |m| m.len())
}
