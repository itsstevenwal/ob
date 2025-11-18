use crate::list::Node;
use crate::order::OrderInterface;
use crate::side::Side;
use std::collections::HashMap;

/// Represents a complete orderbook with bid and ask sides
#[derive(Default)]
pub struct Orderbook<T: OrderInterface> {
    /// The bid side (buy orders)
    bids: Side<T>,
    /// The ask side (sell orders)
    asks: Side<T>,

    orders: HashMap<String, *mut Node<T>>,
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

        if order.is_buy() {
            // Buy order: match against asks (sell orders) starting from lowest ask price
            // Match if buy_price >= ask_price
            while remaining_quantity > 0 {
                // Get the best (lowest) ask price
                if let Some((&ask_price, _)) = self.asks.worst_level() {
                    if price >= ask_price {
                        // Can match - process this level
                        if let Some(level) = self.asks.get_level_mut(ask_price) {
                            // Process orders at this level until we've filled the buy order
                            // or exhausted this level
                            while remaining_quantity > 0 && !level.is_empty() {
                                // Get the first order at this level (FIFO)
                                if let Some(mut resting_order) = level.remove_first_order() {
                                    let resting_quantity = resting_order.remaining();
                                    let resting_order_id = resting_order.id().to_string();

                                    if resting_quantity <= remaining_quantity {
                                        // Fully fill the resting order
                                        resting_order.fill(resting_quantity);
                                        order.fill(resting_quantity);
                                        remaining_quantity -= resting_quantity;

                                        // Remove from orders map
                                        self.orders.remove(&resting_order_id);
                                    } else {
                                        // Partially fill the resting order
                                        resting_order.fill(remaining_quantity);
                                        order.fill(remaining_quantity);

                                        // Put the partially filled order back
                                        let node_ptr = level.add_order(resting_order);
                                        self.orders.insert(resting_order_id, node_ptr);

                                        remaining_quantity = 0;
                                    }
                                } else {
                                    break;
                                }
                            }

                            // If the level is now empty, remove it
                            if level.is_empty() {
                                self.asks.remove_level(ask_price);
                            }
                        } else {
                            break;
                        }
                    } else {
                        // No more matches possible (buy_price < ask_price)
                        break;
                    }
                } else {
                    // No asks to match against
                    break;
                }
            }

            // Add any remaining quantity to bids
            if remaining_quantity > 0 {
                let order_id = order.id().to_string();
                let level = self.bids.get_or_create_level(price);
                let node_ptr = level.add_order(order);
                self.orders.insert(order_id, node_ptr);
            }
        } else {
            // Sell order: match against bids (buy orders) starting from highest bid price
            // Match if sell_price <= bid_price
            while remaining_quantity > 0 {
                // Get the best (highest) bid price
                if let Some((&bid_price, _)) = self.bids.best_level() {
                    if price <= bid_price {
                        // Can match - process this level
                        if let Some(level) = self.bids.get_level_mut(bid_price) {
                            // Process orders at this level until we've filled the sell order
                            // or exhausted this level
                            while remaining_quantity > 0 && !level.is_empty() {
                                // Get the first order at this level (FIFO)
                                if let Some(mut resting_order) = level.remove_first_order() {
                                    let resting_quantity = resting_order.remaining();
                                    let resting_order_id = resting_order.id().to_string();

                                    if resting_quantity <= remaining_quantity {
                                        // Fully fill the resting order
                                        resting_order.fill(resting_quantity);
                                        order.fill(resting_quantity);
                                        remaining_quantity -= resting_quantity;

                                        // Remove from orders map
                                        self.orders.remove(&resting_order_id);
                                    } else {
                                        // Partially fill the resting order
                                        resting_order.fill(remaining_quantity);
                                        order.fill(remaining_quantity);

                                        // Put the partially filled order back
                                        let node_ptr = level.add_order(resting_order);
                                        self.orders.insert(resting_order_id, node_ptr);

                                        remaining_quantity = 0;
                                    }
                                } else {
                                    break;
                                }
                            }

                            // If the level is now empty, remove it
                            if level.is_empty() {
                                self.bids.remove_level(bid_price);
                            }
                        } else {
                            break;
                        }
                    } else {
                        // No more matches possible (sell_price > bid_price)
                        break;
                    }
                } else {
                    // No bids to match against
                    break;
                }
            }

            // Add any remaining quantity to asks
            if remaining_quantity > 0 {
                let order_id = order.id().to_string();
                let level = self.asks.get_or_create_level(price);
                let node_ptr = level.add_order(order);
                self.orders.insert(order_id, node_ptr);
            }
        }
    }

    /// Cancels an order by its ID
    /// Returns the cancelled order if found, None otherwise
    /// Iterates through the btree to find and remove the order
    pub fn cancel_order(&mut self, order_id: &str) -> Option<T> {
        // Get the node pointer from the orders map
        let node_ptr = *self.orders.get(order_id)?;

        // Peek at the order to determine which side it's on
        // We need to determine if it's a buy or sell order to know which side to search
        let is_buy = unsafe {
            let order = &(*node_ptr).data;
            order.is_buy()
        };

        // Search through the appropriate side to find and remove the order
        let mut removed_order = None;
        let mut price_to_remove = None;

        if is_buy {
            // Search through bids
            for (price, level) in self.bids.iter_mut() {
                // Try to remove the order from this level
                if let Some(order) = level.remove_order(node_ptr) {
                    removed_order = Some(order);
                    // If the level is now empty, mark it for removal
                    if level.is_empty() {
                        price_to_remove = Some(*price);
                    }
                    break;
                }
            }
        } else {
            // Search through asks
            for (price, level) in self.asks.iter_mut() {
                // Try to remove the order from this level
                if let Some(order) = level.remove_order(node_ptr) {
                    removed_order = Some(order);
                    // If the level is now empty, mark it for removal
                    if level.is_empty() {
                        price_to_remove = Some(*price);
                    }
                    break;
                }
            }
        }

        // Remove empty level if needed
        if let Some(price) = price_to_remove {
            if is_buy {
                self.bids.remove_level(price);
            } else {
                self.asks.remove_level(price);
            }
        }

        // If successfully removed, also remove from orders map
        if removed_order.is_some() {
            self.orders.remove(order_id);
        }

        removed_order
    }

    // /// Gets a reference to the bid side
    // pub fn bids(&self) -> &Side<T> {
    //     &self.bids
    // }

    // /// Gets a mutable reference to the bid side
    // pub fn bids_mut(&mut self) -> &mut Side<T> {
    //     &mut self.bids
    // }

    // /// Gets a reference to the ask side
    // pub fn asks(&self) -> &Side<T> {
    //     &self.asks
    // }

    // /// Gets a mutable reference to the ask side
    // pub fn asks_mut(&mut self) -> &mut Side<T> {
    //     &mut self.asks
    // }

    // /// Gets the best bid price (highest bid)
    // /// Returns None if there are no bids
    // pub fn best_bid_price(&self) -> Option<u64> {
    //     self.bids.best_price()
    // }

    // /// Gets the best ask price (lowest ask)
    // /// Returns None if there are no asks
    // pub fn best_ask_price(&self) -> Option<u64> {
    //     self.asks.worst_price()
    // }

    // /// Gets the best bid level (highest bid)
    // /// Returns None if there are no bids
    // pub fn best_bid(&self) -> Option<(&u64, &Level<T>)> {
    //     self.bids.best_level()
    // }

    // /// Gets the best ask level (lowest ask)
    // /// Returns None if there are no asks
    // pub fn best_ask(&self) -> Option<(&u64, &Level<T>)> {
    //     self.asks.worst_level()
    // }

    // /// Gets the spread (difference between best ask and best bid)
    // /// Returns None if either side is empty
    // pub fn spread(&self) -> Option<u64> {
    //     let best_bid = self.best_bid_price()?;
    //     let best_ask = self.best_ask_price()?;
    //     Some(best_ask.saturating_sub(best_bid))
    // }

    // /// Adds a bid order at the specified price
    // /// Returns a pointer to the node containing the order
    // pub fn add_bid(&mut self, price: u64, order: T) -> *mut crate::list::Node<T> {
    //     let level = self.bids.get_or_create_level(price);
    //     level.add_order(order)
    // }

    // /// Adds an ask order at the specified price
    // /// Returns a pointer to the node containing the order
    // pub fn add_ask(&mut self, price: u64, order: T) -> *mut crate::list::Node<T> {
    //     let level = self.asks.get_or_create_level(price);
    //     level.add_order(order)
    // }

    // /// Removes a bid order by its node pointer
    // /// Returns the removed order if found, None otherwise
    // pub fn remove_bid(&mut self, price: u64, node_ptr: *mut crate::list::Node<T>) -> Option<T> {
    //     if let Some(level) = self.bids.get_level_mut(price) {
    //         let removed = level.remove_order(node_ptr);
    //         // If the level is now empty, remove it
    //         if level.is_empty() {
    //             self.bids.remove_level(price);
    //         }
    //         removed
    //     } else {
    //         None
    //     }
    // }

    // /// Removes an ask order by its node pointer
    // /// Returns the removed order if found, None otherwise
    // pub fn remove_ask(&mut self, price: u64, node_ptr: *mut crate::list::Node<T>) -> Option<T> {
    //     if let Some(level) = self.asks.get_level_mut(price) {
    //         let removed = level.remove_order(node_ptr);
    //         // If the level is now empty, remove it
    //         if level.is_empty() {
    //             self.asks.remove_level(price);
    //         }
    //         removed
    //     } else {
    //         None
    //     }
    // }

    // /// Gets the total quantity of all bids at a specific price level
    // pub fn bid_quantity_at(&self, price: u64) -> u64 {
    //     self.bids
    //         .get_level(price)
    //         .map(|level| level.total_quantity())
    //         .unwrap_or(0)
    // }

    // /// Gets the total quantity of all asks at a specific price level
    // pub fn ask_quantity_at(&self, price: u64) -> u64 {
    //     self.asks
    //         .get_level(price)
    //         .map(|level| level.total_quantity())
    //         .unwrap_or(0)
    // }

    // /// Returns true if the orderbook is empty (no bids and no asks)
    // pub fn is_empty(&self) -> bool {
    //     self.bids.is_empty() && self.asks.is_empty()
    // }

    // /// Returns the total number of price levels (bids + asks)
    // pub fn level_count(&self) -> usize {
    //     self.bids.level_count() + self.asks.level_count()
    // }

    // /// Clears all bids and asks from the orderbook
    // pub fn clear(&mut self) {
    //     self.bids.clear();
    //     self.asks.clear();
    // }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::order::BasicOrder;

