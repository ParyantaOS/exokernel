//! Cooperative task scheduler for the exokernel.
//!
//! Tasks are lightweight units of execution. Each task has a "step"
//! function that gets called repeatedly. The scheduler gives each
//! task a fuel budget (timer ticks) and switches to the next task
//! when fuel runs out.
//!
//! Tasks hold capabilities — they start with zero and must be
//! explicitly granted access.

pub mod scheduler;

use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};
use crate::caps::CapId;

/// Unique task identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TaskId(u64);

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

impl TaskId {
    fn new() -> Self {
        TaskId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

impl core::fmt::Display for TaskId {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "Task#{}", self.0)
    }
}

/// Task state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    Ready,
    Running,
    Done,
}

/// A schedulable task.
pub struct Task {
    pub id: TaskId,
    pub name: &'static str,
    pub state: TaskState,
    pub current_step: u64,
    pub total_steps: u64,
    pub step_fn: fn(u64, &[CapId]),  // Called with (step_index, caps)
    pub caps: Vec<CapId>,            // Capabilities held by this task
}

impl Task {
    /// Create a new task with the given name, steps, function, and capabilities.
    pub fn new(name: &'static str, total_steps: u64, step_fn: fn(u64, &[CapId]), caps: Vec<CapId>) -> Self {
        Task {
            id: TaskId::new(),
            name,
            state: TaskState::Ready,
            current_step: 0,
            total_steps,
            step_fn,
            caps,
        }
    }
}
