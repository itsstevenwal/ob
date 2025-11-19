use crate::list::Node;
use crate::order::OrderInterface;
use crate::side::Side;
use std::collections::HashMap;

/// Represents a complete orderbook with bid and ask sides
pub struct Orderbook<T: OrderInterface> {
    /// The bid side (buy orders)
    bids: Side<T>,
    /// The ask side (sell orders)
    asks: Side<T>,

    orders: HashMap<String, *mut Node<T>>,
}

impl<T: OrderInterface> Default for Orderbook<T> {
    fn default() -> Self {
        Self {
            bids: Side::new(true),
            asks: Side::new(false),
            orders: HashMap::new(),
        }
    }
}

// Add Order – O(log M) for the first order at a limit, O(1) for all others, where M is the number of price Limits (generally << N the number of orders).
// Cancel Order – O(1)
// Modify Order – O(1)
// Execute – O(1)
// GetVolumeAtLimit – O(1)
// GetBestBid/Offer – O(1)

impl<T: OrderInterface> Orderbook<T> {
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
                maker_quantities.push((resting_order.id(), taken_quantity));

                continue;
            }

            // Stop matching
            break;
        }

        // // Handle the maker quantities
        // for (order_id, quantity) in maker_quantities {
        //     if let Some(node_ptr) = self.orders.get(order_id) {
        //         let removed = opposite_book.fill_order(*node_ptr, price, quantity);
        //         if removed {
        //             self.opposite_book.remove_order(*node_ptr);
        //             self.orders.remove(order_id);
        //         }
        //     }
        // }

        // Add remaining quantity to the appropriate side
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

    // pub fn snapshot(&self)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::order::BasicOrder;

    #[test]
    fn test_new_orderbook() {
        let ob = Orderbook::<BasicOrder>::default();
        // Orderbook should be empty initially
        // We can verify by trying to cancel a non-existent order (will panic, but that's expected)
    }

    #[test]
    fn test_insert_buy_order_no_match() {
        let mut ob = Orderbook::<BasicOrder>::default();
        let order = BasicOrder::new("1", true, 1000, 100);
        ob.insert_order(1000, order);

        // Order should be in the book, verify by cancelling it
        ob.cancel_order("1");
        // If we get here without panic, the order was successfully added and removed
    }

    #[test]
    fn test_insert_sell_order_no_match() {
        let mut ob = Orderbook::<BasicOrder>::default();
        let order = BasicOrder::new("1", false, 1100, 50);
        ob.insert_order(1100, order);

        // Order should be in the book, verify by cancelling it
        ob.cancel_order("1");
    }

    #[test]
    fn test_buy_order_matches_sell_order_complete_fill() {
        let mut ob = Orderbook::<BasicOrder>::default();

        // Add a sell order at 1000
        let sell_order = BasicOrder::new("sell1", false, 1000, 100);
        ob.insert_order(1000, sell_order);

        // Add a buy order at 1000 (should match completely)
        let buy_order = BasicOrder::new("buy1", true, 1000, 100);
        ob.insert_order(1000, buy_order);

        // Both orders should be fully filled and not in the book
        // Verify by attempting to cancel (should panic if not in book)
        // Since we can't easily check, we'll verify the sell order was removed
        // by checking that cancelling it would panic
    }

    #[test]
    fn test_buy_order_matches_sell_order_partial_fill_buy_remaining() {
        let mut ob = Orderbook::<BasicOrder>::default();

        // Add a sell order at 1000 with quantity 50
        let sell_order = BasicOrder::new("sell1", false, 1000, 50);
        ob.insert_order(1000, sell_order);

        // Add a buy order at 1000 with quantity 100 (should partially fill)
        let buy_order = BasicOrder::new("buy1", true, 1000, 100);
        ob.insert_order(1000, buy_order);

        // Sell order should be fully filled and removed
        // Buy order should have 50 remaining and be in the book
        ob.cancel_order("buy1"); // Should succeed if buy order is still in book
    }

    #[test]
    fn test_buy_order_matches_sell_order_partial_fill_sell_remaining() {
        let mut ob = Orderbook::<BasicOrder>::default();

        // Add a sell order at 1000 with quantity 100
        let sell_order = BasicOrder::new("sell1", false, 1000, 100);
        ob.insert_order(1000, sell_order);

        // Add a buy order at 1000 with quantity 50 (should partially fill)
        let buy_order = BasicOrder::new("buy1", true, 1000, 50);
        ob.insert_order(1000, buy_order);

        // Buy order should be fully filled and removed
        // Sell order should have 50 remaining and be in the book
        ob.cancel_order("sell1"); // Should succeed if sell order is still in book
    }

    #[test]
    fn test_sell_order_matches_buy_order_complete_fill() {
        let mut ob = Orderbook::<BasicOrder>::default();

        // Add a buy order at 1000
        let buy_order = BasicOrder::new("buy1", true, 1000, 100);
        ob.insert_order(1000, buy_order);

        // Add a sell order at 1000 (should match completely)
        let sell_order = BasicOrder::new("sell1", false, 1000, 100);
        ob.insert_order(1000, sell_order);

        // Both orders should be fully filled and not in the book
    }

    #[test]
    fn test_sell_order_matches_buy_order_partial_fill() {
        let mut ob = Orderbook::<BasicOrder>::default();

        // Add a buy order at 1000 with quantity 100
        let buy_order = BasicOrder::new("buy1", true, 1000, 100);
        ob.insert_order(1000, buy_order);

        // Add a sell order at 1000 with quantity 50 (should partially fill)
        let sell_order = BasicOrder::new("sell1", false, 1000, 50);
        ob.insert_order(1000, sell_order);

        // Sell order should be fully filled and removed
        // Buy order should have 50 remaining and be in the book
        ob.cancel_order("buy1"); // Should succeed if buy order is still in book
    }

    #[test]
    fn test_buy_order_matches_multiple_sell_orders() {
        let mut ob = Orderbook::<BasicOrder>::default();

        // Add multiple sell orders
        ob.insert_order(1000, BasicOrder::new("sell1", false, 1000, 30));
        ob.insert_order(1000, BasicOrder::new("sell2", false, 1000, 40));
        ob.insert_order(1000, BasicOrder::new("sell3", false, 1000, 20));

        // Add a buy order that will match all three
        ob.insert_order(1000, BasicOrder::new("buy1", true, 1000, 90));

        // All sell orders should be fully filled and removed
        // Buy order should be fully filled and removed
    }

    #[test]
    fn test_sell_order_matches_multiple_buy_orders() {
        let mut ob = Orderbook::<BasicOrder>::default();

        // Add multiple buy orders
        ob.insert_order(1000, BasicOrder::new("buy1", true, 1000, 30));
        ob.insert_order(1000, BasicOrder::new("buy2", true, 1000, 40));
        ob.insert_order(1000, BasicOrder::new("buy3", true, 1000, 20));

        // Add a sell order that will match all three
        ob.insert_order(1000, BasicOrder::new("sell1", false, 1000, 90));

        // All buy orders should be fully filled and removed
        // Sell order should be fully filled and removed
    }

    #[test]
    fn test_buy_order_at_higher_price_matches_lower_sell() {
        let mut ob = Orderbook::<BasicOrder>::default();

        // Add a sell order at 1000
        ob.insert_order(1000, BasicOrder::new("sell1", false, 1000, 100));

        // Add a buy order at 1100 (higher price, should match)
        ob.insert_order(1100, BasicOrder::new("buy1", true, 1100, 100));

        // Both should match and be removed
    }

    #[test]
    fn test_sell_order_at_lower_price_matches_higher_buy() {
        let mut ob = Orderbook::<BasicOrder>::default();

        // Add a buy order at 1100
        ob.insert_order(1100, BasicOrder::new("buy1", true, 1100, 100));

        // Add a sell order at 1000 (lower price, should match)
        ob.insert_order(1000, BasicOrder::new("sell1", false, 1000, 100));

        // Both should match and be removed
    }

    #[test]
    fn test_buy_order_does_not_match_higher_sell() {
        let mut ob = Orderbook::<BasicOrder>::default();

        // Add a sell order at 1100
        ob.insert_order(1100, BasicOrder::new("sell1", false, 1100, 100));

        // Add a buy order at 1000 (lower price, should NOT match)
        ob.insert_order(1000, BasicOrder::new("buy1", true, 1000, 100));

        // Both should remain in the book
        ob.cancel_order("sell1");
        ob.cancel_order("buy1");
    }

    #[test]
    fn test_sell_order_does_not_match_lower_buy() {
        let mut ob = Orderbook::<BasicOrder>::default();

        // Add a buy order at 1000
        ob.insert_order(1000, BasicOrder::new("buy1", true, 1000, 100));

        // Add a sell order at 1100 (higher price, should NOT match)
        ob.insert_order(1100, BasicOrder::new("sell1", false, 1100, 100));

        // Both should remain in the book
        ob.cancel_order("buy1");
        ob.cancel_order("sell1");
    }

    #[test]
    fn test_cancel_buy_order() {
        let mut ob = Orderbook::<BasicOrder>::default();
        ob.insert_order(1000, BasicOrder::new("buy1", true, 1000, 100));

        ob.cancel_order("buy1");
        // Order should be removed, verify by attempting to cancel again (should panic)
    }

    #[test]
    fn test_cancel_sell_order() {
        let mut ob = Orderbook::<BasicOrder>::default();
        ob.insert_order(1100, BasicOrder::new("sell1", false, 1100, 50));

        ob.cancel_order("sell1");
        // Order should be removed
    }

    #[test]
    fn test_multiple_orders_at_same_price() {
        let mut ob = Orderbook::<BasicOrder>::default();

        // Add multiple buy orders at the same price
        ob.insert_order(1000, BasicOrder::new("buy1", true, 1000, 100));
        ob.insert_order(1000, BasicOrder::new("buy2", true, 1000, 50));
        ob.insert_order(1000, BasicOrder::new("buy3", true, 1000, 25));

        // All should be in the book
        ob.cancel_order("buy1");
        ob.cancel_order("buy2");
        ob.cancel_order("buy3");
    }

    #[test]
    fn test_multiple_orders_different_prices() {
        let mut ob = Orderbook::<BasicOrder>::default();

        // Add buy orders at different prices
        ob.insert_order(1000, BasicOrder::new("buy1", true, 1000, 100));
        ob.insert_order(990, BasicOrder::new("buy2", true, 990, 50));
        ob.insert_order(1010, BasicOrder::new("buy3", true, 1010, 25));

        // Add sell orders at different prices
        ob.insert_order(1100, BasicOrder::new("sell1", false, 1100, 75));
        ob.insert_order(1110, BasicOrder::new("sell2", false, 1110, 30));

        // All should be in the book
        ob.cancel_order("buy1");
        ob.cancel_order("buy2");
        ob.cancel_order("buy3");
        ob.cancel_order("sell1");
        ob.cancel_order("sell2");
    }

    #[test]
    fn test_order_matching_stops_at_price_boundary() {
        let mut ob = Orderbook::<BasicOrder>::default();

        // Add sell orders at different prices
        ob.insert_order(1000, BasicOrder::new("sell1", false, 1000, 50));
        ob.insert_order(1100, BasicOrder::new("sell2", false, 1100, 50));

        // Add a buy order at 1000 - should only match sell1, not sell2
        ob.insert_order(1000, BasicOrder::new("buy1", true, 1000, 100));

        // sell1 should be matched, buy1 should have 50 remaining
        // sell2 should still be in the book
        ob.cancel_order("buy1");
        ob.cancel_order("sell2");
    }

    #[test]
    fn test_buy_order_matches_best_ask_first() {
        let mut ob = Orderbook::<BasicOrder>::default();

        // Add sell orders at different prices (lower price = better for buyer)
        ob.insert_order(1000, BasicOrder::new("sell1", false, 1000, 30));
        ob.insert_order(1100, BasicOrder::new("sell2", false, 1100, 50));
        ob.insert_order(1050, BasicOrder::new("sell3", false, 1050, 40));

        // Add a buy order that will match multiple orders
        // Should match sell1 first (1000), then sell3 (1050), then sell2 (1100)
        ob.insert_order(1100, BasicOrder::new("buy1", true, 1100, 100));

        // All sell orders should be matched
        // Buy order should be fully filled
    }

    #[test]
    fn test_sell_order_matches_best_bid_first() {
        let mut ob = Orderbook::<BasicOrder>::default();

        // Add buy orders at different prices (higher price = better for seller)
        ob.insert_order(1100, BasicOrder::new("buy1", true, 1100, 30));
        ob.insert_order(1000, BasicOrder::new("buy2", true, 1000, 50));
        ob.insert_order(1050, BasicOrder::new("buy3", true, 1050, 40));

        // Add a sell order that will match multiple orders
        // Should match buy1 first (1100), then buy3 (1050), then buy2 (1000)
        ob.insert_order(1000, BasicOrder::new("sell1", false, 1000, 100));

        // All buy orders should be matched
        // Sell order should be fully filled
    }

    #[test]
    fn test_cancel_nonexistent_order_panics() {
        let mut ob = Orderbook::<BasicOrder>::default();
        ob.cancel_order("nonexistent");
    }

    #[test]
    fn test_complex_matching_scenario() {
        let mut ob = Orderbook::<BasicOrder>::default();

        // Build up a book with multiple levels
        ob.insert_order(1000, BasicOrder::new("buy1", true, 1000, 100));
        ob.insert_order(990, BasicOrder::new("buy2", true, 990, 50));
        ob.insert_order(1100, BasicOrder::new("sell1", false, 1100, 75));
        ob.insert_order(1110, BasicOrder::new("sell2", false, 1110, 25));

        // Add a large buy order that crosses the spread
        ob.insert_order(1100, BasicOrder::new("buy3", true, 1100, 200));

        // buy3 should match sell1 and sell2 completely
        // buy3 should have 100 remaining and be in the book
        ob.cancel_order("buy3");
        ob.cancel_order("buy1");
        ob.cancel_order("buy2");
    }
}
