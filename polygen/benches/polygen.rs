#[macro_use]
extern crate criterion;

use criterion::*;
use polygen::*;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench(
        "generate_cubic_sphere_vertices",
        ParameterizedBenchmark::new(
            "do_thing",
            |b, &i| b.iter(|| generate_cubic_sphere_vertices(1.0, i)),
            vec![
                0, 1, 2, 3, 4, 5, 6, 7, 8, 9, //
                10, 20, 30, 40, 50, 60, 70, 80, 90, //
                100, 200, 300, 400, 500,
            ],
        )
        .warm_up_time(std::time::Duration::from_secs(1))
        .measurement_time(std::time::Duration::from_secs(1))
        .sample_size(20),
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
