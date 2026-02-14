//! Round-robin cooperative scheduler.
//!
//! Each task has a step function that gets called once per scheduler turn.
//! The scheduler interleaves tasks in round-robin order, giving each task
//! exactly one step per turn. Timer interrupts decrement a fuel counter
//! that can be used for time-based preemption in the future.

use alloc::collections::VecDeque;
use core::sync::atomic::{AtomicU64, Ordering};
use super::{Task, TaskState};
use crate::println;

/// Default fuel (timer ticks) per task slice.
/// At ~18.2 Hz, 18 ticks ≈ 1 second per task.
pub const DEFAULT_FUEL: u64 = 18;

/// Global fuel counter — decremented by timer interrupt.
static FUEL_REMAINING: AtomicU64 = AtomicU64::new(DEFAULT_FUEL);

/// Called from timer interrupt handler — decrement fuel.
pub fn timer_tick() {
    let remaining = FUEL_REMAINING.load(Ordering::Relaxed);
    if remaining > 0 {
        FUEL_REMAINING.fetch_sub(1, Ordering::Relaxed);
    }
}

/// Check if fuel is exhausted (for future preemptive use).
pub fn fuel_exhausted() -> bool {
    FUEL_REMAINING.load(Ordering::Relaxed) == 0
}

/// Reset fuel for the next task slice.
pub fn refuel() {
    FUEL_REMAINING.store(DEFAULT_FUEL, Ordering::Relaxed);
}

/// The cooperative round-robin scheduler.
pub struct Scheduler {
    tasks: VecDeque<Task>,
}

impl Scheduler {
    /// Create a new empty scheduler.
    pub fn new() -> Self {
        Scheduler {
            tasks: VecDeque::new(),
        }
    }

    /// Spawn a new task.
    pub fn spawn(&mut self, name: &'static str, steps: u64, step_fn: fn(u64)) {
        let task = Task::new(name, steps, step_fn);
        println!("[SCHED] Spawned {} ({}, {} steps)", task.name, task.id, steps);
        self.tasks.push_back(task);
    }

    /// Run all tasks in round-robin order until all are done.
    /// Each task gets exactly 1 step per turn, proving interleaving.
    pub fn run(&mut self) {
        println!("[SCHED] Starting scheduler with {} tasks", self.tasks.len());
        println!();

        while !self.tasks.is_empty() {
            if let Some(mut task) = self.tasks.pop_front() {
                task.state = TaskState::Running;
                refuel();

                // Run exactly one step
                if task.current_step < task.total_steps {
                    (task.step_fn)(task.current_step);
                    task.current_step += 1;
                }

                if task.current_step >= task.total_steps {
                    task.state = TaskState::Done;
                    println!("[SCHED] {} completed", task.name);
                } else {
                    task.state = TaskState::Ready;
                    self.tasks.push_back(task);
                }
            }
        }

        println!();
        println!("[SCHED] All tasks completed");
    }
}
