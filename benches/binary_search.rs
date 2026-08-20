use criterion::{Criterion, criterion_group, criterion_main};
use rand::RngExt;
use zcr::{RecordBuilder, WrappedValue};

fn record_has(c: &mut Criterion) {
    let mut group = c.benchmark_group("Record has item");

    for item_count in [1u64, 5, 10, 100, 1_000, 10_000] {
        let record = {
            let mut builder = RecordBuilder::default();

            for item in 0..item_count {
                builder = builder.prop(format!("hello world {item}"), WrappedValue::Boolean(false));
            }

            builder.finish()
        };

        let mut rng = rand::rng();

        group.bench_function(format!("{item_count} items"), |b| {
            b.iter(|| {
                let key = format!("hello world {}", rng.random_range(0..item_count));
                record.has(&key.as_bytes())
            })
        });
    }
}

fn list_has_linear(c: &mut Criterion) {
    let mut group = c.benchmark_group("List has item (linear)");

    for item_count in [1u64, 5, 10, 100, 1_000, 10_000] {
        let list = {
            let mut builder = RecordBuilder::default();

            for item in 0..item_count {
                builder = builder.prop(
                    &*format!("hello world {item}"),
                    format!("hello world {item}"),
                );
            }

            builder.finish().into_values()
        };

        let mut rng = rand::rng();

        group.bench_function(format!("{item_count} items"), |b| {
            b.iter(|| {
                let key = format!("hello world {}", rng.random_range(0..item_count));

                list.iter()
                    .find(|v| unsafe { v.as_str_unchecked().unwrap() } == &key)
            })
        });
    }
}

fn list_has_binary_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("List has item (binary)");

    for item_count in [1u64, 5, 10, 100, 1_000, 10_000] {
        let list = {
            let mut builder = RecordBuilder::default();

            for item in 0..item_count {
                builder = builder.prop(
                    &*format!("hello world {item}"),
                    format!("hello world {item}"),
                );
            }

            builder.finish().into_values()
        };

        let mut rng = rand::rng();

        group.bench_function(format!("{item_count} items"), |b| {
            b.iter(|| {
                let key = format!("hello world {}", rng.random_range(0..item_count));

                list.get(
                    list.binary_search_by(|v| {
                        let v = unsafe { v.as_str_unchecked().unwrap() };
                        v.cmp(&key)
                    })
                    .unwrap(),
                )
            })
        });
    }
}

fn sorted_set_has(c: &mut Criterion) {
    let mut group = c.benchmark_group("SortedSet has item (binary)");

    for item_count in [1u64, 5, 10, 100, 1_000, 10_000] {
        let set = {
            let mut builder = zcr::SortedSetBuilder::default();

            for item in 0..item_count {
                builder.insert(format!("hello world {item}"));
            }

            builder.finish()
        };

        let mut rng = rand::rng();

        group.bench_function(format!("{item_count} items"), |b| {
            b.iter(|| {
                let key = format!("hello world {}", rng.random_range(0..item_count));
                set.has(key.as_bytes())
            })
        });
    }
}

criterion_group!(
    benches,
    record_has,
    list_has_linear,
    list_has_binary_search,
    sorted_set_has,
);
criterion_main!(benches);
