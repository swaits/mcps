use crate::schedule::Schedule;

use rand::prelude::*;
use rand_distr::{Distribution, Normal};
use rayon::prelude::*;

use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};

pub struct SimulationResult {
    pub total_project_duration: Duration,
    pub total_effort_time: Duration,
}

pub fn run_multiple_simulations(
    schedule: &Schedule,
    num_simulations: usize,
) -> (Vec<Duration>, Vec<Duration>) {
    (0..num_simulations)
        .into_par_iter()
        .map(|_| {
            let result = run_simulation(schedule);
            (result.total_project_duration, result.total_effort_time)
        })
        .unzip()
}

fn run_simulation(schedule: &Schedule) -> SimulationResult {
    let mut rng = thread_rng();

    // Simulate task times
    let task_effort_times: HashMap<_, _> = schedule
        .tasks
        .iter()
        .map(|task| {
            let task_time = simulate_task_time(
                &mut rng,
                task.min_time,
                task.max_time,
                schedule.estimate_confidence,
            );
            (&task.id, task_time)
        })
        .collect();

    let total_effort_time: Duration = task_effort_times.values().sum();

    // Pre-compute task dependencies and reverse dependencies
    let mut task_dependencies: HashMap<_, HashSet<_>> = HashMap::new();
    let mut reverse_dependencies: HashMap<_, Vec<_>> = HashMap::new();
    for task in &schedule.tasks {
        task_dependencies.insert(&task.id, task.dependencies.iter().collect());
        for dep in &task.dependencies {
            reverse_dependencies.entry(dep).or_default().push(&task.id);
        }
    }

    // Initialize task queue with tasks that have no dependencies
    let mut task_queue: Vec<_> = schedule
        .tasks
        .iter()
        .filter(|t| t.dependencies.is_empty())
        .map(|t| &t.id)
        .collect();

    let mut current_time = Duration::default();
    let mut completed_tasks = HashSet::new();
    let mut worker_finish_times = vec![Duration::default(); schedule.num_workers];

    while !task_queue.is_empty() || completed_tasks.len() < schedule.tasks.len() {
        // Find all available workers
        let available_workers: Vec<_> = worker_finish_times
            .iter()
            .enumerate()
            .filter(|&(_, &time)| time <= current_time)
            .map(|(i, _)| i)
            .collect();

        if !task_queue.is_empty() && !available_workers.is_empty() {
            for &worker in &available_workers {
                if task_queue.is_empty() {
                    break;
                }
                // Randomly choose the next task to assign
                let task_index = rng.gen_range(0..task_queue.len());
                let task_id = task_queue.swap_remove(task_index);

                let task_duration = task_effort_times[task_id];
                worker_finish_times[worker] = current_time + task_duration;
                completed_tasks.insert(task_id);

                // Add newly available tasks to queue
                if let Some(dependent_tasks) = reverse_dependencies.get(task_id) {
                    for &dep_task in dependent_tasks {
                        if !completed_tasks.contains(dep_task)
                            && !task_queue.contains(&dep_task)
                            && task_dependencies[dep_task]
                                .iter()
                                .all(|dep| completed_tasks.contains(dep))
                        {
                            task_queue.push(dep_task);
                        }
                    }
                }
            }
        }

        // Move time forward to the next event
        current_time = *worker_finish_times.iter().min().unwrap_or(&current_time);
    }

    SimulationResult {
        total_project_duration: *worker_finish_times.iter().max().unwrap_or(&current_time),
        total_effort_time,
    }
}

