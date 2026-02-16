//! Capability Manager — the kernel's authority for minting,
//! verifying, restricting, and revoking capabilities.

use alloc::collections::BTreeMap;
use spin::Mutex;
use super::{CapId, Capability, CapError, Resource, Rights};

/// Global capability manager instance.
static MANAGER: Mutex<CapManagerInner> = Mutex::new(CapManagerInner::new());

struct CapManagerInner {
    caps: Option<BTreeMap<CapId, Capability>>,
}

impl CapManagerInner {
    const fn new() -> Self {
        // BTreeMap can't be const-constructed, so we use Option
        Self { caps: None }
    }

    fn caps(&mut self) -> &mut BTreeMap<CapId, Capability> {
        self.caps.get_or_insert_with(BTreeMap::new)
    }
}

/// Mint a new capability (kernel-only operation).
pub fn mint(resource: Resource, rights: Rights, delegatable: bool) -> CapId {
    let id = CapId::mint();
    let cap = Capability {
        id,
        resource,
        rights,
        delegatable,
        revoked: false,
    };
    MANAGER.lock().caps().insert(id, cap);
    id
}

/// Verify that a capability grants the required rights.
pub fn verify(cap_id: CapId, required: Rights) -> Result<(), CapError> {
    let mgr = MANAGER.lock();
    let caps = mgr.caps.as_ref().ok_or(CapError::NotFound)?;
    let cap = caps.get(&cap_id).ok_or(CapError::NotFound)?;

    if cap.revoked {
        return Err(CapError::Revoked);
    }
    if !cap.rights.contains(required) {
        return Err(CapError::PermissionDenied);
    }
    Ok(())
}

/// Create a restricted child capability with ≤ rights.
pub fn restrict(parent_id: CapId, new_rights: Rights) -> Result<CapId, CapError> {
    let mut mgr = MANAGER.lock();
    let caps = mgr.caps.as_ref().ok_or(CapError::NotFound)?;
    let parent = caps.get(&parent_id).ok_or(CapError::NotFound)?;

    if parent.revoked {
        return Err(CapError::Revoked);
    }
    if !parent.delegatable {
        return Err(CapError::NotDelegatable);
    }
    // Cannot escalate: new rights must be subset of parent rights
    if !parent.rights.contains(new_rights) {
        return Err(CapError::CannotEscalate);
    }

    let child_id = CapId::mint();
    let child = Capability {
        id: child_id,
        resource: parent.resource.clone(),
        rights: new_rights,
        delegatable: parent.delegatable,
        revoked: false,
    };

    // Need mutable access to insert
    mgr.caps().insert(child_id, child);
    Ok(child_id)
}

/// Revoke a capability (marks it invalid, O(1)).
pub fn revoke(cap_id: CapId) -> Result<(), CapError> {
    let mut mgr = MANAGER.lock();
    let caps = mgr.caps.as_mut().ok_or(CapError::NotFound)?;
    let cap = caps.get_mut(&cap_id).ok_or(CapError::NotFound)?;
    cap.revoked = true;
    Ok(())
}

/// Get a description of a capability (for logging).
pub fn describe(cap_id: CapId) -> Result<(Resource, Rights), CapError> {
    let mgr = MANAGER.lock();
    let caps = mgr.caps.as_ref().ok_or(CapError::NotFound)?;
    let cap = caps.get(&cap_id).ok_or(CapError::NotFound)?;
    Ok((cap.resource.clone(), cap.rights))
}
