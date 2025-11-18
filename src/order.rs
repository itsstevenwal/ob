/// Trait defining the interface for orders in the orderbook
/// This allows for different order implementations while maintaining a common interface
pub trait OrderInterface {
    /// Returns the unique identifier for this order
    fn id(&self) -> &str;

    /// Returns the side of this order
    fn is_buy(&self) -> bool;

    /// Returns the quantity of this order
    fn quantity(&self) -> u64;

    // Returns the remaining quantity of this order
    fn remaining(&self) -> u64;

    /// Fills the order with the specified quantity
    fn fill(&mut self, quantity: u64);
}

/// A basic order implementation
#[cfg(test)]
#[derive(Default, Debug, PartialEq, Eq)]
pub struct BasicOrder {
    id: String,
    is_buy: bool,
    quantity: u64,
    filled: u64,
}

#[cfg(test)]
impl BasicOrder {
    pub fn new(id: &str, is_buy: bool, quantity: u64) -> Self {
        Self {
            id: id.to_string(),
            is_buy,
            quantity,
            filled: 0,
        }
    }
}

#[cfg(test)]
impl OrderInterface for BasicOrder {
    fn id(&self) -> &str {
        &self.id
    }

    fn is_buy(&self) -> bool {
        self.is_buy
    }

    fn quantity(&self) -> u64 {
        self.quantity
    }

    fn remaining(&self) -> u64 {
        self.quantity - self.filled
    }

    fn fill(&mut self, quantity: u64) {
        self.filled += quantity;
    }
}
