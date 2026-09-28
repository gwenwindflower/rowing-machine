use std::{hint::black_box, path::PathBuf, time::Duration};

use criterion::{BatchSize, Criterion, Throughput, criterion_group, criterion_main};
use jiff::civil::date;
use rowing_machine::{engine::RunConfig, run};

fn config(output_dir: PathBuf) -> RunConfig {
    RunConfig {
        days: 4 * 365,
        scale: 10,
        seed: 42,
        start_date: date(2023, 1, 1),
        output_dir,
        prefix: "benchmark".into(),
        quiet: true,
    }
}

fn ecommerce(criterion: &mut Criterion) {
    let rows: u64 = {
        let directory = tempfile::tempdir().expect("create benchmark output directory");
        run(&config(directory.path().to_path_buf()))
            .expect("generate benchmark row counts")
            .values()
            .sum()
    };
    let mut group = criterion.benchmark_group("ecommerce");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(10));
    group.throughput(Throughput::Elements(rows));
    group.bench_function("seed42_scale10_years4_csv", |bencher| {
        bencher.iter_batched_ref(
            || tempfile::tempdir().expect("create benchmark output directory"),
            |directory| {
                black_box(
                    run(&config(directory.path().to_path_buf()))
                        .expect("generate benchmark CSV output"),
                )
            },
            BatchSize::PerIteration,
        );
    });
    group.finish();
}

criterion_group!(benches, ecommerce);
criterion_main!(benches);