//     #[test]
//     fn test_new_orderbook() {
//         let ob = Orderbook::<BasicOrder>::default();
//         assert!(ob.is_empty());
//         assert_eq!(ob.level_count(), 0);
//     }

//     #[test]
//     fn test_add_bid() {
//         let mut ob = Orderbook::<BasicOrder>::default();
//         let order = BasicOrder::new("1", true, 100);
//         ob.add_bid(1000, order);

//         assert_eq!(ob.bid_quantity_at(1000), 100);
//         assert_eq!(ob.best_bid_price(), Some(1000));
//     }

//     #[test]
//     fn test_add_ask() {
//         let mut ob = Orderbook::<BasicOrder>::default();
//         let order = BasicOrder::new("1", false, 50);
//         ob.add_ask(1100, order);

//         assert_eq!(ob.ask_quantity_at(1100), 50);
//         assert_eq!(ob.best_ask_price(), Some(1100));
//     }

//     #[test]
//     fn test_best_bid_and_ask() {
//         let mut ob = Orderbook::<BasicOrder>::default();
//         ob.add_bid(1000, BasicOrder::new("1", true, 100));
//         ob.add_bid(990, BasicOrder::new("2", true, 50));
//         ob.add_ask(1100, BasicOrder::new("3", false, 75));
//         ob.add_ask(1110, BasicOrder::new("4", false, 25));

