use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use ob::ob::OrderBook;
use ob::OrderInterface;
use rand::prelude::*;

/// Order type for benchmarks
#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct BenchOrder {
    id: u64,
    is_buy: bool,
    price: u64,
    quantity: u64,
    remaining: u64,
}

impl BenchOrder {
    pub fn new(id: u64, is_buy: bool, price: u64, quantity: u64) -> Self {
        Self {
            id,
            is_buy,
            price,
            quantity,
            remaining: quantity,
        }
    }
}

impl OrderInterface for BenchOrder {
    type T = u64;
    type N = u64;

    fn id(&self) -> &u64 {
        &self.id
    }

    fn price(&self) -> u64 {
        self.price
    }

    fn is_buy(&self) -> bool {
        self.is_buy
    }

    fn quantity(&self) -> u64 {
        self.quantity
    }

    fn remaining(&self) -> u64 {
        self.remaining
    }

    fn fill(&mut self, quantity: u64) {
        self.remaining -= quantity;
    }
}

/// Benchmark pure insert (no matching) - orders placed away from spread
fn bench_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("insert");
    group.throughput(Throughput::Elements(1));

    group.bench_function("insert_no_match", |b| {
        let mut ob = OrderBook::<BenchOrder>::default();
        let mut id = 0u64;

        b.iter(|| {
            // Alternate buy/sell orders at prices that won't match
            let is_buy = id % 2 == 0;
            let price = if is_buy { 900 } else { 1100 };
            let order = BenchOrder::new(id, is_buy, price, 100);
            let (_, instructions) = ob.eval_insert(black_box(order));
            ob.apply(instructions);
            id += 1;
        });
    });

    group.finish();
}

/// Benchmark matching - incoming order fully matches resting order
fn bench_match(c: &mut Criterion) {
    let mut group = c.benchmark_group("match");
    group.throughput(Throughput::Elements(1));

    group.bench_function("match_single", |b| {
        let mut ob = OrderBook::<BenchOrder>::default();
        let mut id = 0u64;

        b.iter(|| {
            // Insert a sell order, then match with a buy order
            let sell = BenchOrder::new(id, false, 1000, 100);
            let (_, instructions) = ob.eval_insert(sell);
            ob.apply(instructions);

            let buy = BenchOrder::new(id + 1, true, 1000, 100);
            let (_, instructions) = ob.eval_insert(black_box(buy));
            ob.apply(instructions);

            id += 2;
        });
    });

    group.bench_function("match_multi_maker", |b| {
        let mut ob = OrderBook::<BenchOrder>::default();
        let mut id = 0u64;

        b.iter(|| {
            // Insert 5 sell orders at different price levels
            for i in 0..5 {
                let sell = BenchOrder::new(id + i, false, 1000 + i, 20);
                let (_, instructions) = ob.eval_insert(sell);
                ob.apply(instructions);
            }

            // Match all with one aggressive buy
            let buy = BenchOrder::new(id + 5, true, 1010, 100);
            let (_, instructions) = ob.eval_insert(black_box(buy));
            ob.apply(instructions);

            id += 6;
        });
    });

    group.finish();
}

/// Benchmark cancel operations
fn bench_cancel(c: &mut Criterion) {
    let mut group = c.benchmark_group("cancel");
    group.throughput(Throughput::Elements(1));

    group.bench_function("cancel", |b| {
        let mut ob = OrderBook::<BenchOrder>::default();

        // Pre-populate with orders
        for i in 0..10000u64 {
            let is_buy = i % 2 == 0;
            let price = if is_buy { 900 } else { 1100 };
            let order = BenchOrder::new(i, is_buy, price, 100);
            let (_, instructions) = ob.eval_insert(order);
            ob.apply(instructions);
        }
        let mut id = 10000u64;

        b.iter(|| {
            // Add an order then cancel it
            let order = BenchOrder::new(id, true, 900, 100);
            let (_, instructions) = ob.eval_insert(order);
            ob.apply(instructions);

            let cancel_instr = ob.eval_cancel(black_box(id));
            ob.apply(vec![cancel_instr]);

            id += 1;
        });
    });

    group.finish();
}

/// Benchmark realistic mixed workload
fn bench_mixed_workload(c: &mut Criterion) {
    let mut group = c.benchmark_group("mixed");
    group.throughput(Throughput::Elements(1000));

    group.bench_function("realistic_1000_ops", |b| {
        let mut rng = StdRng::seed_from_u64(42);

        b.iter(|| {
            let mut ob = OrderBook::<BenchOrder>::default();
            let mut next_id = 0u64;
            let mut active_ids: Vec<u64> = Vec::with_capacity(1000);

            for _ in 0..1000 {
                let op_type = rng.gen_range(0..100);

                if op_type < 60 {
                    // 60% - Insert (may or may not match)
                    let is_buy = rng.gen_bool(0.5);
                    // Price around 1000, spread of ±50
                    let price = if is_buy {
                        rng.gen_range(950..1010)
                    } else {
                        rng.gen_range(990..1050)
                    };
                    let quantity = rng.gen_range(10..200);

                    let order = BenchOrder::new(next_id, is_buy, price, quantity);
                    let (_, instructions) = ob.eval_insert(black_box(order));
                    ob.apply(instructions);
                    active_ids.push(next_id);
                    next_id += 1;
                } else if op_type < 90 && !active_ids.is_empty() {
                    // 30% - Cancel (if we have active orders)
                    let idx = rng.gen_range(0..active_ids.len());
                    let id = active_ids.swap_remove(idx);
                    let cancel_instr = ob.eval_cancel(black_box(id));
                    ob.apply(vec![cancel_instr]);
                } else {
                    // 10% - Aggressive order (guaranteed to match if possible)
                    let is_buy = rng.gen_bool(0.5);
                    let price = if is_buy { 1100 } else { 900 }; // Cross the spread
                    let quantity = rng.gen_range(50..500);

                    let order = BenchOrder::new(next_id, is_buy, price, quantity);
                    let (_, instructions) = ob.eval_insert(black_box(order));
                    ob.apply(instructions);
                    next_id += 1;
                }
            }
        });
    });

    group.finish();
}

/// Benchmark with pre-populated order book (more realistic scenario)
fn bench_with_depth(c: &mut Criterion) {
    let mut group = c.benchmark_group("with_depth");
    group.throughput(Throughput::Elements(1));

    // Test insert performance with varying book depth
    for depth in [100, 1000, 10000] {
        group.bench_function(format!("insert_depth_{}", depth), |b| {
            let mut ob = OrderBook::<BenchOrder>::default();

            // Pre-populate order book with `depth` orders on each side
            for i in 0..depth {
                let buy = BenchOrder::new(i as u64, true, 900 + (i % 50) as u64, 100);
                let (_, instructions) = ob.eval_insert(buy);
                ob.apply(instructions);

                let sell = BenchOrder::new((i + depth) as u64, false, 1100 + (i % 50) as u64, 100);
                let (_, instructions) = ob.eval_insert(sell);
                ob.apply(instructions);
            }

            let mut id = (depth * 2) as u64;

            b.iter(|| {
                let order = BenchOrder::new(id, true, 895, 100); // Won't match
                let (_, instructions) = ob.eval_insert(black_box(order));
                ob.apply(instructions);
                id += 1;
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_insert,
    bench_match,
    bench_cancel,
    bench_mixed_workload,
    bench_with_depth,
);
criterion_main!(benches);
