//! Capability-gated Object Store access.
//!
//! Wraps raw store operations with capability checks.
//! WRITE cap required to create, READ to read/query, DELETE to delete.

use alloc::vec::Vec;
use super::{ObjId, Object, ObjError, store};
use crate::caps::{self, CapId, Rights, CapError};

/// Error from a gated store operation.
#[derive(Debug)]
pub enum GatedError {
    Cap(CapError),
    Store(ObjError),
}

impl core::fmt::Display for GatedError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            GatedError::Cap(e) => write!(f, "{}", e),
            GatedError::Store(e) => write!(f, "{}", e),
        }
    }
}

impl From<CapError> for GatedError {
    fn from(e: CapError) -> Self { GatedError::Cap(e) }
}
impl From<ObjError> for GatedError {
    fn from(e: ObjError) -> Self { GatedError::Store(e) }
}

/// Create an object (requires WRITE cap).
pub fn create(cap_id: CapId, obj: Object) -> Result<ObjId, GatedError> {
    caps::manager::verify(cap_id, Rights::WRITE)?;
    Ok(store::create(obj)?)
}

/// Read an object (requires READ cap).
pub fn read(cap_id: CapId, obj_id: ObjId) -> Result<Object, GatedError> {
    caps::manager::verify(cap_id, Rights::READ)?;
    Ok(store::read(obj_id)?)
}

/// Query by tag (requires READ cap).
pub fn query_by_tag(cap_id: CapId, tag: &str) -> Result<Vec<ObjId>, GatedError> {
    caps::manager::verify(cap_id, Rights::READ)?;
    Ok(store::query_by_tag(tag))
}

/// Delete an object (requires DELETE cap).
pub fn delete(cap_id: CapId, obj_id: ObjId) -> Result<(), GatedError> {
    caps::manager::verify(cap_id, Rights::DELETE)?;
    Ok(store::delete(obj_id)?)
}
