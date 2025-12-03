use crate::list::Node;
use crate::order::OrderInterface;
use crate::side::Side;
use std::collections::HashMap;

/// Represents a complete orderbook with bid and ask sides
pub struct OrderBook<T: OrderInterface> {
    /// The bid side (buy orders)
    bids: Side<T>,
    /// The ask side (sell orders)
    asks: Side<T>,

    // All orders in the orderbook
    orders: HashMap<String, *mut Node<T>>,
}

impl<T: OrderInterface> Default for OrderBook<T> {
    fn default() -> Self {
        Self {
            bids: Side::new(true),
            asks: Side::new(false),
            orders: HashMap::new(),
        }
    }
}

impl<T: OrderInterface> OrderBook<T> {
    /// Inserts an order into the orderbook at the specified price
    /// Iterates through the btree on the opposite side to check for matches
    /// Matches orders and executes trades when prices cross
    /// Any remaining quantity is added to the appropriate side
    pub fn insert_order(&mut self, price: u64, mut order: T) {
        let mut remaining_quantity = order.remaining();
        let mut taker_quantity = 0;
        let mut maker_quantities = Vec::new();
        let is_buy = order.is_buy();

        let check_fn = if is_buy {
            // For a buy order, stop matching when the price is less than the resting order price
            |price: u64, resting_order: &T| price >= resting_order.price()
        } else {
            // For a sell order, stop matching when the price is greater than the resting order price
            |price: u64, resting_order: &T| price <= resting_order.price()
        };

        // Match against the opposite side and collect orders to remove

        let opposite_book = if is_buy {
            &mut self.asks
        } else {
            &mut self.bids
        };

        for resting_order in opposite_book.iter_mut() {
            if check_fn(price, resting_order) && remaining_quantity > 0 {
                let taken_quantity = remaining_quantity.min(resting_order.remaining());
                remaining_quantity -= taken_quantity;

                taker_quantity += taken_quantity;
                maker_quantities.push((resting_order.id().to_string(), taken_quantity));
            } else {
                break;
            }
        }

        // Handle the maker quantities
        for (order_id, quantity) in maker_quantities {
            println!("filling maker order {} quantity {}", order_id, quantity);
            if let Some(node_ptr) = self.orders.get(&order_id) {
                let removed = opposite_book.fill_order(*node_ptr, quantity);
                if removed {
                    self.orders.remove(&order_id);
                }
            }
        }

        // Handle the taker quantity
        if remaining_quantity > 0 {
            order.fill(taker_quantity);
            let id = order.id().to_string();
            let node_ptr = if is_buy {
                self.bids.insert_order(order)
            } else {
                self.asks.insert_order(order)
            };
            self.orders.insert(id, node_ptr);
        }
    }

    /// Cancels an order by its ID
    /// Returns the cancelled order if found, None otherwise
    /// Uses the node pointer stored in the orders map for O(1) cancellation
    pub fn cancel_order(&mut self, order_id: &str) {
        // Get the node pointer from the orders map
        let node_ptr = if let Some(&ptr) = self.orders.get(order_id) {
            ptr
        } else {
            return; // Order not found, nothing to cancel
        };

        // Determine which side the order is on
        let is_buy = unsafe { (*node_ptr).data.is_buy() };

        // Remove from the side (the data has been forgotten, so this is safe)
        if is_buy {
            self.bids.remove_order(node_ptr);
        } else {
            self.asks.remove_order(node_ptr);
        }

        // Remove from orders map
        self.orders.remove(order_id);
    }
}
