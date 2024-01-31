use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use trax_document::EntityRef;

const SRC: &str = include_str!("../tests/testfiles/todo.trax");

fn bench(c: &mut Criterion) {
    let mut document_parse = c.benchmark_group("document_parse");

    document_parse.bench_function("todo", |b| {
        b.iter(|| trax_document::Document::new(black_box(SRC)))
    });

    document_parse.finish();

    let mut document_drop = c.benchmark_group("document_drop");

    let parsed_todo = trax_document::Document::new(SRC).unwrap();

    document_drop.bench_function("todo_large", |b| {
        b.iter_batched_ref(
            || parsed_todo.clone(),
            |mut doc| {
                doc.drop(EntityRef::Element(6)).unwrap();
                black_box(&mut doc);
            },
            BatchSize::SmallInput,
        )
    });

    document_drop.bench_function("todo_small", |b| {
        b.iter_batched_ref(
            || parsed_todo.clone(),
            |mut doc| {
                doc.drop(EntityRef::Element(15)).unwrap();
                black_box(&mut doc);
            },
            BatchSize::SmallInput,
        )
    });

    document_drop.finish();
}

criterion_group!(
    name = benches;
    config = Criterion::default();
    targets = bench
);

criterion_main!(benches);
