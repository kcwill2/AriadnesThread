use anyhow::Result;
use ariadne_build::BuildDiscoverer;
use ariadne_repo::Repository;
use ariadne_report::ArtifactManager;
use std::path::PathBuf;

pub fn handle_measure(repo_path: &PathBuf) -> Result<ariadne_build::BuildMeasurement> {
    let repository = Repository::locate(repo_path)?;
    let artifacts = ArtifactManager::prepare_for_repo(&repository)?;

    let repo_discovery = repository.discover().ok();
    let mut build_discoverer = BuildDiscoverer::new(&repository.root);
    if let Some(ref disc) = repo_discovery {
        build_discoverer = build_discoverer.with_repo_discovery(disc);
    }
    let build_result = build_discoverer.discover()?;

    let load_report = artifacts.load_load_artifacts().ok();
    let collateral_load = load_report.map(|lr| {
        lr.symbol_metrics
            .iter()
            .map(|m| m.collateral_load)
            .sum::<usize>()
    });

    let measurer = ariadne_build::BuildMeasurer::new(repository.root.clone())
        .with_structural_metrics(collateral_load, Some(build_result.translation_units.len()));

    println!(
        "Measuring {} translation units...",
        build_result.translation_units.len()
    );
    let measurement = measurer.measure_all(&build_result.translation_units)?;

    artifacts.save_measurement_artifacts(&measurement)?;

    println!(
        "Measured front-end CPU: {:.1}s",
        measurement.total_frontend_cpu_time.as_secs_f64()
    );
    println!(
        "Measured wall time: {:.1}s",
        measurement.full_build_wall_time.as_secs_f64()
    );
    println!("Peak memory: {} bytes", measurement.peak_memory_bytes);

    Ok(measurement)
}

pub fn handle_compare(
    before_path: &PathBuf,
    after_path: &PathBuf,
) -> Result<ariadne_build::MeasurementComparison> {
    let before = ArtifactManager::load_measurement_from_path(before_path)?;
    let after = ArtifactManager::load_measurement_from_path(after_path)?;

    let comparison = ariadne_build::MeasurementComparison::compare(before, after);
    println!("{}", comparison.format_report());

    Ok(comparison)
}
