use mcps::{
    schedule::Schedule,
    simulation::run_multiple_simulations,
    task::{days_to_duration, Task},
};

use std::time::Duration;

fn create_independent_tasks(num_tasks: usize) -> Vec<Task> {
    (0..num_tasks)
        .map(|i| {
            Task::new(
                &format!("Task_{}", i),
                vec![],
                days_to_duration(1.0),
                days_to_duration(3.0),
            )
        })
        .collect()
}

#[test]
fn test_no_dependencies_schedule() {
    let num_tasks = 32;
    let tasks = create_independent_tasks(num_tasks);
    let estimate_confidence = 0.8;
    let num_simulations = 1_000;

    let mut prev_avg_effort = 0.0;

    for num_workers in [1, 2, 4, 8] {
        let schedule = Schedule::new(tasks.clone(), num_workers, estimate_confidence)
            .expect("Failed to create schedule");
        let (project_durations, effort_times) =
            run_multiple_simulations(&schedule, num_simulations);

        let avg_duration = project_durations.iter().sum::<Duration>() / num_simulations as u32;
        let avg_effort = effort_times.iter().sum::<Duration>() / num_simulations as u32;

        let avg_duration_days = avg_duration.as_secs_f64() / 86400.0;
        let avg_effort_days = avg_effort.as_secs_f64() / 86400.0;

        println!(
            "Workers: {}, Avg Duration: {:.2} days, Avg Effort: {:.2} person-days",
            num_workers, avg_duration_days, avg_effort_days
        );

        // Check that effort is consistent across different numbers of workers
        if num_workers > 1 {
            let effort_difference = (avg_effort_days - prev_avg_effort).abs();
            assert!(
                effort_difference < 2.0,
                "Effort should be consistent. Previous: {:.2}, Current: {:.2}",
                prev_avg_effort,
                avg_effort_days
            );
        }
        prev_avg_effort = avg_effort_days;

        // Check effort/duration ratio
        let effort_ratio = avg_effort_days / avg_duration_days;
        let expected_ratio = num_workers as f64;
        let ratio_error = (effort_ratio - expected_ratio).abs() / expected_ratio;

        println!(
            "  Effort/Duration Ratio: {:.2} (Expected: {:.2}, Error: {:.2})",
            effort_ratio, expected_ratio, ratio_error
        );

        assert!(ratio_error < 0.20, "Expected effort/duration ratio to be close to number of workers. Expected: {}, Actual: {}", expected_ratio, effort_ratio);

        // For 1 worker, duration should be very close to effort
        if num_workers == 1 {
            let duration_effort_ratio = avg_duration_days / avg_effort_days;
            assert!(
                (duration_effort_ratio - 1.0).abs() < 0.05,
                "For single worker, duration should equal effort. Ratio: {}",
                duration_effort_ratio
            );
        }

        // Check that duration decreases proportionally to the increase in workers
        if num_workers > 1 {
            let expected_duration = avg_effort_days / num_workers as f64;
            let actual_duration = avg_duration_days;
            let duration_ratio = actual_duration / expected_duration;
            let duration_error = (duration_ratio - 1.0).abs();

            println!(
                "  Duration Ratio: {:.2} (Expected: {:.2}, Actual: {:.2}, Error: {:.2})",
                duration_ratio, expected_duration, actual_duration, duration_error
            );

            assert!(duration_error < 0.20, "Expected duration to decrease proportionally with workers. Expected: {:.2}, Actual: {:.2}", expected_duration, actual_duration);
        }
    }
}
