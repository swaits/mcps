use crate::task::Task;

use std::collections::{HashMap, HashSet};
use std::time::Duration;

#[derive(Debug)]
pub struct Schedule {
    pub tasks: Vec<Task>,
    pub num_workers: usize,
    pub estimate_confidence: f64,
}

impl Schedule {
    pub fn new(
        tasks: Vec<Task>,
        num_workers: usize,
        estimate_confidence: f64,
    ) -> Result<Self, String> {
        let schedule = Schedule {
            tasks,
            num_workers,
            estimate_confidence,
        };
        schedule.validate()?;
        Ok(schedule)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.tasks.is_empty() {
            return Err("Empty task list".to_string());
        }
        if self.num_workers == 0 {
            return Err("Invalid number of workers (must be 1 or more)".to_string());
        }
        let all_task_ids: HashSet<&String> = self.tasks.iter().map(|t| &t.id).collect();
        for task in &self.tasks {
            if task.min_time > task.max_time {
                return Err(format!(
                    "Minimum duration greater than maximum for task {}",
                    task.id
                ));
            }
            if task.min_time <= Duration::from_secs(0) || task.max_time <= Duration::from_secs(0) {
                return Err(format!("Invalid task duration for task {}", task.id));
            }
            for dep in &task.dependencies {
                if !all_task_ids.contains(dep) {
                    return Err(format!("Missing dependency {} for task {}", dep, task.id));
                }
            }
        }
        self.check_cyclic_dependencies()
    }

    fn check_cyclic_dependencies(&self) -> Result<(), String> {
        let mut visited = HashSet::new();
        let mut stack = HashSet::new();
        let task_map: HashMap<_, _> = self.tasks.iter().map(|t| (&t.id, t)).collect();

        for task in &self.tasks {
            Self::dfs(&task.id, &task_map, &mut visited, &mut stack)?;
        }
        Ok(())
    }

    fn dfs<'a>(
        task_id: &'a str,
        task_map: &'a HashMap<&'a String, &'a Task>,
        visited: &mut HashSet<&'a str>,
        stack: &mut HashSet<&'a str>,
    ) -> Result<(), String> {
        if stack.contains(task_id) {
            return Err(format!(
                "Cyclic dependency detected involving task {}",
                task_id
            ));
        }
        if visited.contains(task_id) {
            return Ok(());
        }
        visited.insert(task_id);
        stack.insert(task_id);
        if let Some(task) = task_map.get(&task_id.to_string()) {
            for dep in &task.dependencies {
                Self::dfs(dep, task_map, visited, stack)?;
            }
        }
        stack.remove(task_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_task(id: &str, min: u64, max: u64, deps: Vec<&str>) -> Task {
        Task {
            id: id.to_string(),
            dependencies: deps.into_iter().map(String::from).collect(),
            min_time: Duration::from_secs(min),
            max_time: Duration::from_secs(max),
        }
    }

    #[test]
    fn test_valid_schedule() {
        let tasks = vec![
            create_task("A", 1, 3, vec![]),
            create_task("B", 2, 4, vec!["A"]),
            create_task("C", 3, 5, vec!["A"]),
        ];
        let schedule = Schedule::new(tasks, 2, 0.8);
        assert!(schedule.is_ok());
    }

    #[test]
    fn test_empty_task_list() {
        let schedule = Schedule::new(vec![], 2, 0.8);
        assert!(schedule.is_err());
        assert_eq!(schedule.unwrap_err(), "Empty task list");
    }

    #[test]
    fn test_invalid_task_duration() {
        let tasks = vec![create_task("A", 3, 1, vec![])];
        let schedule = Schedule::new(tasks, 2, 0.8);
        assert!(schedule.is_err());
        assert_eq!(
            schedule.unwrap_err(),
            "Minimum duration greater than maximum for task A"
        );
    }

    #[test]
    fn test_zero_duration() {
        let tasks = vec![create_task("A", 0, 1, vec![])];
        let schedule = Schedule::new(tasks, 2, 0.8);
        assert!(schedule.is_err());
        assert_eq!(schedule.unwrap_err(), "Invalid task duration for task A");
    }

    #[test]
    fn test_missing_dependency() {
        let tasks = vec![
            create_task("A", 1, 3, vec![]),
            create_task("B", 2, 4, vec!["C"]),
        ];
        let schedule = Schedule::new(tasks, 2, 0.8);
        assert!(schedule.is_err());
        assert_eq!(schedule.unwrap_err(), "Missing dependency C for task B");
    }

    #[test]
    fn test_cyclic_dependency() {
        let tasks = vec![
            create_task("A", 1, 3, vec!["B"]),
            create_task("B", 2, 4, vec!["A"]),
        ];
        let schedule = Schedule::new(tasks, 2, 0.8);
        assert!(schedule.is_err());
        assert!(schedule
            .unwrap_err()
            .starts_with("Cyclic dependency detected involving task"));
    }

    #[test]
    fn test_complex_valid_schedule() {
        let tasks = vec![
            create_task("A", 1, 3, vec![]),
            create_task("B", 2, 4, vec!["A"]),
            create_task("C", 3, 5, vec!["A"]),
            create_task("D", 2, 6, vec!["B", "C"]),
            create_task("E", 1, 2, vec!["A"]),
            create_task("F", 2, 4, vec!["D", "E"]),
        ];
        let schedule = Schedule::new(tasks, 3, 0.9);
        assert!(schedule.is_ok());
    }

    #[test]
    fn test_complex_cyclic_dependency() {
        let tasks = vec![
            create_task("A", 1, 3, vec!["F"]),
            create_task("B", 2, 4, vec!["A"]),
            create_task("C", 3, 5, vec!["B"]),
            create_task("D", 2, 6, vec!["C"]),
            create_task("E", 1, 2, vec!["D"]),
            create_task("F", 2, 4, vec!["E"]),
        ];
        let schedule = Schedule::new(tasks, 3, 0.9);
        assert!(
            schedule.is_err(),
            "Expected cyclic dependency error, but got Ok"
        );
        if let Err(err) = schedule {
            assert!(
                err.starts_with("Cyclic dependency detected"),
                "Unexpected error message: {}",
                err
            );
        }
    }

    #[test]
    fn test_self_dependency() {
        let tasks = vec![create_task("A", 1, 3, vec!["A"])];
        let schedule = Schedule::new(tasks, 1, 0.8);
        assert!(
            schedule.is_err(),
            "Expected self-dependency error, but got Ok"
        );
        if let Err(err) = schedule {
            assert!(
                err.starts_with("Cyclic dependency detected"),
                "Unexpected error message: {}",
                err
            );
        }
    }
}
