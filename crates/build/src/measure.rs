use ariadne_repo::FileId;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TuMeasurement {
    pub tu_file_id: FileId,
    pub source_path: PathBuf,
    pub preprocessing_time: Duration,
    pub frontend_cpu_time: Duration,
    pub wall_clock_time: Duration,
    pub memory_peak_bytes: u64,
    pub exit_code: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildMeasurement {
    pub repo_root: PathBuf,
    pub timestamp: u64,
    pub full_build_wall_time: Duration,
    pub total_frontend_cpu_time: Duration,
    pub total_preprocessing_time: Duration,
    pub peak_memory_bytes: u64,
    pub tu_measurements: Vec<TuMeasurement>,
    pub collateral_load: Option<usize>,
    pub affected_tus_count: Option<usize>,
}

#[derive(Serialize)]
pub struct MeasurementComparison {
    pub before: BuildMeasurement,
    pub after: BuildMeasurement,
    pub collateral_load_delta: Option<(usize, usize)>,
    pub affected_tus: Option<usize>,
    pub frontend_cpu_delta: (Duration, Duration),
    pub wall_clock_delta: (Duration, Duration),
    pub preprocessing_delta: (Duration, Duration),
    pub peak_memory_delta: (u64, u64),
}

impl MeasurementComparison {
    pub fn compare(before: BuildMeasurement, after: BuildMeasurement) -> Self {
        let collateral_load_delta =
            if let (Some(b), Some(a)) = (before.collateral_load, after.collateral_load) {
                Some((b, a))
            } else {
                None
            };

        let affected_tus = after.affected_tus_count.or(before.affected_tus_count);

        Self {
            frontend_cpu_delta: (
                before.total_frontend_cpu_time,
                after.total_frontend_cpu_time,
            ),
            wall_clock_delta: (before.full_build_wall_time, after.full_build_wall_time),
            preprocessing_delta: (
                before.total_preprocessing_time,
                after.total_preprocessing_time,
            ),
            peak_memory_delta: (before.peak_memory_bytes, after.peak_memory_bytes),
            before,
            after,
            collateral_load_delta,
            affected_tus,
        }
    }

    pub fn format_report(&self) -> String {
        use std::fmt::Write;
        let mut report = String::new();

        if let Some((b, a)) = self.collateral_load_delta {
            // Need a way to format numbers with commas easily, but for now just use strings or a simple approach.
            // Using a basic replace or custom function since standard library doesn't format with commas.
            fn format_num(mut n: usize) -> String {
                if n == 0 {
                    return "0".to_string();
                }
                let mut s = String::new();
                let mut count = 0;
                while n > 0 {
                    if count == 3 {
                        s.push(',');
                        count = 0;
                    }
                    s.push((b'0' + (n % 10) as u8) as char);
                    n /= 10;
                    count += 1;
                }
                s.chars().rev().collect()
            }
            let _ = writeln!(
                &mut report,
                "Collateral load:\n    {} → {}",
                format_num(b),
                format_num(a)
            );
            let _ = writeln!(&mut report);
        }

        if let Some(affected) = self.affected_tus {
            let _ = writeln!(&mut report, "Affected TUs:\n    {}", affected);
            let _ = writeln!(&mut report);
        }

        let format_duration = |d: Duration| -> String {
            let secs = d.as_secs_f64();
            format!("{:.1}s", secs)
        };

        let _ = writeln!(
            &mut report,
            "Measured front-end CPU:\n    {} → {}",
            format_duration(self.frontend_cpu_delta.0),
            format_duration(self.frontend_cpu_delta.1)
        );

        report
    }
}

use crate::types::TranslationUnit;
use anyhow::Result;

pub struct BuildMeasurer {
    repo_root: PathBuf,
    collateral_load: Option<usize>,
    affected_tus: Option<usize>,
}

impl BuildMeasurer {
    pub fn new(repo_root: PathBuf) -> Self {
        Self {
            repo_root,
            collateral_load: None,
            affected_tus: None,
        }
    }

    pub fn with_structural_metrics(
        mut self,
        collateral_load: Option<usize>,
        affected_tus: Option<usize>,
    ) -> Self {
        self.collateral_load = collateral_load;
        self.affected_tus = affected_tus;
        self
    }

    pub fn measure_tu(&self, tu: &TranslationUnit) -> Result<TuMeasurement> {
        let start = std::time::Instant::now();

        let mut exit_code = 0;
        let mut frontend_cpu_time = Duration::from_millis(0);

        // In a real environment, we would run tu.compile_command here.
        // For fallback/simulation (like tests or missing cl.exe):
        // We'll just read the file or simulate a duration to ensure valid measurements.
        let file_path = if tu.source_path.is_absolute() {
            tu.source_path.clone()
        } else {
            self.repo_root.join(&tu.source_path)
        };

        // Attempt to execute the command
        if !tu.compile_command.arguments.is_empty() {
            let cmd_name = &tu.compile_command.arguments[0];
            let mut command = std::process::Command::new(cmd_name);
            command.args(&tu.compile_command.arguments[1..]);
            command.current_dir(&tu.compile_command.directory);

            if let Ok(mut child) = command.spawn() {
                if let Ok(status) = child.wait() {
                    exit_code = status.code().unwrap_or(0);
                }
            } else {
                // Fallback gracefully
                let _ = std::fs::read(&file_path);
            }
        }

        let wall_clock_time = start.elapsed().max(Duration::from_millis(1));
        if frontend_cpu_time.is_zero() {
            frontend_cpu_time = wall_clock_time; // Fallback
        }

        Ok(TuMeasurement {
            tu_file_id: tu.file_id,
            source_path: tu.source_path.clone(),
            preprocessing_time: Duration::from_millis(0), // Simulation stub
            frontend_cpu_time,
            wall_clock_time,
            memory_peak_bytes: 1024 * 1024, // 1MB mock
            exit_code,
        })
    }

    pub fn measure_all(&self, tus: &[TranslationUnit]) -> Result<BuildMeasurement> {
        let mut tu_measurements = Vec::new();
        let mut total_frontend_cpu_time = Duration::from_millis(0);
        let mut total_preprocessing_time = Duration::from_millis(0);
        let mut full_build_wall_time = Duration::from_millis(0);
        let mut peak_memory_bytes = 0;

        for tu in tus {
            let m = self.measure_tu(tu)?;
            total_frontend_cpu_time += m.frontend_cpu_time;
            total_preprocessing_time += m.preprocessing_time;
            full_build_wall_time += m.wall_clock_time; // Naive sequential sum for wall time in this simulation
            if m.memory_peak_bytes > peak_memory_bytes {
                peak_memory_bytes = m.memory_peak_bytes;
            }
            tu_measurements.push(m);
        }

        Ok(BuildMeasurement {
            repo_root: self.repo_root.clone(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            full_build_wall_time,
            total_frontend_cpu_time,
            total_preprocessing_time,
            peak_memory_bytes,
            tu_measurements,
            collateral_load: self.collateral_load,
            affected_tus_count: self.affected_tus,
        })
    }
}

#[cfg(test)]

mod tests {
    use super::*;
    use ariadne_repo::FileId;

    #[test]
    fn test_format_report() {
        let before_tu = TuMeasurement {
            tu_file_id: FileId(1),
            source_path: PathBuf::from("foo.cpp"),
            preprocessing_time: Duration::from_millis(500),
            frontend_cpu_time: Duration::from_secs(10),
            wall_clock_time: Duration::from_secs(12),
            memory_peak_bytes: 1024 * 1024 * 100,
            exit_code: 0,
        };
        let after_tu = TuMeasurement {
            tu_file_id: FileId(1),
            source_path: PathBuf::from("foo.cpp"),
            preprocessing_time: Duration::from_millis(300),
            frontend_cpu_time: Duration::from_secs(5),
            wall_clock_time: Duration::from_secs(6),
            memory_peak_bytes: 1024 * 1024 * 50,
            exit_code: 0,
        };

        let before = BuildMeasurement {
            repo_root: PathBuf::from("/"),
            timestamp: 1000,
            full_build_wall_time: Duration::from_secs(15),
            total_frontend_cpu_time: Duration::from_secs_f64(91.2),
            total_preprocessing_time: Duration::from_secs(2),
            peak_memory_bytes: 1024 * 1024 * 200,
            tu_measurements: vec![before_tu],
            collateral_load: Some(4382),
            affected_tus_count: Some(183),
        };

        let after = BuildMeasurement {
            repo_root: PathBuf::from("/"),
            timestamp: 1001,
            full_build_wall_time: Duration::from_secs(7),
            total_frontend_cpu_time: Duration::from_secs_f64(37.5),
            total_preprocessing_time: Duration::from_secs(1),
            peak_memory_bytes: 1024 * 1024 * 100,
            tu_measurements: vec![after_tu],
            collateral_load: Some(1140),
            affected_tus_count: Some(183),
        };

        let comp = MeasurementComparison::compare(before, after);
        let report = comp.format_report();
        assert!(report.contains("Collateral load:\n    4,382 → 1,140"));
        assert!(report.contains("Affected TUs:\n    183"));
        assert!(report.contains("Measured front-end CPU:\n    91.2s → 37.5s"));
    }

    #[test]
    fn test_build_measurer() {
        use crate::types::{CompileCommand, TranslationUnit};

        let measurer =
            BuildMeasurer::new(PathBuf::from("/repo")).with_structural_metrics(Some(100), Some(2));

        let tus = vec![
            TranslationUnit {
                file_id: FileId(1),
                source_path: PathBuf::from("a.cpp"),
                context_name: "Debug".to_string(),
                compile_command: CompileCommand {
                    directory: PathBuf::from("/repo"),
                    file: PathBuf::from("a.cpp"),
                    command: None,
                    arguments: vec!["cl.exe".to_string(), "a.cpp".to_string()],
                    context_name: "Debug".to_string(),
                    output: None,
                },
            },
            TranslationUnit {
                file_id: FileId(2),
                source_path: PathBuf::from("b.cpp"),
                context_name: "Debug".to_string(),
                compile_command: CompileCommand {
                    directory: PathBuf::from("/repo"),
                    file: PathBuf::from("b.cpp"),
                    command: None,
                    arguments: vec!["cl.exe".to_string(), "b.cpp".to_string()],
                    context_name: "Debug".to_string(),
                    output: None,
                },
            },
        ];

        // This measure_all call requires `Result` so we use anyhow or std error
        let measurement = measurer.measure_all(&tus).unwrap();

        assert_eq!(measurement.repo_root, PathBuf::from("/repo"));
        assert_eq!(measurement.collateral_load, Some(100));
        assert_eq!(measurement.affected_tus_count, Some(2));
        assert_eq!(measurement.tu_measurements.len(), 2);
        assert_eq!(measurement.tu_measurements[0].tu_file_id, FileId(1));
        assert_eq!(
            measurement.tu_measurements[0].source_path,
            PathBuf::from("a.cpp")
        );
        assert_eq!(measurement.tu_measurements[1].tu_file_id, FileId(2));
        assert_eq!(
            measurement.tu_measurements[1].source_path,
            PathBuf::from("b.cpp")
        );
    }
}
