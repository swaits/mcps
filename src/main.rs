use chrono::{NaiveDate, Utc};
use mcps::{schedule::Project, simulation::run_multiple_simulations};

use clap::{Arg, Command};
use workdays::WorkCalendar;

use std::{str::FromStr, time::Duration};

const AFTER_HELP_TEXT: &str = "\
Find example project definition and schedule config files in the repository:

    https://github.com/swaits/mcps/blob/main/examples/

";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("Monte Carlo Project Scheduler")
        .version("0.3.0")
        .author("Stephen Waits <steve@waits.net>")
        .about("Runs Monte Carlo simulations on project schedules")
        .arg(
            Arg::new("filename")
                .help("Path to the project file (.yaml or .json)")
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
                .short('n')
                .long("workers")
                .help("Override `num_workers` specified in project file")
                .value_name("num_workers"),
        )
        .arg(
            Arg::new("begin")
                .short('b')
                .long("begin")
                .help("Override `start_date` specified in project file")
                .value_name("YYYY-MM-DD"),
        )
        .arg(
            Arg::new("schedule")
                .short('s')
                .long("schedule")
                .help("Work schedule config file (.yaml or .json)")
                .value_name("filename"),
        )
        .after_help(AFTER_HELP_TEXT)
        .get_matches();

    let project_path = matches.get_one::<String>("filename").unwrap();
    let num_simulations: usize = matches.get_one::<String>("iterations").unwrap().parse()?;

    // Ensure iterations is >= 100
    if num_simulations < 100 {
        return Err("Iterations must be at least 100.".into());
    }

    // Load the project
    let mut project = Project::from_file(project_path)?;

    // Load work schedule if it exists
    let calendar: WorkCalendar = match matches.get_one::<String>("schedule") {
        Some(filename) => WorkCalendar::from_str(&std::fs::read_to_string(filename)?)?,
        None => WorkCalendar::new(),
    };

    // Determine the start date (command line > project file > TODAY)
    let start_date = matches
        .get_one::<String>("begin")
        .and_then(|date| NaiveDate::parse_from_str(date, "%Y-%m-%d").ok())
        .or(project.start_date)
        .unwrap_or_else(|| Utc::now().date_naive());

    // Check if workers are overridden by command-line argument
    if let Some(workers_str) = matches.get_one::<String>("workers") {
        let workers: usize = workers_str.parse()?;
        if workers < 1 {
            return Err("Invalid number of workers, must be 1 or more".into());
        }
        project.num_workers = workers;
    }

    // Monte Carlo simulation
    let (project_durations, effort_times) = run_multiple_simulations(&project, num_simulations);

    // Results output
    print_ascii_cdf(
        &project_durations,
        format!(
            "Completion Time ({} Worker{}, starting {})",
            project.num_workers,
            if project.num_workers == 1 { "" } else { "s" },
            start_date,
        )
        .as_str(),
        &start_date,
        &calendar,
    );

    println!();

    print_ascii_cdf(
        &effort_times,
        format!("Total Work Effort (1 worker, starting {})", start_date).as_str(),
        &start_date,
        &calendar,
    );

    Ok(())
}

fn print_ascii_cdf(data: &[Duration], title: &str, start: &NaiveDate, calendar: &WorkCalendar) {
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

    println!("────┬────────────────────────────────────────────────────────────┬──────────┬──────────┬──────────");
    println!("%ile│{}│ Workdays │ Schedule │ Complete  ", centered_title);
    println!("────┼────────────────────────────────────────────────────────────┼──────────┼──────────┼──────────");

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
        let (end_date, calendar_duration) =
            calendar.compute_end_date(*start, *days as i64).unwrap();

        let shifted_bar_position = bar_position + offset;

        let (fg, bg) = if i % 2 == 0 {
            ('░', '▓')
        } else {
            ('▒', '█')
        };

        let color_code = match 100 - i * 5 {
            0..=45 => "\x1b[31m",        // Red
            50..=65 => "\x1b[38;5;173m", // Orange
            70..=80 => "\x1b[33m",       // Yellow
            85..=95 => "\x1b[32m",       // Green
            96..=100 => "\x1b[33m",      // Yellow
            _ => "\x1b[0m",              // Default (shouldn't happen)
        };
        let reset_code = "\x1b[0m";

        let bar_with_divider: String = (0..bar_width)
            .map(|j| match j {
                _ if j == shifted_bar_position => '▮',
                _ if j < shifted_bar_position => fg,
                _ => bg,
            })
            .collect();

        let trailing = match 100 - i * 5 {
            95 => "◀━┓",
            60 => "  ┣━━━━━━━━━━━━┓",
            55 => "  ┃ 90%        ┃",
            50 => "  ┃ Confidence ┃",
            45 => "  ┃ Interval   ┃",
            40 => "  ┣━━━━━━━━━━━━┛",
            5 => "◀━┛",
            6..=94 => "  ┃",
            _ => "",
        };

        println!(
            "{}{:>4}{}│{}{}{}│{}{:5.0} days{}│{}{:5.0} days{}│{}{}{}{}",
            color_code,
            format!("p{}", 100 - i * 5),
            reset_code,
            color_code,
            bar_with_divider,
            reset_code,
            color_code,
            days,
            reset_code,
            color_code,
            calendar_duration.num_days(),
            reset_code,
            color_code,
            end_date,
            reset_code,
            trailing
        );
    }
    println!("────┴────────────────────────────────────────────────────────────┴──────────┴──────────┴──────────");
}
