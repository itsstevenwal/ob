use std::{
    fmt::Display,
    hash::Hash,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign},
};

/// Trait defining the interface for orders in the orderbook
/// This allows for different order implementations while maintaining a common interface
///
/// T: The type of the order identifier. Needs to be unique.
/// N: The numeric type used in the orderbook. Needs to be a number.
pub trait OrderInterface {
    type T: Eq + Display + Default + Hash + Clone;
    type N: Ord
        + Eq
        + Copy
        + Default
        + Display
        + Add<Output = Self::N>
        + Sub<Output = Self::N>
        + Mul<Output = Self::N>
        + Div<Output = Self::N>
        + AddAssign
        + SubAssign
        + MulAssign
        + DivAssign;

    fn id(&self) -> &Self::T;

    fn is_buy(&self) -> bool;

    fn price(&self) -> Self::N;

    /// Returns the original quantity of this order
    /// Not updated when the order is filled.
    fn quantity(&self) -> Self::N;

    /// Returns the remaining quantity of this order
    /// Updated when the order is filled.
    fn remaining(&self) -> Self::N;

    /// Fills the order with the specified quantity
    /// Updates the remaining quantity.
    fn fill(&mut self, quantity: Self::N);
}

#[cfg(test)]
#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct TestOrder {
    id: String,
    is_buy: bool,
    price: u64,
    quantity: u64,
    remaining: u64,
}

#[cfg(test)]
impl TestOrder {
    pub fn new(id: &str, is_buy: bool, price: u64, quantity: u64) -> Self {
        Self {
            id: id.to_string(),
            is_buy,
            price,
            quantity,
            remaining: quantity,
        }
    }
}

#[cfg(test)]
impl OrderInterface for TestOrder {
    type T = String;
    type N = u64;

    fn id(&self) -> &String {
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
