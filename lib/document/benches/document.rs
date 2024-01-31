use criterion::{black_box, criterion_group, criterion_main, Criterion};

const SRC: &str = include_str!("../tests/testfiles/todo.trax");

fn bench(c: &mut Criterion) {
    let mut parse_group = c.benchmark_group("document");

    parse_group.bench_function("todo", |b| {
        b.iter(|| trax_document::Document::new(black_box(SRC)))
    });

    parse_group.finish();
}

criterion_group!(
    name = benches;
    config = Criterion::default();
    targets = bench
);

criterion_main!(benches);
