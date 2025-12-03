use std::{
    fmt::Display,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign},
};

pub trait Id: Eq + Display + Default {}

pub trait Num:
    Ord
    + Eq
    + Copy
    + Default
    + Display
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + AddAssign<Self>
    + SubAssign<Self>
    + MulAssign<Self>
    + DivAssign<Self>
{
}

/// Trait defining the interface for orders in the orderbook
/// This allows for different order implementations while maintaining a common interface
pub trait OrderInterface<T: Id, N: Num> {
    /// Returns the unique identifier for this order
    fn id<'a>(&'a self) -> &'a T;

    /// Returns the side of this order
    fn is_buy(&self) -> bool;

    /// Returns the price of this order
    fn price(&self) -> &N;

    /// Returns the quantity of this order
    fn quantity(&self) -> &N;

    // Returns the remaining quantity of this order
    fn remaining(&self) -> &N;

    /// Fills the order with the specified quantity
    fn fill(&mut self, quantity: N);
}
