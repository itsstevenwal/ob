use crate::list::{Iter, IterMut, List};
use crate::order::OrderInterface;
use std::marker::PhantomData;

/// Represents a price level in the orderbook.
/// A level contains all orders at a specific price point.
pub struct Level<O: OrderInterface> {
    price: O::N,
    orders: List<O>,

    /// Total quantity across all orders at this level.
    /// Cached for performance.
    total_quantity: O::N,

    /// Total volume (price * quantity) across all orders at this level.
    /// Cached for performance.
    total_volume: O::N,

    /// Phantom data to mark the identifier type T as part of this struct's type
    _phantom: PhantomData<O::T>,
}

impl<O: OrderInterface> Level<O> {
    pub fn new(price: O::N) -> Self {
        Level {
            price,
            orders: List::new(),
            total_quantity: O::N::default(),
            total_volume: O::N::default(),
            _phantom: PhantomData,
        }
    }

    /// Returns the price of this level
    pub fn price(&self) -> O::N {
        self.price
    }

    /// Returns the total quantity of all orders at this level
    pub fn total_quantity(&self) -> O::N {
        self.total_quantity
    }

    /// Returns the total volume of all orders at this level
    pub fn total_volume(&self) -> O::N {
        self.total_volume
    }

    /// Returns the number of orders at this level
    pub fn len(&self) -> usize {
        self.orders.len()
    }

    /// Returns true if this level has no orders
    pub fn is_empty(&self) -> bool {
        self.orders.is_empty()
    }

    /// Adds an order to this level
    /// Orders are added to the back (FIFO order)
    /// Returns a pointer to the newly inserted node
    pub fn add_order(&mut self, order: O) -> *mut crate::list::Node<O> {
        self.total_quantity += order.remaining();
        self.orders.push_back(order)
    }

    /// Fills an order and returns true if the order is fully filled
    pub fn fill_order(
        &mut self,
        node_ptr: *mut crate::list::Node<O>,
        order: &mut O,
        fill: O::N,
    ) -> bool {
        order.fill(fill);
        self.total_quantity -= fill;

        if order.remaining() == O::N::default() {
            self.orders.remove(node_ptr);
            return true;
        }

        false
    }

    /// Removes an order by its pointer
    /// Returns the removed order if found, None otherwise
    pub fn remove_order(&mut self, node_ptr: *mut crate::list::Node<O>) {
        let removed = self.orders.remove(node_ptr);
        if let Some(ref order) = removed {
            self.total_quantity -= order.remaining();
        }
    }

    /// Returns an iterator over the orders at this level
    pub fn iter(&self) -> Iter<'_, O> {
        self.orders.iter()
    }

    /// Returns a mutable iterator over the orders at this level
    pub fn iter_mut(&mut self) -> IterMut<'_, O> {
        self.orders.iter_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::order::TestOrder;

    #[test]
    fn test_level_basics() {
        let level = Level::<TestOrder>::new(100);
        assert_eq!(level.price(), 100);
        assert_eq!(level.total_quantity(), 0);
        assert_eq!(level.total_volume(), 0);
        assert!(level.is_empty());
    }

    #[test]
    fn test_add_orders() {
        let mut level = Level::<TestOrder>::new(100);

        level.add_order(TestOrder::new("1", true, 100, 50));
        assert_eq!(level.total_quantity(), 50);
        assert_eq!(level.len(), 1);

        level.add_order(TestOrder::new("2", true, 100, 30));
        level.add_order(TestOrder::new("3", true, 100, 20));
        assert_eq!(level.total_quantity(), 100);
        assert_eq!(level.len(), 3);
    }

    #[test]
    fn test_remove_order() {
        let mut level = Level::<TestOrder>::new(100);
        level.add_order(TestOrder::new("1", true, 100, 50));
        let node_ptr = level.add_order(TestOrder::new("2", true, 100, 30));
        level.add_order(TestOrder::new("3", true, 100, 20));

        level.remove_order(node_ptr);
        assert_eq!(level.total_quantity(), 70);
        assert_eq!(level.len(), 2);

        // Null pointer does nothing
        level.remove_order(std::ptr::null_mut());
        assert_eq!(level.total_quantity(), 70);
    }
}
