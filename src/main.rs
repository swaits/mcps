use mcps::{schedule::Schedule, simulation::run_multiple_simulations};

use clap::{Arg, Command};

use std::time::Duration;

const AFTER_HELP_TEXT: &str = "\
Examples of supported file formats:

    YAML Format:

        num_workers: 5
        estimate_confidence: 0.80
        tasks:
        - id: DesignPhase
            min_time: 1.5  # 1.5 days
            max_time: 3.5  # 3.5 days
            dependencies: []
        - id: ImplementationPhase
            min_time: 2.25  # 2.25 days
            max_time: 4.75  # 4.75 days
            dependencies: [DesignPhase]

    JSON Format:

        {
            \"num_workers\": 5,
            \"estimate_confidence\": 0.80,
            \"tasks\": [
                {
                    \"id\": \"DesignPhase\",
                    \"min_time\": 1.5,
                    \"max_time\": 3.5,
                    \"dependencies\": []
                },
                {
                    \"id\": \"ImplementationPhase\",
                    \"min_time\": 2.25,
                    \"max_time\": 4.75,
                    \"dependencies\": [\"DesignPhase\"]
                }
            ]
        }

    Note: `estimate_confidence` is the confidence (0.0,1.0) that the
          actual values fall in the task estimate ranges. For example,
          0.8 means you believe that the actual time it will take to
          complete a task falls in your [min_time,max_time] estimate
          range for that task 80% of the time.";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("Monte Carlo Project Scheduler")
        .version("1.0")
        .author("Stephen Waits <steve@waits.net>")
        .about("Runs Monte Carlo simulations on project schedules")
        .arg(
            Arg::new("filename")
                .help("Path to the schedule file (.yaml or .json).")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("iterations")
                .short('i')
                .long("iterations")
                .help("Number of iterations to run")
                .default_value("50000"),
        )
        .arg(
            Arg::new("workers")
                .short('w')
                .long("workers")
                .help("Override `num_workers` specified in schedule file")
                .value_name("num_workers"),
        )
        .after_help(AFTER_HELP_TEXT)
        .get_matches();

    let schedule_path = matches.get_one::<String>("filename").unwrap();
    let num_simulations: usize = matches.get_one::<String>("iterations").unwrap().parse()?;

    // Ensure iterations is >= 100
    if num_simulations < 100 {
        return Err("Iterations must be at least 100.".into());
    }

    // Load the schedule
    let mut schedule = Schedule::from_file(schedule_path)?;

    // Check if workers are overridden by command-line argument
    if let Some(workers_str) = matches.get_one::<String>("workers") {
        let workers: usize = workers_str.parse()?;
        if workers < 1 {
            return Err("Invalid number of workers, must be 1 or more".into());
        }
        schedule.num_workers = workers;
    }

    // Monte Carlo simulation
    let (project_durations, effort_times) = run_multiple_simulations(&schedule, num_simulations);

    // Results output
    print_ascii_cdf(
        &project_durations,
        format!(
            "Project Completion Time with {} Worker{} (by probability)",
            schedule.num_workers,
            if schedule.num_workers == 1 { "" } else { "s" }
        )
        .as_str(),
        "Duration",
    );
    println!();
    print_ascii_cdf(
        &effort_times,
        "Total Work Effort (by probability)",
        "Effort",
    );

    Ok(())
}

fn print_ascii_cdf(data: &[Duration], title: &str, units: &str) {
    let mut sorted_data = data.to_vec();
    sorted_data.sort_unstable();
    let min = sorted_data[0];
    let max = *sorted_data.last().unwrap();

    let width = 60; // Total width of the field

    // Calculate padding for the title
    let padding = (width - title.len()) / 2;
    let centered_title = format!(
        "{:padding_left$}{}{:padding_right$}",
        "",
        title,
        "",
        padding_left = padding,
        padding_right = width - padding - title.len()
    );

    println!("────┬────────────────────────────────────────────────────────────┬──────────");
    println!("%ile│{}│{:>10}", centered_title, units);
    println!("────┼────────────────────────────────────────────────────────────┼──────────");

    let bar_width = 60;

    // Calculate bar positions
    let mut bar_positions = Vec::new();
    for i in 0..=20 {
        let lower_percentile = if i == 20 {
            0.0000000001
        } else {
            (95 - i * 5) as f64 / 100.0
        };
        let upper_percentile = if i == 0 {
            0.9999999999
        } else {
            (100 - i * 5) as f64 / 100.0
        };
        let lower_index = (lower_percentile * (sorted_data.len() - 1) as f64).round() as usize;
        let upper_index = (upper_percentile * (sorted_data.len() - 1) as f64).round() as usize;
        let mid_duration = (sorted_data[lower_index] + sorted_data[upper_index]) / 2;
        let days = mid_duration.as_secs_f64() / 86400.0;
        let normalized_position = (days - min.as_secs_f64() / 86400.0)
            / (max.as_secs_f64() / 86400.0 - min.as_secs_f64() / 86400.0);
        let bar_position = (normalized_position * bar_width as f64).round() as usize;

        bar_positions.push((bar_position, days));
    }

    // Determine the needed shift to center the graph
    let min_bar_position = bar_positions.iter().map(|(pos, _)| pos).min().unwrap_or(&0);
    let max_bar_position = bar_positions
        .iter()
        .map(|(pos, _)| pos)
        .max()
        .unwrap_or(&bar_width);
    let offset = (bar_width - (max_bar_position - min_bar_position)) / 2;

    for (i, (bar_position, days)) in bar_positions.iter().enumerate() {
        let shifted_bar_position = bar_position + offset;

        let (fg, bg) = if i % 2 == 0 {
            ('░', '▓')
        } else {
            ('▒', '█')
        };

        let color_code = match 100 - i * 5 {
            0..=50 => "\x1b[31m",   // Red
            55..=75 => "\x1b[33m",  // Orange
            80..=90 => "\x1b[32m",  // Green
            95..=100 => "\x1b[31m", // Red
            _ => "\x1b[0m",         // Default (shouldn't happen)
        };
        let reset_code = "\x1b[0m";

        let bar_with_divider: String = (0..bar_width)
            .map(|j| match j {
                _ if j == shifted_bar_position => '▮',
                _ if j < shifted_bar_position => fg,
                _ => bg,
            })
            .collect();

        println!(
            "{}{:>4}{}│{}{}{}│{}{:5.0} days{}",
            color_code,
            format!("p{}", 100 - i * 5),
            reset_code,
            color_code,
            bar_with_divider,
            reset_code,
            color_code,
            days,
            reset_code
        );
    }
    println!("────┴────────────────────────────────────────────────────────────┴──────────");
}
