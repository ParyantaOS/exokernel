//! Capability-based security for the exokernel.
//!
//! Zero ambient authority: tasks start with no rights and must be
//! explicitly granted capabilities to access any resource.

pub mod manager;

use core::sync::atomic::{AtomicU64, Ordering};

// ─── Core types ──────────────────────────────────────────────────

/// Unforgeable capability identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CapId(u64);

static NEXT_CAP_ID: AtomicU64 = AtomicU64::new(1);

impl CapId {
    /// Mint a new unique capability ID (kernel-only).
    fn mint() -> Self {
        CapId(NEXT_CAP_ID.fetch_add(1, Ordering::Relaxed))
    }

    /// Get the raw numeric ID (for display).
    pub fn raw(&self) -> u64 {
        self.0
    }
}

impl core::fmt::Display for CapId {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "Cap#{}", self.0)
    }
}

// ─── Rights (bitflags) ──────────────────────────────────────────

bitflags::bitflags! {
    /// Access rights that a capability can grant.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Rights: u32 {
        const READ    = 0b0000_0001;
        const WRITE   = 0b0000_0010;
        const EXECUTE = 0b0000_0100;
        const DELETE  = 0b0000_1000;

        const RW  = Self::READ.bits() | Self::WRITE.bits();
        const ALL = Self::READ.bits() | Self::WRITE.bits()
                  | Self::EXECUTE.bits() | Self::DELETE.bits();
    }
}

impl core::fmt::Display for Rights {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let mut parts = alloc::vec::Vec::new();
        if self.contains(Rights::READ)    { parts.push("R"); }
        if self.contains(Rights::WRITE)   { parts.push("W"); }
        if self.contains(Rights::EXECUTE) { parts.push("X"); }
        if self.contains(Rights::DELETE)  { parts.push("D"); }
        if parts.is_empty() {
            write!(f, "NONE")
        } else {
            write!(f, "{}", parts.join(""))
        }
    }
}

// ─── Resource ───────────────────────────────────────────────────

/// The type of resource a capability grants access to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Resource {
    /// A region of physical memory.
    Memory { base: u64, size: u64 },
    /// A hardware device (by port or MMIO base).
    Device(u32),
    /// A named object (future: Object Store).
    Object(u64),
    /// CPU time slice (in ticks).
    Cpu(u64),
}

impl core::fmt::Display for Resource {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Resource::Memory { base, size } => write!(f, "Memory(0x{:x}+{})", base, size),
            Resource::Device(id) => write!(f, "Device({})", id),
            Resource::Object(id) => write!(f, "Object({})", id),
            Resource::Cpu(ticks) => write!(f, "Cpu({} ticks)", ticks),
        }
    }
}

// ─── Capability ─────────────────────────────────────────────────

/// A capability: an unforgeable token granting specific rights to a resource.
#[derive(Debug, Clone)]
pub struct Capability {
    pub id: CapId,
    pub resource: Resource,
    pub rights: Rights,
    pub delegatable: bool,
    pub revoked: bool,
}

// ─── Errors ─────────────────────────────────────────────────────

/// Capability operation errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapError {
    /// Capability not found in manager.
    NotFound,
    /// Capability has been revoked.
    Revoked,
    /// Rights insufficient for the requested operation.
    PermissionDenied,
    /// Cannot escalate rights beyond what the parent cap grants.
    CannotEscalate,
    /// Capability is not delegatable.
    NotDelegatable,
}

impl core::fmt::Display for CapError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            CapError::NotFound => write!(f, "cap not found"),
            CapError::Revoked => write!(f, "revoked"),
            CapError::PermissionDenied => write!(f, "permission denied"),
            CapError::CannotEscalate => write!(f, "cannot escalate"),
            CapError::NotDelegatable => write!(f, "not delegatable"),
        }
    }
}
