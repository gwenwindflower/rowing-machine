use std::{hint::black_box, path::PathBuf, time::Duration};

use criterion::{BatchSize, Criterion, Throughput, criterion_group, criterion_main};
use jiff::civil::date;
use rowing_machine::{engine::RunConfig, run_scenario, scenario::ScenarioKind, theme::Theme};

fn config(output_dir: PathBuf, workers: usize, scale: usize) -> RunConfig {
    RunConfig {
        days: 4 * 365,
        scale,
        seed: 42,
        start_date: date(2023, 1, 1),
        output_dir,
        prefix: "benchmark".into(),
        quiet: true,
        format: rowing_machine::output::Format::Csv,
        compress: false,
        target_rows: None,
        workers,
        renames: rowing_machine::output::Renames::default(),
    }
}

fn scenarios(criterion: &mut Criterion) {
    for (kind, scale) in [
        (ScenarioKind::Ecommerce, 10),
        (ScenarioKind::Ecommerce, 100),
        (ScenarioKind::Saas, 10),
        (ScenarioKind::Saas, 100),
    ] {
        if std::env::var("ROWING_BENCH_SCENARIO").is_ok_and(|name| name != kind.name())
            || std::env::var("ROWING_BENCH_SCALE").is_ok_and(|value| value != scale.to_string())
        {
            continue;
        }
        benchmark_scale(criterion, kind, scale);
    }
}

fn benchmark_scale(criterion: &mut Criterion, kind: ScenarioKind, scale: usize) {
    let generate = |directory: PathBuf, workers| {
        run_scenario(
            &config(directory, workers, scale),
            Theme::load("plain").expect("load benchmark theme"),
            kind,
        )
    };
    let rows: u64 = {
        let directory = tempfile::tempdir().expect("create benchmark output directory");
        generate(directory.path().to_path_buf(), 1)
            .expect("generate benchmark row counts")
            .values()
            .sum()
    };
    let mut group =
        criterion.benchmark_group(format!("{}/seed42_scale{scale}_years4_csv", kind.name()));
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(10));
    group.throughput(Throughput::Elements(rows));
    let available = std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get);
    if let Some(path) = std::env::var_os("ROWING_BENCH_WORKERS_FILE") {
        std::fs::write(path, available.to_string()).expect("record available benchmark workers");
    }
    println!("Worker settings: workers_1=1, workers_available={available}");
    for (label, workers) in [("workers_1", 1), ("workers_available", available)] {
        group.bench_function(label, |bencher| {
            bencher.iter_batched_ref(
                || tempfile::tempdir().expect("create benchmark output directory"),
                |directory| {
                    black_box(
                        generate(directory.path().to_path_buf(), workers)
                            .expect("generate benchmark CSV output"),
                    )
                },
                BatchSize::PerIteration,
            );
        });
    }
    group.finish();
}

criterion_group!(benches, scenarios);
criterion_main!(benches);
