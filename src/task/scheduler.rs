//! Round-robin cooperative scheduler.
//!
//! Each task has a step function that gets called once per scheduler turn.
//! Tasks hold capabilities that are passed to the step function.

use alloc::collections::VecDeque;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};
use super::{Task, TaskState};
use crate::caps::CapId;
use crate::println;

/// Default fuel (timer ticks) per task slice.
pub const DEFAULT_FUEL: u64 = 18;

/// Global fuel counter.
static FUEL_REMAINING: AtomicU64 = AtomicU64::new(DEFAULT_FUEL);

/// Called from timer interrupt handler.
pub fn timer_tick() {
    let remaining = FUEL_REMAINING.load(Ordering::Relaxed);
    if remaining > 0 {
        FUEL_REMAINING.fetch_sub(1, Ordering::Relaxed);
    }
}

/// Check if fuel is exhausted.
pub fn fuel_exhausted() -> bool {
    FUEL_REMAINING.load(Ordering::Relaxed) == 0
}

/// Reset fuel.
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

    /// Spawn a new task with capabilities.
    pub fn spawn(&mut self, name: &'static str, steps: u64, caps: Vec<CapId>, step_fn: fn(u64, &[CapId])) {
        let task = Task::new(name, steps, step_fn, caps);
        println!("[SCHED] Spawned {} ({}, {} steps)", task.name, task.id, steps);
        self.tasks.push_back(task);
    }

    /// Run all tasks in round-robin order until all are done.
    pub fn run(&mut self) {
        println!("[SCHED] Starting scheduler with {} tasks", self.tasks.len());
        println!();

        while !self.tasks.is_empty() {
            if let Some(mut task) = self.tasks.pop_front() {
                task.state = TaskState::Running;
                refuel();

                // Run exactly one step, passing the task's capabilities
                if task.current_step < task.total_steps {
                    (task.step_fn)(task.current_step, &task.caps);
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
