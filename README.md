# ob

A high-performance, price-time priority orderbook implementation in Rust.

## Features

- **Bid/Ask Sides** — Maintains separate bid (buy) and ask (sell) order books
- **Price-Time Priority** — Orders are matched by best price first, then by arrival time
- **Eval/Apply Pattern** — Separates order evaluation from state mutation for flexibility
- **Zero Dependencies** — No runtime dependencies, only dev dependencies for testing and benchmarking

## Usage

Implement the `OrderInterface` trait for your order type:

```rust
use ob::{OrderInterface, ob::OrderBook, ob::Op};

#[derive(Clone, Debug, PartialEq, Eq)]
struct MyOrder {
    id: u64,
    is_buy: bool,
    price: u64,
    quantity: u64,
    remaining: u64,
}

impl OrderInterface for MyOrder {
    type T = u64;  // Order ID type
    type N = u64;  // Quantity type

    fn id(&self) -> &u64 { &self.id }
    fn price(&self) -> u64 { self.price }
    fn is_buy(&self) -> bool { self.is_buy }
    fn quantity(&self) -> u64 { self.quantity }
    fn remaining(&self) -> u64 { self.remaining }
    fn fill(&mut self, quantity: u64) { self.remaining -= quantity; }
}

// Create orderbook and process orders
let mut ob = OrderBook::<MyOrder>::default();
let (matches, instructions) = ob.eval(vec![Op::Insert(order)]);
ob.apply(instructions);
```

## Run Tests

```bash
cargo test
```

## Run Benchmark

```bash
cargo bench
```

### Benchmark Results

| Benchmark | Time | Throughput |
|-----------|------|------------|
| `insert/insert_no_match` | 79.26 ns | 12.6 M ops/s |
| `match/match_single` | 137.02 ns | 7.3 M ops/s |
| `match/match_multi_maker` | 589.15 ns | 1.7 M ops/s |
| `cancel/cancel` | 96.67 ns | 10.3 M ops/s |
| `mixed/realistic_1000_ops` | 151.98 µs | 6.6 M ops/s |
| `with_depth/insert_depth_100` | 74.37 ns | 13.4 M ops/s |
| `with_depth/insert_depth_1000` | 73.19 ns | 13.7 M ops/s |
| `with_depth/insert_depth_10000` | 80.40 ns | 12.4 M ops/s |

*Measured on Apple Silicon. Results may vary.*

## License

MIT