//         assert_eq!(ob.best_bid_price(), Some(1000));
//         assert_eq!(ob.best_ask_price(), Some(1100));
//     }

//     #[test]
//     fn test_spread() {
//         let mut ob = Orderbook::<BasicOrder>::default();
//         ob.add_bid(1000, BasicOrder::new("1", true, 100));
//         ob.add_ask(1100, BasicOrder::new("2", false, 50));

//         assert_eq!(ob.spread(), Some(100));
//     }

//     #[test]
//     fn test_spread_none_when_empty() {
//         let ob = Orderbook::<BasicOrder>::default();
//         assert_eq!(ob.spread(), None);

//         let mut ob = Orderbook::<BasicOrder>::default();
//         ob.add_bid(1000, BasicOrder::new("1", true, 100));
//         assert_eq!(ob.spread(), None);

//         let mut ob = Orderbook::<BasicOrder>::default();
//         ob.add_ask(1100, BasicOrder::new("2", false, 50));
//         assert_eq!(ob.spread(), None);
//     }

//     #[test]
//     fn test_remove_bid() {
//         let mut ob = Orderbook::<BasicOrder>::default();
//         let node_ptr = ob.add_bid(1000, BasicOrder::new("1", true, 100));

//         let removed = ob.remove_bid(1000, node_ptr);
//         assert_eq!(removed, Some(BasicOrder::new("1", true, 100)));
//         assert_eq!(ob.bid_quantity_at(1000), 0);
//         assert!(ob.bids.is_empty());
//     }

//     #[test]
//     fn test_remove_ask() {
//         let mut ob = Orderbook::<BasicOrder>::default();
//         let node_ptr = ob.add_ask(1100, BasicOrder::new("1", false, 50));

//         let removed = ob.remove_ask(1100, node_ptr);
//         assert_eq!(removed, Some(BasicOrder::new("1", false, 50)));
//         assert_eq!(ob.ask_quantity_at(1100), 0);
//         assert!(ob.asks.is_empty());
//     }

//     #[test]
//     fn test_multiple_orders_at_same_price() {
//         let mut ob = Orderbook::<BasicOrder>::default();
//         ob.add_bid(1000, BasicOrder::new("1", true, 100));
//         ob.add_bid(1000, BasicOrder::new("2", true, 50));
//         ob.add_bid(1000, BasicOrder::new("3", true, 25));

//         assert_eq!(ob.bid_quantity_at(1000), 175);

//         let level = ob.bids.get_level(1000).unwrap();
//         assert_eq!(level.order_count(), 3);
//     }

//     #[test]
//     fn test_clear() {
//         let mut ob = Orderbook::<BasicOrder>::default();
//         ob.add_bid(1000, BasicOrder::new("1", true, 100));
//         ob.add_ask(1100, BasicOrder::new("2", false, 50));

//         ob.clear();
//         assert!(ob.is_empty());
//         assert_eq!(ob.level_count(), 0);
//     }
// }
