use mcps::{
    schedule::Schedule,
    task::{days_to_duration, Task},
};

fn create_schedule_and_expect_error(tasks: Vec<Task>, expected_error: &str) {
    let result = Schedule::new(tasks, 1, 0.8);

    assert!(
        result.is_err(),
        "Expected schedule creation to fail, but it didn't"
    );
    if let Err(error_message) = result {
        assert!(
            error_message.contains(expected_error),
            "Expected error message '{}', but got '{}'",
            expected_error,
            error_message
        );
    }
}

#[test]
fn test_cyclic_dependency() {
    let tasks = vec![
        Task::new(
            "A",
            vec!["B".to_string()],
            days_to_duration(1.0),
            days_to_duration(2.0),
        ),
        Task::new(
            "B",
            vec!["A".to_string()],
            days_to_duration(1.0),
            days_to_duration(2.0),
        ),
    ];

    create_schedule_and_expect_error(tasks, "Cyclic dependency detected");
}

#[test]
fn test_missing_dependency() {
    let tasks = vec![
        Task::new(
            "A",
            vec!["C".to_string()],
            days_to_duration(1.0),
            days_to_duration(2.0),
        ),
        Task::new(
            "B",
            vec!["A".to_string()],
            days_to_duration(1.0),
            days_to_duration(2.0),
        ),
    ];

    create_schedule_and_expect_error(tasks, "Missing dependency");
}

#[test]
fn test_negative_duration() {
    let tasks = vec![Task::new(
        "A",
        vec![],
        days_to_duration(0.0),
        days_to_duration(1.0),
    )];

    create_schedule_and_expect_error(tasks, "Invalid task duration");
}

#[test]
fn test_min_greater_than_max() {
    let tasks = vec![Task::new(
        "A",
        vec![],
        days_to_duration(3.0),
        days_to_duration(2.0),
    )];

    create_schedule_and_expect_error(tasks, "Minimum duration greater than maximum");
}

#[test]
fn test_empty_task_list() {
    let tasks = vec![];
    create_schedule_and_expect_error(tasks, "Empty task list");
}
