mod task;
mod scheduler;

use crate::scheduler::{Core, SchedulingStrategy, SchedulerFactory};
use crate::scheduler::{GlobalEDFScheduler, CFS, GlobalQueueScheduler, DynamicPriorityPartitionedScheduler, MultiLevelQueueScheduler, WorkStealingScheduler};
use crate::task::{create_tasks, Task};

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

use std::thread;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use std::io::{self, Write};

/*
Global Queue Scheduling
Work Stealing Scheduling
Hierarchical Scheduling
Dynamic Load Balancing
Gang Scheduling

*/

fn main() {
    println!("1: Global Earliest Deadline First Scheduler");
    println!("2: Completely Fair Scheduler");
    println!("3: Global Queue Scheduler");
    println!("4: Dynamic Priority Partitioned Scheduler");
    println!("5: Multi-Level Queue Scheduler");
    println!("6: Work-Stealing Scheduler");

    print!("Plz choose (1, 2, ...): ");
    io::stdout().flush().unwrap();
    let mut choice = String::new();
    io::stdin().read_line(&mut choice).unwrap();

    let multi_progress = MultiProgress::new();
    let bar_style = ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos:>7}/{len:7} {msg}")
        .progress_chars("#>-");

    let algorithm = match choice.trim() {
        "1" => "GlobalEDF",
        "2" => "CFS",
        "3" => "GlobalQueue",
        "4" => "DynamicPriorityPartitioned",
        "5" => "MultiLevelQueue",
        "6" => "WorkStealing",
        _ => "",
    };

    let core_count = 8;
    let all_tasks = create_tasks();

    let mut handles = Vec::new();

    for i in 0..core_count {
        let scheduler = SchedulerFactory::create_scheduler(algorithm, core_count);
        let pb = multi_progress.add(ProgressBar::new(100));
        pb.set_style(bar_style.clone());
        pb.set_message(format!("Core {}", i));

        let core = Core {
            id: i,
            scheduler: scheduler.clone(),
            progress_bar: pb,
        };

        if let Some(tasks) = assign_tasks_to_core(i, &create_tasks(), core_count) {
            for task in tasks {
                core.scheduler.lock().unwrap().add_task(task);
            }
        }

        handles.push(core.run());
    }

    let progress_thread = thread::spawn(move || {
        multi_progress.join().expect("MultiProgress join failed");
    });



    for handle in handles {
        handle.join().expect("Core thread panicked");
    }

    progress_thread.join().expect("Progress thread panicked");
} 

fn assign_tasks_to_core(core_id: usize, tasks: &[Task], core_count: usize) -> Option<Vec<Task>> {
    let total_tasks = tasks.len();
    let tasks_per_core = total_tasks / core_count;
    let start = core_id * tasks_per_core;
    let end = if core_id == core_count - 1 {
        total_tasks
    } else {
        start + tasks_per_core
    };

    if start < end {
        Some(tasks[start..end].to_vec())
    } else {
        None
    }
}