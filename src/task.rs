use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Task {
    pub id: String,
    pub dependencies: Vec<String>,
    pub min_time: Duration,
    pub max_time: Duration,
}

impl Task {
    pub fn new(
        id: &str,
        dependencies: Vec<String>,
        min_time: Duration,
        max_time: Duration,
    ) -> Self {
        Task {
            id: id.to_string(),
            dependencies,
            min_time,
            max_time,
        }
    }
}

pub fn days_to_duration(days: f64) -> Duration {
    Duration::from_secs_f64(days * 24.0 * 60.0 * 60.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f64 = 1e-6;

    fn assert_duration_eq(actual: Duration, expected: Duration) {
        let actual_secs = actual.as_secs_f64();
        let expected_secs = expected.as_secs_f64();
        assert!(
            (actual_secs - expected_secs).abs() < EPSILON,
            "Durations are not equal: actual = {:?}, expected = {:?}",
            actual,
            expected
        );
    }

    #[test]
    fn test_task_creation() {
        let task = Task::new(
            "Task1",
            vec!["Dep1".to_string(), "Dep2".to_string()],
            Duration::from_secs(1),
            Duration::from_secs(5),
        );

        assert_eq!(task.id, "Task1");
        assert_eq!(task.dependencies, vec!["Dep1", "Dep2"]);
        assert_eq!(task.min_time, Duration::from_secs(1));
        assert_eq!(task.max_time, Duration::from_secs(5));
    }

    #[test]
    fn test_task_creation_empty_dependencies() {
        let task = Task::new(
            "Task2",
            vec![],
            Duration::from_secs(2),
            Duration::from_secs(10),
        );

        assert_eq!(task.id, "Task2");
        assert!(task.dependencies.is_empty());
        assert_eq!(task.min_time, Duration::from_secs(2));
        assert_eq!(task.max_time, Duration::from_secs(10));
    }

    #[test]
    fn test_task_creation_same_min_max_time() {
        let task = Task::new(
            "Task3",
            vec!["Dep3".to_string()],
            Duration::from_secs(5),
            Duration::from_secs(5),
        );

        assert_eq!(task.id, "Task3");
        assert_eq!(task.dependencies, vec!["Dep3"]);
        assert_eq!(task.min_time, task.max_time);
    }

    #[test]
    fn test_task_clone() {
        let task1 = Task::new(
            "Task4",
            vec!["Dep4".to_string()],
            Duration::from_secs(3),
            Duration::from_secs(7),
        );
        let task2 = task1.clone();

        assert_eq!(task1.id, task2.id);
        assert_eq!(task1.dependencies, task2.dependencies);
        assert_eq!(task1.min_time, task2.min_time);
        assert_eq!(task1.max_time, task2.max_time);
    }

    #[test]
    fn test_days_to_duration() {
        assert_duration_eq(days_to_duration(1.0), Duration::from_secs(24 * 60 * 60));
        assert_duration_eq(days_to_duration(7.0), Duration::from_secs(7 * 24 * 60 * 60));
        assert_duration_eq(days_to_duration(0.0), Duration::from_secs(0));
        assert_duration_eq(
            days_to_duration(365.0),
            Duration::from_secs(365 * 24 * 60 * 60),
        );
    }

    #[test]
    fn test_days_to_duration_fractional() {
        assert_duration_eq(days_to_duration(0.5), Duration::from_secs(12 * 60 * 60));
        assert_duration_eq(days_to_duration(1.5), Duration::from_secs(36 * 60 * 60));
        assert_duration_eq(days_to_duration(0.25), Duration::from_secs(6 * 60 * 60));
        assert_duration_eq(
            days_to_duration(0.1),
            Duration::from_secs_f64(0.1 * 24.0 * 60.0 * 60.0),
        );
    }

    #[test]
    fn test_days_to_duration_large_number() {
        let large_days = 1000.0;
        let expected_seconds = large_days * 24.0 * 60.0 * 60.0;
        assert_duration_eq(
            days_to_duration(large_days),
            Duration::from_secs_f64(expected_seconds),
        );
    }

    #[test]
    fn test_days_to_duration_very_small() {
        let small_days = 1e-6;
        let expected_seconds = small_days * 24.0 * 60.0 * 60.0;
        assert_duration_eq(
            days_to_duration(small_days),
            Duration::from_secs_f64(expected_seconds),
        );
    }
}
