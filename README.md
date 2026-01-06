# ob

A single threaded price-time priority orderbook implementation in Rust.

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

#### Insert Operations

| Benchmark | Eval | Apply |
|-----------|------|-------|
| Empty book | 24.7 ns (40.5 M/s) | 93.8 ns (10.7 M/s) |
| Depth 1,000 | 46.3 ns (21.6 M/s) | 74.2 ns (13.5 M/s) |
| Depth 10,000 | 61.4 ns (16.3 M/s) | 91.0 ns (11.0 M/s) |

#### Cancel Operations

| Benchmark | Eval | Apply |
|-----------|------|-------|
| Depth 1,000 | 63.3 ns (15.8 M/s) | 177.9 ns (5.6 M/s) |
| Depth 10,000 | 72.4 ns (13.8 M/s) | 303.3 ns (3.3 M/s) |

#### Match Operations

| Benchmark | Eval | Apply |
|-----------|------|-------|
| 1 level | 83.8 ns (11.9 M/s) | 80.1 ns (12.5 M/s) |
| 10 levels | 609.2 ns (1.6 M/s) | 584.6 ns (1.7 M/s) |
| 100 levels | 4.62 µs (216 K/s) | 7.44 µs (134 K/s) |

*Measured on Apple Silicon M4 Max.*

## License

MIT

