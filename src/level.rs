use crate::list::{Iter, IterMut, List};
use crate::order::{Id, Num, OrderInterface};
use std::marker::PhantomData;

/// Represents a price level in the orderbook
/// A level contains all orders at a specific price point
pub struct Level<T: Id, N: Num, O: OrderInterface<T, N>> {
    /// The price of this level
    price: N,
    /// List of orders at this price level
    orders: List<O>,
    /// Total quantity across all orders at this level (cached for performance)
    total_quantity: N,
    /// Phantom data to mark the identifier type T as part of this struct's type
    _phantom: PhantomData<T>,
}

impl<T: Id, N: Num, O: OrderInterface<T, N>> Level<T, N, O> {
    /// Creates a new empty level at the specified price
    pub fn new(price: N) -> Self {
        Level {
            price,
            orders: List::new(),
            total_quantity: N::default(),
            _phantom: PhantomData,
        }
    }

    /// Returns the price of this level
    pub fn price(&self) -> &N {
        &self.price
    }

    /// Returns the total quantity of all orders at this level
    pub fn total_quantity(&self) -> &N {
        &self.total_quantity
    }

    /// Returns the number of orders at this level
    pub fn order_count(&self) -> usize {
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
        self.total_quantity += *order.remaining();
        self.orders.push_back(order)
    }

    /// Fills an order and returns true if the order is fully filled
    /// and should be removed from the level
    pub fn fill_order(
        &mut self,
        node_ptr: *mut crate::list::Node<O>,
        order: &mut O,
        fill: N,
    ) -> bool {
        order.fill(fill);
        self.total_quantity -= fill;

        if *order.remaining() == N::default() {
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
            self.total_quantity -= *order.remaining();
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
