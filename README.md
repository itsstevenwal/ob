# ob

A single threaded, zero dependency price-time priority orderbook implementation in Rust.

## design

The orderbook separates state evaluation from state mutation:

- **`eval`**: evaluates operations against current state, returns matches and instructions without modifying the book
- **`apply`**: takes instructions from eval and commits them to the book

## benchmark

| Operation | Benchmark | Eval | Apply | Total* |
|-----------|-----------|------|-------|--------|
| Insert | Empty book | 24.7 ns (40.5 M/s) | 93.8 ns (10.7 M/s) | 118.5 ns (8.4 M/s) |
| Insert | Depth 1,000 | 46.3 ns (21.6 M/s) | 74.2 ns (13.5 M/s) | 120.5 ns (8.3 M/s) |
| Insert | Depth 10,000 | 61.4 ns (16.3 M/s) | 91.0 ns (11.0 M/s) | 152.4 ns (6.6 M/s) |
| Cancel | Depth 1,000 | 63.3 ns (15.8 M/s) | 177.9 ns (5.6 M/s) | 241.2 ns (4.1 M/s) |
| Cancel | Depth 10,000 | 72.4 ns (13.8 M/s) | 303.3 ns (3.3 M/s) | 375.7 ns (2.7 M/s) |
| Match | 1 level | 83.8 ns (11.9 M/s) | 80.1 ns (12.5 M/s) | 163.9 ns (6.1 M/s) |
| Match | 10 levels | 609.2 ns (1.6 M/s) | 584.6 ns (1.7 M/s) | 1.19 µs (838 K/s) |
| Match | 100 levels | 4.62 µs (216 K/s) | 7.44 µs (134 K/s) | 12.06 µs (83 K/s) |

*Total = Eval + Apply. Measured on Apple Silicon M4 Max.*

## usage

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

// Evaluate result on insertion
let (matches, instructions) = ob.eval(vec![Op::Insert(order)]);

// Apply state changes
ob.apply(instructions);
```

## license

MIT

