# Monte Carlo Project Scheduler (mcps)

Monte Carlo Project Scheduler (mcps) is a command-line tool that runs Monte
Carlo simulations on project schedules to estimate completion times and effort
required. It helps project managers and engineers gain a better understanding of
potential project risks and timelines by providing a probabilistic analysis of
project durations and total work effort.

## Features

- **Simulate Project Schedules**: Run multiple simulations to estimate the
  probability distribution of project completion times and total work effort.
- **Customizable Inputs**: Accepts project definitions in YAML or JSON formats,
  with options to override certain parameters like the number of workers or the
  number of iterations.
- **Visual Output**: Generates an ASCII-based cumulative distribution function
  (CDF) graph, providing an easy-to-understand visualization of the simulation
  results.

## Installation

### Prerequisites

You'll need to have Rust installed on your system to build the project. You can
install Rust using `rustup`:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Option 1: Install from Crates.io (preferred)

```bash
cargo install mcps
```

Note, if you're upgrading, use `cargo install --force mcps` to force
installation of the latest version..

### Option 2: Clone and build the repository locally

```bash
git clone https://github.com/swaits/mcps.git
cd mcps
cargo build --release
```

The binary will be available at `target/release/mcps`.

## Usage

Once you have built the tool, you can use it by providing a project definition
file in YAML or JSON format.

### Basic Usage

```bash
mcps project.yaml -i 100000 -w 10
```

This command runs the Monte Carlo simulation on the
[`project.yaml`](./assets/project.yaml) project file with 100,000 iterations
and overrides the number of workers to 10.

### Command-Line Options

- `-i, --iterations <iterations>`: Specify the number of iterations to run. Must
  be at least 100. Default is 50,000.
- `-n, --workers <num_workers>`: Override `num_workers` specified in project file
- `-b, --begin <YYYY-MM-DD>`: Override `start_date` specified in project file
- `-s, --schedule <filename>`: Work schedule config file (.yaml or .json)
- `-h, --help`: Print help
- `-V, --version`: Print version

### Project Definition File Format

`mcps` accepts projects in YAML or JSON format. Below is an example of the
YAML format:

```yaml
num_workers: 5 # for the purpose of scheduling simulation
start_date: 2024-08-01 # [optional] project start date
tasks:
  - id: DesignPhase
    estimate:
      min: 1.5 # 1.5 days
      likely: 2.2 # 2.2 days (best guess at actual time)
      max: 3.5 # 3.5 days
    dependencies: []
  - id: ImplementationPhase
    estimate:
      min: 2.25 # 2.25 days
      likely: 3.9 # 3.9 days (best guess at actual time)
      max: 4.75 # 4.75 days
    dependencies: [DesignPhase]
```

This repo includes examples in [JSON](./assets/project.json) and
[YAML](./assets/project.yaml).

### Work Schedule Configuration File Format

This file is optional. It allows you to configure the days of the week you work
along with any specific work holidays.

If you don't provide a schedule file, the tool will default to a 5-day work week
with no scheduled holidays.

`mcps` accepts schedules in YAML or JSON format. Below is an example of the
YAML format:

work_days:

```yaml
work_days:
  - Monday
  - Tuesday
  - Wednesday
holidays:
  - 2023-12-25
```

This repo includes examples in [JSON](./assets/schedule.json) and
[YAML](./assets/schedule.yaml).

### Example Output

The tool generates an ASCII-based cumulative distribution function (CDF) graph,
which visually represents the distribution of project durations and effort:

![example output of mcps](./assets/output.png)

## Project Details

### Contributing

Contributions are welcome! If you'd like to contribute, please fork the
repository and make changes as you'd like. Pull requests are warmly welcomed.

### Issues

If you encounter any issues with the tool, feel free to open an issue on the repository.

### Acknowledgments

This tool was developed by Stephen Waits. Contributions and suggestions are
always welcome!

### License

This project is licensed under the MIT License. See the [LICENSE](./LICENSE)
file for details.
