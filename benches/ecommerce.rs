use std::{hint::black_box, path::PathBuf, time::Duration};

use criterion::{BatchSize, Criterion, Throughput, criterion_group, criterion_main};
use jiff::civil::date;
use rowing_machine::{engine::RunConfig, run};

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
    }
}

fn ecommerce(criterion: &mut Criterion) {
    for scale in [10, 100] {
        benchmark_scale(criterion, scale);
    }
}

fn benchmark_scale(criterion: &mut Criterion, scale: usize) {
    let rows: u64 = {
        let directory = tempfile::tempdir().expect("create benchmark output directory");
        run(&config(directory.path().to_path_buf(), 1, scale))
            .expect("generate benchmark row counts")
            .values()
            .sum()
    };
    let mut group = criterion.benchmark_group(format!("ecommerce/seed42_scale{scale}_years4_csv"));
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(10));
    group.throughput(Throughput::Elements(rows));
    let available = std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get);
    println!("Worker settings: workers_1=1, workers_available={available}");
    for (label, workers) in [("workers_1", 1), ("workers_available", available)] {
        group.bench_function(label, |bencher| {
            bencher.iter_batched_ref(
                || tempfile::tempdir().expect("create benchmark output directory"),
                |directory| {
                    black_box(
                        run(&config(directory.path().to_path_buf(), workers, scale))
                            .expect("generate benchmark CSV output"),
                    )
                },
                BatchSize::PerIteration,
            );
        });
    }
    group.finish();
}

criterion_group!(benches, ecommerce);
criterion_main!(benches);
