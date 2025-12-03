use std::{
    fmt::Display,
    hash::Hash,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign},
};

/// Trait definition for the order identifier.
/// This definition makes it easy to reference the trait for objects that use it.
pub(crate) trait Id: Eq + Display + Default + Hash + Clone {}

/// Trait definition for the numeric type used in the orderbook.
/// This definition makes it easy to reference the trait for objects that use it.
pub(crate) trait Num:
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
///
/// T: The type of the order identifier. Needs to be unique.
/// N: The numeric type used in the orderbook. Needs to be a number.
pub trait OrderInterface<T, N>
where
    T: Eq + Display + Default + Hash + Clone,
    N: Ord
        + Eq
        + Copy
        + Default
        + Display
        + Add<Output = N>
        + Sub<Output = N>
        + Mul<Output = N>
        + Div<Output = N>
        + AddAssign<N>
        + SubAssign<N>
        + MulAssign<N>
        + DivAssign<N>,
{
    fn id<'a>(&'a self) -> &'a T;

    fn is_buy(&self) -> bool;

    fn price(&self) -> N;

    /// Returns the original quantity of this order
    /// Not updated when the order is filled.
    fn quantity(&self) -> N;

    /// Returns the remaining quantity of this order
    /// Updated when the order is filled.
    fn remaining(&self) -> N;

    /// Fills the order with the specified quantity
    /// Updates the remaining quantity.
    fn fill(&mut self, quantity: N);
}
