use crate::{schedule::Schedule, task::Task};

use serde::{Deserialize, Serialize};

use std::{fs::File, io::Read, path::Path, time::Duration};

#[derive(Debug, Deserialize, Serialize)]
struct ScheduleInput {
    num_workers: usize,
    estimate_confidence: f64,
    tasks: Vec<TaskInput>,
}

#[derive(Debug, Deserialize, Serialize)]
struct TaskInput {
    id: String,
    min_time: f64,
    max_time: f64,
    dependencies: Vec<String>,
}

enum FileFormat {
    Yaml,
    Json,
}

impl Schedule {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let mut file = File::open(&path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let format = Self::detect_format(&path)?;
        let input: ScheduleInput = match format {
            FileFormat::Yaml => serde_yaml::from_str(&contents)?,
            FileFormat::Json => serde_json::from_str(&contents)?,
        };

        let tasks = input
            .tasks
            .into_iter()
            .map(|t| Task {
                id: t.id,
                min_time: Duration::from_secs_f64(t.min_time * 24.0 * 60.0 * 60.0),
                max_time: Duration::from_secs_f64(t.max_time * 24.0 * 60.0 * 60.0),
                dependencies: t.dependencies,
            })
            .collect();

        let schedule = Schedule::new(tasks, input.num_workers, input.estimate_confidence)?;

        schedule.validate()?;

        Ok(schedule)
    }

    fn detect_format<P: AsRef<Path>>(path: P) -> Result<FileFormat, Box<dyn std::error::Error>> {
        let extension = path
            .as_ref()
            .extension()
            .and_then(std::ffi::OsStr::to_str)
            .ok_or("File has no extension")?
            .to_lowercase();

        match extension.as_str() {
            "yaml" | "yml" => Ok(FileFormat::Yaml),
            "json" => Ok(FileFormat::Json),
            _ => Err("Unsupported file format. Use .yaml, .yml, or .json".into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_temp_file(content: &str, extension: &str) -> (NamedTempFile, std::path::PathBuf) {
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", content).unwrap();
        let path = temp_file.path().to_owned();
        let new_path = path.with_extension(extension);
        std::fs::rename(&path, &new_path).unwrap();
        (temp_file, new_path)
    }

    #[test]
    fn test_load_from_yaml() {
        let yaml_content = r#"
num_workers: 3
estimate_confidence: 0.9
tasks:
  - id: A
    min_time: 1
    max_time: 3
    dependencies: []
  - id: B
    min_time: 2
    max_time: 4
    dependencies: [A]
"#;
        let (_temp_file, path) = create_temp_file(yaml_content, "yaml");
        let schedule = Schedule::from_file(path).unwrap();
        assert_eq!(schedule.tasks.len(), 2);
        assert_eq!(schedule.num_workers, 3);
        assert_eq!(schedule.estimate_confidence, 0.9);
    }

    #[test]
    fn test_load_from_json() {
        let json_content = r#"
{
  "num_workers": 3,
  "estimate_confidence": 0.9,
  "tasks": [
    {
      "id": "A",
      "min_time": 1,
      "max_time": 3,
      "dependencies": []
    },
    {
      "id": "B",
      "min_time": 2,
      "max_time": 4,
      "dependencies": ["A"]
    }
  ]
}
"#;
        let (_temp_file, path) = create_temp_file(json_content, "json");
        let schedule = Schedule::from_file(path).unwrap();
        assert_eq!(schedule.tasks.len(), 2);
        assert_eq!(schedule.num_workers, 3);
        assert_eq!(schedule.estimate_confidence, 0.9);
    }

    #[test]
    fn test_unsupported_format() {
        let content = "irrelevant content";
        let (_temp_file, path) = create_temp_file(content, "txt");
        let result = Schedule::from_file(path);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unsupported file format"));
    }

    #[test]
    fn test_invalid_schedule() {
        let yaml_content = r#"
num_workers: 3
estimate_confidence: 0.9
tasks:
  - id: A
    min_time: 3
    max_time: 1  # Invalid: min_time > max_time
    dependencies: []
"#;
        let (_temp_file, path) = create_temp_file(yaml_content, "yaml");
        let result = Schedule::from_file(path);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Minimum duration greater than maximum for task A"));
    }

    #[test]
    fn test_load_with_floating_point_times_yaml() {
        let yaml_content = r#"
num_workers: 3
estimate_confidence: 0.9
tasks:
  - id: A
    min_time: 1.5  # 1.5 days
    max_time: 3.5  # 3.5 days
    dependencies: []
"#;
        let (_temp_file, path) = create_temp_file(yaml_content, "yaml");
        let schedule = Schedule::from_file(path).unwrap();

        // Verify that there is one task and it has the correct times
        assert_eq!(schedule.tasks.len(), 1);
        assert_eq!(
            schedule.tasks[0].min_time.as_secs_f64(),
            1.5 * 24.0 * 60.0 * 60.0
        );
        assert_eq!(
            schedule.tasks[0].max_time.as_secs_f64(),
            3.5 * 24.0 * 60.0 * 60.0
        );
    }

    #[test]
    fn test_load_with_floating_point_times_json() {
        let json_content = r#"
{
  "num_workers": 3,
  "estimate_confidence": 0.9,
  "tasks": [
    {
      "id": "A",
      "min_time": 1.5,
      "max_time": 3.5,
      "dependencies": []
    }
  ]
}
"#;
        let (_temp_file, path) = create_temp_file(json_content, "json");
        let schedule = Schedule::from_file(path).unwrap();

        // Verify that there is one task and it has the correct times
        assert_eq!(schedule.tasks.len(), 1);
        assert_eq!(
            schedule.tasks[0].min_time.as_secs_f64(),
            1.5 * 24.0 * 60.0 * 60.0
        );
        assert_eq!(
            schedule.tasks[0].max_time.as_secs_f64(),
            3.5 * 24.0 * 60.0 * 60.0
        );
    }
}
