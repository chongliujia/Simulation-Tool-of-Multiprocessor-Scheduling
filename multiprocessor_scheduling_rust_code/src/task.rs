use core::time::{Duration};

use std::time::Instant;
use std::cmp::Ordering;
use std::collections::BinaryHeap;


#[derive(Debug, Eq, Clone)]
pub struct Task {
	pub name: String,
	pub wcet: u32,
	pub deadline: Instant,
	pub priority: u8,
	pub executed_time: Duration,
	pub dynamic_priority: u32,
	pub core_id: usize,

}

impl Task {

}

impl PartialEq for Task {
    fn eq(&self, other: &Self) -> bool {
        self.dynamic_priority == other.dynamic_priority
    }
}

impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Task {
    fn cmp(&self, other: &Self) -> Ordering {
        other.dynamic_priority.cmp(&self.dynamic_priority)
            .then_with(|| self.deadline.cmp(&other.deadline))
    }
}

pub fn create_tasks() -> Vec<Task> {
	vec![
		Task {
			name: String::from("Task 1"),
			wcet: 1,
			deadline: Instant::now() + Duration::from_millis(50),
			priority: 2,
			executed_time: Duration::from_secs(1),
			dynamic_priority: 3,
			core_id: 0,
		},
		Task {
			name: String::from("Task 2"),
			wcet: 2,
			deadline: Instant::now() + Duration::from_millis(110),
			priority: 1,
			executed_time: Duration::from_secs(2),
			dynamic_priority: 1,
			core_id: 1,
		},
		Task {
			name: String::from("Task 3"),
			wcet: 3,
			deadline: Instant::now() + Duration::from_millis(30),
			priority: 3,
			executed_time: Duration::from_secs(3),
			dynamic_priority: 5,
			core_id: 2,
		},
		Task {
			name: String::from("Task 4"),
			wcet: 4,
			deadline: Instant::now() + Duration::from_millis(70),
			priority: 4,
			executed_time: Duration::from_secs(4),
			dynamic_priority: 2,
			core_id: 3,
		},
		Task {
            name: String::from("Task 5"),
            wcet: 5,
            deadline: Instant::now() + Duration::from_millis(150),
            priority: 5,
            executed_time: Duration::from_secs(5),
            dynamic_priority: 6,
            core_id: 4,
        },
        Task {
            name: String::from("Task 6"),
            wcet: 6,
            deadline: Instant::now() + Duration::from_millis(180),
            priority: 6,
            executed_time: Duration::from_secs(6),
            dynamic_priority: 7,
            core_id: 5,
        },
        Task {
            name: String::from("Task 7"),
            wcet: 7,
            deadline: Instant::now() + Duration::from_millis(210),
            priority: 7,
            executed_time: Duration::from_secs(7),
            dynamic_priority: 8,
            core_id: 6,
        },
        Task {
            name: String::from("Task 8"),
            wcet: 8,
            deadline: Instant::now() + Duration::from_millis(240),
            priority: 8,
            executed_time: Duration::from_secs(8),
            dynamic_priority: 9,
            core_id: 7,
        },
	]
}