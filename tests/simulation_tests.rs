use mcps::{
    schedule::Schedule,
    simulation::run_multiple_simulations,
    task::{days_to_duration, Task},
};

use std::time::Duration;

#[test]
fn test_simulation_with_different_worker_counts() {
    let tasks = vec![
        Task::new("A", vec![], days_to_duration(1.0), days_to_duration(3.0)),
        Task::new(
            "B",
            vec!["A".to_string()],
            days_to_duration(2.0),
            days_to_duration(4.0),
        ),
        Task::new(
            "C",
            vec!["A".to_string()],
            days_to_duration(3.0),
            days_to_duration(5.0),
        ),
        Task::new(
            "D",
            vec!["B".to_string(), "C".to_string()],
            days_to_duration(2.0),
            days_to_duration(6.0),
        ),
        Task::new(
            "E",
            vec!["B".to_string()],
            days_to_duration(1.0),
            days_to_duration(2.0),
        ),
        Task::new(
            "F",
            vec!["D".to_string(), "E".to_string()],
            days_to_duration(2.0),
            days_to_duration(4.0),
        ),
    ];

    let estimate_confidence = 0.8;
    let num_simulations = 1000;

    let mut all_efforts = Vec::new();

    for num_workers in [1, 2, 4, 8, 16, 32] {
        let schedule = Schedule::new(tasks.clone(), num_workers, estimate_confidence)
            .expect("Failed to create schedule");
        let (project_durations, effort_times) =
            run_multiple_simulations(&schedule, num_simulations);

        let avg_duration = project_durations.iter().sum::<Duration>() / num_simulations as u32;
        let avg_effort = effort_times.iter().sum::<Duration>() / num_simulations as u32;

        println!(
            "Workers: {}, Avg Duration: {:.2} days, Avg Effort: {:.2} person-days",
            num_workers,
            avg_duration.as_secs_f64() / 86400.0,
            avg_effort.as_secs_f64() / 86400.0
        );

        all_efforts.push(avg_effort);

        // Basic sanity checks
        assert!(
            avg_duration <= avg_effort,
            "Average duration should not exceed average effort"
        );
        if num_workers > 1 {
            assert!(
                avg_duration < avg_effort,
                "With multiple workers, average duration should be less than average effort"
            );
        }
    }

    // Check that effort remains relatively constant regardless of worker count
    let max_effort = all_efforts.iter().max().unwrap();
    let min_effort = all_efforts.iter().min().unwrap();
    let effort_difference = (max_effort.as_secs_f64() - min_effort.as_secs_f64()) / 86400.0;

    println!("Maximum effort difference: {:.2} days", effort_difference);
    assert!(
        effort_difference < 1.0,
        "Average effort should not vary significantly with different worker counts"
    );
}
