use std::sync::{Mutex, Arc};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use std::time::Instant;

use std::collections::VecDeque;

use ansi_term::Colour;

use indicatif::ProgressBar;

use crate::task::Task;

pub struct SchedulerFactory;

/*
Global Earliest Deadline
Completely Fair Scheduler
Global Queue Scheduler
Dynamic Priority Paritioned Scheduler
MultiLevel Queue Scheduler
Work Stealing Scheduler


*/

impl SchedulerFactory {
	pub fn create_scheduler(algorithm: &str, core_count: usize) -> Arc<Mutex<dyn SchedulingStrategy>> {
        match algorithm {
            "GlobalEDF" => Arc::new(Mutex::new(GlobalEDFScheduler::new())),
            "CFS" => Arc::new(Mutex::new(CFS::new())),
            "GlobalQueue" => Arc::new(Mutex::new(GlobalQueueScheduler::new())),
            "DynamicPriorityPartitioned" => Arc::new(Mutex::new(DynamicPriorityPartitionedScheduler::new(core_count))),
            "MultiLevelQueue" => Arc::new(Mutex::new(MultiLevelQueueScheduler::new())),
            "WorkStealing" => Arc::new(Mutex::new(WorkStealingScheduler::new(core_count))),
            _ => panic!("I don't know: {}", algorithm),
        }
    }

}


pub trait SchedulingStrategy: Send + Sync {
	fn schedule(&mut self, tasks: &mut Vec<Task>) ->Option<Task>;
	fn add_task(&mut self, task: Task); 
}

pub struct FixedPriorityScheduler {
	pub tasks: Vec<Task>,
}

pub struct GlobalEDFScheduler {
	tasks: Arc<Mutex<Vec<Task>>>,
}

impl GlobalEDFScheduler {
	pub fn new() -> Self {
		GlobalEDFScheduler {
			tasks: Arc::new(Mutex::new(Vec::new())), 
		}
	}

	pub fn add_task(&self, task: Task) {
		let mut tasks = self.tasks.lock().unwrap();
		tasks.push(task);
		tasks.sort_by_key(|task| task.deadline);
	}

	pub fn schedule(&self) -> Option<Task> {
        let mut tasks = self.tasks.lock().unwrap();
        if let Some(next_task) = tasks.iter().min_by_key(|task| task.deadline) {
            let index = tasks.iter().position(|task| task.name == next_task.name).unwrap();
            Some(tasks.remove(index))
        } else {
            None
        }
    }
}

impl SchedulingStrategy for GlobalEDFScheduler {
    fn schedule(&mut self, tasks: &mut Vec<Task>) -> Option<Task> {
        tasks.sort_by_key(|task| task.deadline);
        tasks.pop()
    }

    fn add_task(&mut self, task: Task) {
        self.tasks.lock().unwrap().push(task);
    }
}

pub struct CFS {
	tasks: Arc<Mutex<Vec<Task>>>,
}