fn simulate_task_time(
    rng: &mut impl Rng,
    min_time: Duration,
    max_time: Duration,
    estimate_confidence: f64,
) -> Duration {
    let min_secs = min_time.as_secs_f64();
    let max_secs = max_time.as_secs_f64();
    let mean = (min_secs + max_secs) / 2.0;

    let z_score = (1.0 - (1.0 - estimate_confidence) / 2.0).sqrt() * 2.0;
    let std_dev = (max_secs - min_secs) / (2.0 * z_score);

    let normal = Normal::new(mean, std_dev).unwrap();
    let sampled_secs = normal.sample(rng).clamp(min_secs, max_secs);

    Duration::from_secs_f64(sampled_secs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::Task;

    #[test]
    fn test_simulate_task_time() {
        let mut rng = thread_rng();
        let min_time = Duration::from_secs(5);
        let max_time = Duration::from_secs(15);
        let estimate_confidence = 0.8;

        for _ in 0..1000 {
            let time = simulate_task_time(&mut rng, min_time, max_time, estimate_confidence);
            assert!(
                time >= min_time && time <= max_time,
                "Simulated time should be within range"
            );
        }
    }

    #[test]
    fn test_single_task_simulation() {
        let task = Task::new("A", vec![], Duration::from_secs(5), Duration::from_secs(15));
        let schedule = Schedule::new(vec![task], 1, 0.8).unwrap();

        let result = run_simulation(&schedule);

        assert_eq!(
            result.total_project_duration, result.total_effort_time,
            "For a single task, project duration should equal effort time"
        );
        assert!(
            result.total_project_duration >= Duration::from_secs(5)
                && result.total_project_duration <= Duration::from_secs(15),
            "Project duration should be within the task's time range"
        );
    }

    #[test]
    fn test_multiple_independent_tasks() {
        let tasks = vec![
            Task::new("A", vec![], Duration::from_secs(5), Duration::from_secs(10)),
            Task::new("B", vec![], Duration::from_secs(7), Duration::from_secs(12)),
            Task::new("C", vec![], Duration::from_secs(3), Duration::from_secs(8)),
        ];
        let schedule = Schedule::new(tasks, 2, 0.8).unwrap();

        let result = run_simulation(&schedule);

        assert!(
            result.total_project_duration <= result.total_effort_time,
            "Project duration should not exceed total effort time"
        );
        assert!(
            result.total_project_duration >= Duration::from_secs(7),
            "Project duration should be at least the longest minimum task duration"
        );
    }

    #[test]
    fn test_tasks_with_dependencies() {
        let tasks = vec![
            Task::new("A", vec![], Duration::from_secs(5), Duration::from_secs(10)),
            Task::new(
                "B",
                vec!["A".to_string()],
                Duration::from_secs(7),
                Duration::from_secs(12),
            ),
            Task::new(
                "C",
                vec!["A".to_string()],
                Duration::from_secs(3),
                Duration::from_secs(8),
            ),
            Task::new(
                "D",
                vec!["B".to_string(), "C".to_string()],
                Duration::from_secs(4),
                Duration::from_secs(9),
            ),
        ];
        let schedule = Schedule::new(tasks, 2, 0.8).unwrap();

        let result = run_simulation(&schedule);

        assert!(
            result.total_project_duration >= Duration::from_secs(14),
            "Project duration should be at least the minimum critical path duration"
        );
    }

    #[test]
    fn test_multiple_simulations_consistency() {
        let tasks = vec![
            Task::new("A", vec![], Duration::from_secs(5), Duration::from_secs(10)),
            Task::new(
                "B",
                vec!["A".to_string()],
                Duration::from_secs(7),
                Duration::from_secs(12),
            ),
        ];
        let schedule = Schedule::new(tasks, 1, 0.8).unwrap();

        let (durations, efforts) = run_multiple_simulations(&schedule, 1000);

        assert_eq!(durations.len(), 1000, "Should run 1000 simulations");
        assert_eq!(efforts.len(), 1000, "Should run 1000 simulations");

        let avg_duration: Duration = durations.iter().sum::<Duration>() / 1000;
        let avg_effort: Duration = efforts.iter().sum::<Duration>() / 1000;

        assert!(
            avg_duration >= Duration::from_secs(12) && avg_duration <= Duration::from_secs(22),
            "Average duration should be within expected range"
        );
        assert!(
            avg_effort >= Duration::from_secs(12) && avg_effort <= Duration::from_secs(22),
            "Average effort should be within expected range"
        );
    }
}
