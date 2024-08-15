use mcps::{
    schedule::Schedule,
    simulation::run_multiple_simulations,
    task::{days_to_duration, Task},
};

use std::time::Duration;

#[test]
fn test_average_effort_calculation() {
    let num_tasks = 10;
    let min_days = 5.0;
    let max_days = 15.0;
    let expected_avg_days = 10.0;
    let estimate_confidence = 0.8;
    let num_simulations = 10000;
    let num_workers = 1; // Using 1 worker to simplify effort calculation

    // Create tasks
    let tasks: Vec<Task> = (0..num_tasks)
        .map(|i| {
            Task::new(
                &format!("Task{}", i),
                vec![],
                days_to_duration(min_days),
                days_to_duration(max_days),
            )
        })
        .collect();

    let schedule =
        Schedule::new(tasks, num_workers, estimate_confidence).expect("Failed to create schedule");

    let (_, effort_times) = run_multiple_simulations(&schedule, num_simulations);

    let total_effort: Duration = effort_times.iter().sum();
    let avg_effort = total_effort / num_simulations as u32;
    let avg_effort_days = avg_effort.as_secs_f64() / 86400.0;

    println!(
        "Average effort: {:.2} days (expected: {:.2} days)",
        avg_effort_days,
        expected_avg_days * num_tasks as f64
    );

    // Check if the average effort is within 1% of the expected value
    let expected_total_days = expected_avg_days * num_tasks as f64;
    let difference_percentage =
        (avg_effort_days - expected_total_days).abs() / expected_total_days * 100.0;

    assert!(
        difference_percentage < 1.0,
        "Average effort ({:.2} days) deviates more than 1% from expected ({:.2} days)",
        avg_effort_days,
        expected_total_days
    );
}