impl CFS {
    pub fn new() -> Self {
        CFS {
            tasks: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn add_task(&self, task: Task) {
        let mut tasks = self.tasks.lock().unwrap();
        tasks.push(task);
    }

    pub fn internal_schedule(&self) -> Option<Task> {
        let mut tasks = self.tasks.lock().unwrap();

        if tasks.is_empty() {
            return None;
        }

        let next_task_index = tasks.iter().enumerate().min_by_key(|&(_, task)| task.executed_time).map(|(index, _)| index);

        next_task_index.map(|index| tasks.remove(index))
    }
}

impl SchedulingStrategy for CFS {
    fn schedule(&mut self, tasks: &mut Vec<Task>) -> Option<Task> {
        self.internal_schedule()
    }

    fn add_task(&mut self, task: Task) {
        self.tasks.lock().unwrap().push(task);
    }
}

pub struct GlobalQueueScheduler {
    tasks: Arc<Mutex<Vec<Task>>>,
}

impl GlobalQueueScheduler {
    pub fn new() -> Self {
        GlobalQueueScheduler {
            tasks: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn add_task(&self, task: Task) {
        let mut tasks = self.tasks.lock().unwrap();
        tasks.push(task);
    }

    pub fn schedule(&self) -> Option<Task> {
        let mut tasks = self.tasks.lock().unwrap();
        tasks.pop()
    }
}

impl SchedulingStrategy for GlobalQueueScheduler {
    fn schedule(&mut self, tasks: &mut Vec<Task>) -> Option<Task> {
    	let mut tasks = self.tasks.lock().unwrap();
        tasks.pop()
    }

    fn add_task(&mut self, task: Task) {
    	let mut tasks = self.tasks.lock().unwrap();
        tasks.push(task);
    }
}

pub struct DynamicPriorityPartitionedScheduler {
	queues: Vec<Arc<Mutex<Vec<Task>>>>,
}

impl DynamicPriorityPartitionedScheduler {
	pub fn new(core_count: usize) -> Self {
		let queues = (0..core_count)
			.map(|_| Arc::new(Mutex::new(Vec::new())))
			.collect();

		DynamicPriorityPartitionedScheduler { queues }
	}

	pub fn add_task(&mut self, task: Task, core_id: usize) {
		let queue = &self.queues[core_id];
		queue.lock().unwrap().push(task);
	}

	pub fn schedule(&self, core_id: usize) -> Option<Task> {
		let queue = &self.queues[core_id];
		let mut tasks = queue.lock().unwrap();

		tasks.sort_by(|a, b| (b.priority as u32).cmp(&a.dynamic_priority));

		tasks.pop()
	}
}

impl SchedulingStrategy for DynamicPriorityPartitionedScheduler {
	fn schedule(&mut self, tasks: &mut Vec<Task>) -> Option<Task> {
        tasks.sort(); 
        tasks.pop()
    }

    fn add_task(&mut self, task: Task) {
        let queue = &self.queues[task.core_id];
        let mut queue_tasks = queue.lock().unwrap();
        queue_tasks.push(task);
    }
}

pub struct MultiLevelQueueScheduler {
    high_priority_queue: VecDeque<Task>,
    low_priority_queue: VecDeque<Task>,
}

impl MultiLevelQueueScheduler {
    pub fn new() -> Self {
        MultiLevelQueueScheduler {
            high_priority_queue: VecDeque::new(),
            low_priority_queue: VecDeque::new(),
        }
    }

    pub fn add_task(&mut self, task: Task) {
        if task.priority > 5 {
            self.high_priority_queue.push_back(task);
        } else {
            self.low_priority_queue.push_back(task);
        }
    }

    pub fn schedule(&mut self) -> Option<Task> {
        self.high_priority_queue.pop_front()
            .or_else(|| self.low_priority_queue.pop_front())
    }
}

impl SchedulingStrategy for MultiLevelQueueScheduler {
    fn schedule(&mut self, tasks: &mut Vec<Task>) -> Option<Task> {
        self.high_priority_queue.pop_front()
            .or_else(|| self.low_priority_queue.pop_front())
    }

    fn add_task(&mut self, task: Task) {
        if task.priority > 5 {
            self.high_priority_queue.push_back(task);
        } else {
            self.low_priority_queue.push_back(task);
        }
    }
}


pub struct WorkStealingScheduler {
    core_queues: Vec<Mutex<VecDeque<Task>>>,
    current_core_id: usize,
}

impl WorkStealingScheduler {
    pub fn new(core_count: usize) -> Self {
        let core_queues = (0..core_count)
            .map(|_| Mutex::new(VecDeque::new()))
            .collect();

        WorkStealingScheduler { 
        	core_queues,
        	current_core_id: 0,

        }
    }

    pub fn add_task(&self, task: Task) {
        let core_id = task.core_id % self.core_queues.len();
        self.core_queues[core_id].lock().unwrap().push_back(task);
    }

    pub fn schedule(&self, core_id: usize) -> Option<Task> {
        let queue = &self.core_queues[core_id];
        let mut queue = queue.lock().unwrap();
        queue.pop_front().or_else(|| {
            for other_queue in self.core_queues.iter() {
                if let Ok(mut other_queue) = other_queue.try_lock() {
                    if let Some(task) = other_queue.pop_front() {
                        return Some(task);
                    }
                }
            }
            None
        })
    }
}

impl SchedulingStrategy for WorkStealingScheduler {
    fn schedule(&mut self, tasks: &mut Vec<Task>) -> Option<Task> {
        let core_id = self.current_core_id;

        let task = self.core_queues[core_id].lock().unwrap().pop_front();

        if task.is_none() {
            for other_queue in self.core_queues.iter().enumerate().filter(|&(id, _)| id != core_id) {
                let (_, queue) = other_queue;
                if let Ok(mut other_queue) = queue.try_lock() {
                    if let Some(stolen_task) = other_queue.pop_front() {
                        return Some(stolen_task);
                    }
                }
            }
        }

        task  
    }

    fn add_task(&mut self, task: Task) {
        let core_id = task.core_id % self.core_queues.len();
        self.core_queues[core_id].lock().unwrap().push_back(task);
    }
}


pub struct Core {
	pub id: usize,
	pub scheduler: Arc<Mutex<dyn SchedulingStrategy>>,
	pub progress_bar: ProgressBar,
}


impl Core {
    pub fn run(&self) -> JoinHandle<()> {
        let id = self.id;
        let scheduler_clone = self.scheduler.clone();
        let pb = self.progress_bar.clone();

        thread::spawn(move || {
            let mut task_completion_times = Vec::new();

            while let Some(task) = scheduler_clone.lock().unwrap().schedule(&mut vec![]) {
                let start_time = Instant::now();
                pb.set_message(format!("Core {}: Executing Task {}", id, task.name));

                for _ in 0..task.wcet * 10 {
                    thread::sleep(Duration::from_millis(100)); 
                    pb.inc(1);

                }

                pb.println(format!("Core {}: Task {} completed", id, task.name));

                let end_time = Instant::now();

                task_completion_times.push((task.name, start_time, end_time));
            }

            for (task_name, start, end) in task_completion_times {
                println!("Task {} completed by Core {} in {:?}", task_name, id, end.duration_since(start));
            }
                      
            pb.finish_with_message(format!("Core {} all tasks done!", id));
        })
    }
}
