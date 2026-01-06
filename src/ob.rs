use crate::list::Node;
use crate::order::OrderInterface;
use crate::side::Side;
use std::collections::HashMap;

/// Represents a complete orderbook with bid and ask sides
pub struct OrderBook<O: OrderInterface> {
    /// The bid side (buy orders)
    bids: Side<O>,
    /// The ask side (sell orders)
    asks: Side<O>,

    // All orders in the orderbook
    orders: HashMap<O::T, *mut Node<O>>,

    // Temporary remaining quantities for the orders
    temp: HashMap<O::T, O::N>,
}

impl<O: OrderInterface> Default for OrderBook<O> {
    fn default() -> Self {
        Self {
            bids: Side::new(true),
            asks: Side::new(false),
            orders: HashMap::new(),
            temp: HashMap::new(),
        }
    }
}

/// An operation to apply to the orderbook
/// There are two types of operations: insert and delete
pub enum Op<O: OrderInterface> {
    Insert(O),
    Delete(O::T),
}

/// The eval result of an operation
/// There are three types of results: insert, match, and delete
/// Insert: The order was inserted into the orderbook
/// Match: The order was matched with other orders
/// Delete: The order was deleted from the orderbook
#[allow(dead_code)]
pub enum EvalResult<O: OrderInterface> {
    Insert(O, O::N),
    Match(O, O::N, Vec<(O::T, O::N)>), // Taker, Quantity, Makers
    Delete(O::T),
    NoOp(Msg), // No operation was performed
}

/// A message to indicate an error
pub enum Msg {
    OrderNotFound,
    OrderAlreadyExists,
}

pub enum Instruction<O: OrderInterface> {
    Insert(O, O::N),  // Order, Remaining Quantity
    Delete(O::T),     // Order ID
    Fill(O::T, O::N), // Order ID, Quantity
    NoOp(Msg),
}

/// Represents a match between a taker and one or more makers
pub struct Match<O: OrderInterface> {
    /// The taker order ID and filled quantity
    pub taker: (O::T, O::N),
    /// The maker order IDs and their filled quantities
    pub makers: Vec<(O::T, O::N)>,
}

impl<O: OrderInterface> OrderBook<O> {
    /// Applies a list of instructions to the orderbook, mutating state
    pub fn apply(&mut self, instructions: Vec<Instruction<O>>) {
        for instruction in instructions {
            match instruction {
                Instruction::Insert(order, remaining) => {
                    self.apply_insert(order, remaining);
                }
                Instruction::Delete(order_id) => {
                    self.apply_delete(&order_id);
                }
                Instruction::Fill(order_id, quantity) => {
                    self.apply_fill(&order_id, quantity);
                }
                Instruction::NoOp(_) => {}
            }
        }
        // Clear temporary state after applying
        self.temp.clear();
    }

    /// Inserts an order with the given remaining quantity
    fn apply_insert(&mut self, mut order: O, remaining: O::N) {
        let filled = order.quantity() - remaining;
        if filled > O::N::default() {
            order.fill(filled);
        }

        let id = order.id().clone();
        let is_buy = order.is_buy();

        let node_ptr = if is_buy {
            self.bids.insert_order(order)
        } else {
            self.asks.insert_order(order)
        };
        self.orders.insert(id, node_ptr);
    }

    /// Deletes an order by its ID
    fn apply_delete(&mut self, order_id: &O::T) {
        let node_ptr = if let Some(&ptr) = self.orders.get(order_id) {
            ptr
        } else {
            return;
        };

        let is_buy = unsafe { (*node_ptr).data.is_buy() };

        if is_buy {
            self.bids.remove_order(node_ptr);
        } else {
            self.asks.remove_order(node_ptr);
        }

        self.orders.remove(order_id);
    }

    /// Fills an order by the given quantity, removing it if fully filled
    fn apply_fill(&mut self, order_id: &O::T, quantity: O::N) {
        let node_ptr = if let Some(&ptr) = self.orders.get(order_id) {
            ptr
        } else {
            return;
        };

        let is_buy = unsafe { (*node_ptr).data.is_buy() };

        let removed = if is_buy {
            self.bids.fill_order(node_ptr, quantity)
        } else {
            self.asks.fill_order(node_ptr, quantity)
        };

        if removed {
            self.orders.remove(order_id);
        }
    }

    // Eval
    pub fn eval(&mut self, ops: Vec<Op<O>>) -> (Vec<Match<O>>, Vec<Instruction<O>>) {
        let mut matches = Vec::new();
        let mut instructions = Vec::new();

        for op in ops {
            match op {
                Op::Insert(order) => {
                    let (match_result, mut instrs) = self.eval_insert(order);
                    if let Some(m) = match_result {
                        matches.push(m);
                    }
                    instructions.append(&mut instrs);
                }
                Op::Delete(order_id) => {
                    let instr = self.eval_cancel(order_id);
                    instructions.push(instr);
                }
            }
        }

        (matches, instructions)
    }

    pub fn eval_insert(&mut self, order: O) -> (Option<Match<O>>, Vec<Instruction<O>>) {
        if self.orders.contains_key(order.id()) {
            return (None, vec![Instruction::NoOp(Msg::OrderAlreadyExists)]);
        }

        let mut remaining_quantity = order.remaining();
        let mut taker_quantity = O::N::default();
        let mut maker_quantities = Vec::new();
        let mut instructions = Vec::new();
        let is_buy = order.is_buy();

        let price = order.price();

        let check_fn = if is_buy {
            // For a buy order, stop matching when the price is less than the resting order price
            |price: O::N, resting_order: &O| price >= resting_order.price()
        } else {
            // For a sell order, stop matching when the price is greater than the resting order price
            |price: O::N, resting_order: &O| price <= resting_order.price()
        };

        // Match against the opposite side and collect orders to remove

        let opposite_book = if is_buy {
            &mut self.asks
        } else {
            &mut self.bids
        };

        for resting_order in opposite_book.iter_mut() {
            if check_fn(price, resting_order) && remaining_quantity > O::N::default() {
                // Check if the resting order has been partially filled in temp state
                let remaining = if let Some(rm) = self.temp.get(resting_order.id()) {
                    *rm
                } else {
                    resting_order.remaining()
                };

                if remaining == O::N::default() {
                    // Resting order is fully filled or cancelled, skip it
                    continue;
                }

                let taken_quantity = remaining_quantity.min(remaining);
                remaining_quantity -= taken_quantity;

                taker_quantity += taken_quantity;
                instructions.push(Instruction::Fill(
                    resting_order.id().clone(),
                    taken_quantity,
                ));
                maker_quantities.push((resting_order.id().clone(), taken_quantity));

                // Update temp state for the resting order
                self.temp
                    .insert(resting_order.id().clone(), remaining - taken_quantity);
            } else {
                break;
            }
        }

        let match_result = if taker_quantity > O::N::default() {
            Some(Match {
                taker: (order.id().clone(), taker_quantity),
                makers: maker_quantities,
            })
        } else {
            None
        };

        if remaining_quantity > O::N::default() {
            instructions.insert(0, Instruction::Insert(order, remaining_quantity));
        }

        (match_result, instructions)
    }

    pub fn eval_cancel(&mut self, order_id: O::T) -> Instruction<O> {
        if !self.orders.contains_key(&order_id) {
            return Instruction::NoOp(Msg::OrderNotFound);
        }

        self.temp.insert(order_id.clone(), O::N::default());

        Instruction::Delete(order_id)
    }
    // /// Inserts an order into the orderbook at the specified price
    // /// Iterates through the btree on the opposite side to check for matches
    // /// Matches orders and executes trades when prices cross
    // /// Any remaining quantity is added to the appropriate side
    // pub fn insert_order(&mut self, price: O::N, mut order: O) {
    //     let mut remaining_quantity = order.remaining();
    //     let mut taker_quantity = O::N::default();
    //     let mut maker_quantities = Vec::new();
    //     let is_buy = order.is_buy();

    //     let check_fn = if is_buy {
    //         // For a buy order, stop matching when the price is less than the resting order price
    //         |price: O::N, resting_order: &O| price >= resting_order.price()
    //     } else {
    //         // For a sell order, stop matching when the price is greater than the resting order price
    //         |price: O::N, resting_order: &O| price <= resting_order.price()
    //     };

    //     // Match against the opposite side and collect orders to remove

    //     let opposite_book = if is_buy {
    //         &mut self.asks
    //     } else {
    //         &mut self.bids
    //     };

    //     for resting_order in opposite_book.iter_mut() {
    //         if check_fn(price, resting_order) && remaining_quantity > O::N::default() {
    //             let remaining = if let Some(rm) = self.temp.get(order.id()) {
    //                 *rm
    //             } else {
    //                 order.remaining()
    //             };

    //             if remaining == O::N::default() {
    //                 // Order is fully filled or cancelled, skip it
    //                 continue;
    //             }

    //             let taken_quantity = remaining_quantity.min(remaining);
    //             remaining_quantity -= taken_quantity;

    //             taker_quantity += taken_quantity;
    //             let order_id = order.id().clone();
    //             maker_quantities.push((order_id.clone(), taken_quantity));
    //             self.temp.insert(order_id, remaining - taken_quantity);
    //         } else {
    //             break;
    //         }
    //     }

    //     // Handle the maker quantities
    //     for (order_id, quantity) in maker_quantities {
    //         if let Some(node_ptr) = self.orders.get(&order_id) {
    //             if opposite_book.fill_order(*node_ptr, quantity) {
    //                 self.orders.remove(&order_id);
    //             }
    //         }
    //     }

    //     // Handle the taker quantity
    //     if remaining_quantity > O::N::default() {
    //         order.fill(taker_quantity);
    //         let id = order.id().clone();
    //         let node_ptr = if is_buy {
    //             self.bids.insert_order(order)
    //         } else {
    //             self.asks.insert_order(order)
    //         };
    //         self.orders.insert(id, node_ptr);
    //     }
    // }

    // /// Cancels an order by its ID
    // /// Returns the cancelled order if found, None otherwise
    // /// Uses the node pointer stored in the orders map for O(1) cancellation
    // pub fn cancel_order(&mut self, order_id: &O::T) {
    //     // Get the node pointer from the orders map
    //     let node_ptr = if let Some(&ptr) = self.orders.get(order_id) {
    //         ptr
    //     } else {
    //         return; // Order not found, nothing to cancel
    //     };

    //     // Determine which side the order is on
    //     let is_buy = unsafe { (*node_ptr).data.is_buy() };

    //     // Remove from the side (the data has been forgotten, so this is safe)
    //     if is_buy {
    //         self.bids.remove_order(node_ptr);
    //     } else {
    //         self.asks.remove_order(node_ptr);
    //     }

    //     // Remove from orders map
    //     self.orders.remove(order_id);
    // }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::order::TestOrder;

    #[test]
    fn test_new_orderbook() {
        let ob = OrderBook::<TestOrder>::default();
        assert!(ob.bids.is_empty());
        assert!(ob.asks.is_empty());
    }

    #[test]
    fn test_eval_insert_buy_order_no_match() {
        let mut ob = OrderBook::<TestOrder>::default();
        let order = TestOrder::new("1", true, 1000, 100);

        let (match_result, instructions) = ob.eval_insert(order);

        // No match expected (empty book)
        assert!(match_result.is_none());
        // Should have an Insert instruction
        assert_eq!(instructions.len(), 1);
        assert!(matches!(&instructions[0], Instruction::Insert(_, 100)));
    }

    #[test]
    fn test_eval_insert_sell_order_no_match() {
        let mut ob = OrderBook::<TestOrder>::default();
        let order = TestOrder::new("1", false, 1100, 50);

        let (match_result, instructions) = ob.eval_insert(order);

        assert!(match_result.is_none());
        assert_eq!(instructions.len(), 1);
        assert!(matches!(&instructions[0], Instruction::Insert(_, 50)));
    }

    #[test]
    fn test_eval_insert_duplicate_order() {
        let mut ob = OrderBook::<TestOrder>::default();

        // Insert first order into the orders map to simulate it exists
        let order1 = TestOrder::new("1", true, 1000, 100);
        let node_ptr = ob.bids.insert_order(order1);
        ob.orders.insert(String::from("1"), node_ptr);

        // Try to insert duplicate
        let order2 = TestOrder::new("1", true, 1000, 50);
        let (match_result, instructions) = ob.eval_insert(order2);

        assert!(match_result.is_none());
        assert_eq!(instructions.len(), 1);
        assert!(matches!(
            &instructions[0],
            Instruction::NoOp(Msg::OrderAlreadyExists)
        ));
    }

    #[test]
    fn test_eval_cancel_nonexistent_order() {
        let mut ob = OrderBook::<TestOrder>::default();

        let instruction = ob.eval_cancel(String::from("nonexistent"));

        assert!(matches!(instruction, Instruction::NoOp(Msg::OrderNotFound)));
    }

    #[test]
    fn test_eval_cancel_existing_order() {
        let mut ob = OrderBook::<TestOrder>::default();

        // Add order to the book
        let order = TestOrder::new("1", true, 1000, 100);
        let node_ptr = ob.bids.insert_order(order);
        ob.orders.insert(String::from("1"), node_ptr);

        let instruction = ob.eval_cancel(String::from("1"));

        assert!(matches!(instruction, Instruction::Delete(_)));
        // Check temp map has zero remaining
        assert_eq!(*ob.temp.get(&String::from("1")).unwrap(), 0);
    }

    #[test]
    fn test_eval_buy_matches_sell_complete_fill() {
        let mut ob = OrderBook::<TestOrder>::default();

        // Add a sell order at 1000
        let sell_order = TestOrder::new("sell1", false, 1000, 100);
        let node_ptr = ob.asks.insert_order(sell_order);
        ob.orders.insert(String::from("sell1"), node_ptr);

        // Buy order at 1000 should match
        let buy_order = TestOrder::new("buy1", true, 1000, 100);
        let (match_result, instructions) = ob.eval_insert(buy_order);

        // Should have a match
        assert!(match_result.is_some());
        let m = match_result.unwrap();
        assert_eq!(m.taker.0, "buy1");
        assert_eq!(m.taker.1, 100);
        assert_eq!(m.makers.len(), 1);
        assert_eq!(m.makers[0].0, "sell1");
        assert_eq!(m.makers[0].1, 100);

        // Should have Fill instruction (no Insert since fully matched)
        assert_eq!(instructions.len(), 1);
        assert!(matches!(&instructions[0], Instruction::Fill(id, 100) if id == "sell1"));
    }

    #[test]
    fn test_eval_buy_matches_sell_partial_fill_buy_remaining() {
        let mut ob = OrderBook::<TestOrder>::default();

        // Add a sell order at 1000 with quantity 50
        let sell_order = TestOrder::new("sell1", false, 1000, 50);
        let node_ptr = ob.asks.insert_order(sell_order);
        ob.orders.insert(String::from("sell1"), node_ptr);

        // Buy order with quantity 100 should partially match
        let buy_order = TestOrder::new("buy1", true, 1000, 100);
        let (match_result, instructions) = ob.eval_insert(buy_order);

        assert!(match_result.is_some());
        let m = match_result.unwrap();
        assert_eq!(m.taker.1, 50); // Only 50 matched
        assert_eq!(m.makers[0].1, 50);

        // Should have Insert (remaining 50) + Fill
        assert_eq!(instructions.len(), 2);
        assert!(matches!(&instructions[0], Instruction::Insert(_, 50)));
        assert!(matches!(&instructions[1], Instruction::Fill(_, 50)));
    }

    #[test]
    fn test_eval_buy_does_not_match_higher_sell() {
        let mut ob = OrderBook::<TestOrder>::default();

        // Add a sell order at 1100
        let sell_order = TestOrder::new("sell1", false, 1100, 100);
        let node_ptr = ob.asks.insert_order(sell_order);
        ob.orders.insert(String::from("sell1"), node_ptr);

        // Buy order at 1000 should NOT match (price too low)
        let buy_order = TestOrder::new("buy1", true, 1000, 100);
        let (match_result, instructions) = ob.eval_insert(buy_order);

        assert!(match_result.is_none());
        assert_eq!(instructions.len(), 1);
        assert!(matches!(&instructions[0], Instruction::Insert(_, 100)));
    }

    #[test]
    fn test_eval_sell_does_not_match_lower_buy() {
        let mut ob = OrderBook::<TestOrder>::default();

        // Add a buy order at 1000
        let buy_order = TestOrder::new("buy1", true, 1000, 100);
        let node_ptr = ob.bids.insert_order(buy_order);
        ob.orders.insert(String::from("buy1"), node_ptr);

        // Sell order at 1100 should NOT match (price too high)
        let sell_order = TestOrder::new("sell1", false, 1100, 100);
        let (match_result, instructions) = ob.eval_insert(sell_order);

        assert!(match_result.is_none());
        assert_eq!(instructions.len(), 1);
        assert!(matches!(&instructions[0], Instruction::Insert(_, 100)));
    }

    #[test]
    fn test_eval_buy_at_higher_price_matches_lower_sell() {
        let mut ob = OrderBook::<TestOrder>::default();

        // Add a sell order at 1000
        let sell_order = TestOrder::new("sell1", false, 1000, 100);
        let node_ptr = ob.asks.insert_order(sell_order);
        ob.orders.insert(String::from("sell1"), node_ptr);

        // Buy order at 1100 should match the 1000 sell
        let buy_order = TestOrder::new("buy1", true, 1100, 100);
        let (match_result, _) = ob.eval_insert(buy_order);

        assert!(match_result.is_some());
        assert_eq!(match_result.unwrap().taker.1, 100);
    }

    #[test]
    fn test_eval_sell_at_lower_price_matches_higher_buy() {
        let mut ob = OrderBook::<TestOrder>::default();

        // Add a buy order at 1100
        let buy_order = TestOrder::new("buy1", true, 1100, 100);
        let node_ptr = ob.bids.insert_order(buy_order);
        ob.orders.insert(String::from("buy1"), node_ptr);

        // Sell order at 1000 should match the 1100 buy
        let sell_order = TestOrder::new("sell1", false, 1000, 100);
        let (match_result, _) = ob.eval_insert(sell_order);

        assert!(match_result.is_some());
        assert_eq!(match_result.unwrap().taker.1, 100);
    }

    #[test]
    fn test_eval_with_ops() {
        let mut ob = OrderBook::<TestOrder>::default();

        let ops = vec![
            Op::Insert(TestOrder::new("buy1", true, 1000, 100)),
            Op::Insert(TestOrder::new("sell1", false, 1100, 50)),
        ];

        let (matches, instructions) = ob.eval(ops);

        // No matches (prices don't cross)
        assert!(matches.is_empty());
        // Two inserts
        assert_eq!(instructions.len(), 2);
    }

    #[test]
    fn test_eval_matching_orders() {
        let mut ob = OrderBook::<TestOrder>::default();

        // First add a sell order to the book
        let sell_order = TestOrder::new("sell1", false, 1000, 100);
        let node_ptr = ob.asks.insert_order(sell_order);
        ob.orders.insert(String::from("sell1"), node_ptr);

        // Now evaluate a matching buy order
        let ops = vec![Op::Insert(TestOrder::new("buy1", true, 1000, 100))];

        let (matches, instructions) = ob.eval(ops);

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].taker.0, "buy1");
        assert_eq!(matches[0].makers[0].0, "sell1");
        assert_eq!(instructions.len(), 1);
    }

    #[test]
    fn test_temp_state_tracks_fills() {
        let mut ob = OrderBook::<TestOrder>::default();

        // Add a sell order
        let sell_order = TestOrder::new("sell1", false, 1000, 100);
        let node_ptr = ob.asks.insert_order(sell_order);
        ob.orders.insert(String::from("sell1"), node_ptr);

        // Partial match
        let buy_order = TestOrder::new("buy1", true, 1000, 30);
        ob.eval_insert(buy_order);

        // Temp should track remaining
        assert_eq!(*ob.temp.get(&String::from("sell1")).unwrap(), 70);

        // Another partial match
        let buy_order2 = TestOrder::new("buy2", true, 1000, 20);
        ob.eval_insert(buy_order2);

        assert_eq!(*ob.temp.get(&String::from("sell1")).unwrap(), 50);
    }

    #[test]
    fn test_cancelled_order_skipped_in_matching() {
        let mut ob = OrderBook::<TestOrder>::default();

        // Add a sell order
        let sell_order = TestOrder::new("sell1", false, 1000, 100);
        let node_ptr = ob.asks.insert_order(sell_order);
        ob.orders.insert(String::from("sell1"), node_ptr);

        // Cancel it
        ob.eval_cancel(String::from("sell1"));

        // Try to match - should not match cancelled order
        let buy_order = TestOrder::new("buy1", true, 1000, 50);
        let (match_result, instructions) = ob.eval_insert(buy_order);

        // No match since order was cancelled
        assert!(match_result.is_none());
        // Should insert the buy order
        assert!(matches!(&instructions[0], Instruction::Insert(_, 50)));
    }

    #[test]
    fn test_apply_insert() {
        let mut ob = OrderBook::<TestOrder>::default();

        let instructions = vec![Instruction::Insert(
            TestOrder::new("1", true, 1000, 100),
            100,
        )];
        ob.apply(instructions);

        assert!(!ob.bids.is_empty());
        assert!(ob.orders.contains_key(&String::from("1")));
        assert!(ob.temp.is_empty()); // Temp should be cleared
    }

    #[test]
    fn test_apply_insert_partial_fill() {
        let mut ob = OrderBook::<TestOrder>::default();

        // Insert with 70 remaining (30 already filled)
        let instructions = vec![Instruction::Insert(
            TestOrder::new("1", true, 1000, 100),
            70,
        )];
        ob.apply(instructions);

        // Verify order was inserted with correct remaining quantity
        let order = ob.bids.iter().next().unwrap();
        assert_eq!(order.remaining(), 70);
    }

    #[test]
    fn test_apply_delete() {
        let mut ob = OrderBook::<TestOrder>::default();

        // First insert an order
        let order = TestOrder::new("1", true, 1000, 100);
        let node_ptr = ob.bids.insert_order(order);
        ob.orders.insert(String::from("1"), node_ptr);

        // Now delete it
        let instructions = vec![Instruction::Delete(String::from("1"))];
        ob.apply(instructions);

        assert!(ob.bids.is_empty());
        assert!(!ob.orders.contains_key(&String::from("1")));
    }

    #[test]
    fn test_apply_fill_partial() {
        let mut ob = OrderBook::<TestOrder>::default();

        // Insert an order
        let order = TestOrder::new("1", false, 1000, 100);
        let node_ptr = ob.asks.insert_order(order);
        ob.orders.insert(String::from("1"), node_ptr);

        // Partially fill it
        let instructions = vec![Instruction::Fill(String::from("1"), 30)];
        ob.apply(instructions);

        // Order should still exist with reduced quantity
        assert!(ob.orders.contains_key(&String::from("1")));
        let order = ob.asks.iter().next().unwrap();
        assert_eq!(order.remaining(), 70);
    }

    #[test]
    fn test_apply_fill_complete() {
        let mut ob = OrderBook::<TestOrder>::default();

        // Insert an order
        let order = TestOrder::new("1", false, 1000, 100);
        let node_ptr = ob.asks.insert_order(order);
        ob.orders.insert(String::from("1"), node_ptr);

        // Fully fill it
        let instructions = vec![Instruction::Fill(String::from("1"), 100)];
        ob.apply(instructions);

        // Order should be removed
        assert!(!ob.orders.contains_key(&String::from("1")));
        assert!(ob.asks.is_empty());
    }

    #[test]
    fn test_apply_clears_temp() {
        let mut ob = OrderBook::<TestOrder>::default();

        // Add some temp state
        ob.temp.insert(String::from("1"), 50);

        // Apply empty instructions
        ob.apply(vec![]);

        // Temp should be cleared
        assert!(ob.temp.is_empty());
    }

    #[test]
    fn test_apply_noop() {
        let mut ob = OrderBook::<TestOrder>::default();

        // Apply NoOp instruction - should do nothing
        let instructions = vec![
            Instruction::NoOp(Msg::OrderNotFound),
            Instruction::NoOp(Msg::OrderAlreadyExists),
        ];
        ob.apply(instructions);

        assert!(ob.bids.is_empty());
        assert!(ob.asks.is_empty());
    }

    #[test]
    fn test_apply_insert_sell_order() {
        let mut ob = OrderBook::<TestOrder>::default();

        let instructions = vec![Instruction::Insert(
            TestOrder::new("1", false, 1000, 100),
            100,
        )];
        ob.apply(instructions);

        assert!(!ob.asks.is_empty());
        assert!(ob.orders.contains_key(&String::from("1")));
    }

    #[test]
    fn test_apply_delete_nonexistent() {
        let mut ob = OrderBook::<TestOrder>::default();

        // Delete non-existent order should not panic
        let instructions = vec![Instruction::Delete(String::from("nonexistent"))];
        ob.apply(instructions);

        assert!(ob.orders.is_empty());
    }

    #[test]
    fn test_apply_delete_sell_order() {
        let mut ob = OrderBook::<TestOrder>::default();

        // First insert a sell order
        let order = TestOrder::new("1", false, 1000, 100);
        let node_ptr = ob.asks.insert_order(order);
        ob.orders.insert(String::from("1"), node_ptr);

        // Now delete it
        let instructions = vec![Instruction::Delete(String::from("1"))];
        ob.apply(instructions);

        assert!(ob.asks.is_empty());
        assert!(!ob.orders.contains_key(&String::from("1")));
    }

    #[test]
    fn test_apply_fill_nonexistent() {
        let mut ob = OrderBook::<TestOrder>::default();

        // Fill non-existent order should not panic
        let instructions = vec![Instruction::Fill(String::from("nonexistent"), 50)];
        ob.apply(instructions);

        assert!(ob.orders.is_empty());
    }

    #[test]
    fn test_apply_fill_buy_order() {
        let mut ob = OrderBook::<TestOrder>::default();

        // Insert a buy order
        let order = TestOrder::new("1", true, 1000, 100);
        let node_ptr = ob.bids.insert_order(order);
        ob.orders.insert(String::from("1"), node_ptr);

        // Fill it
        let instructions = vec![Instruction::Fill(String::from("1"), 100)];
        ob.apply(instructions);

        assert!(!ob.orders.contains_key(&String::from("1")));
        assert!(ob.bids.is_empty());
    }

    #[test]
    fn test_eval_buy_exhausts_quantity_mid_match() {
        let mut ob = OrderBook::<TestOrder>::default();

        // Add two sell orders
        let sell1 = TestOrder::new("sell1", false, 1000, 50);
        let node_ptr1 = ob.asks.insert_order(sell1);
        ob.orders.insert(String::from("sell1"), node_ptr1);

        let sell2 = TestOrder::new("sell2", false, 1000, 50);
        let node_ptr2 = ob.asks.insert_order(sell2);
        ob.orders.insert(String::from("sell2"), node_ptr2);

        // Buy order exactly matches first sell, second sell untouched
        // This tests: check_fn true, but remaining_quantity becomes 0
        let buy_order = TestOrder::new("buy1", true, 1000, 50);
        let (match_result, instructions) = ob.eval_insert(buy_order);

        assert!(match_result.is_some());
        let m = match_result.unwrap();
        assert_eq!(m.taker.1, 50);
        assert_eq!(m.makers.len(), 1);
        assert_eq!(instructions.len(), 1); // Only 1 fill, no insert
    }

    #[test]
    fn test_eval_sell_exhausts_quantity_mid_match() {
        let mut ob = OrderBook::<TestOrder>::default();

        // Add two buy orders
        let buy1 = TestOrder::new("buy1", true, 1000, 50);
        let node_ptr1 = ob.bids.insert_order(buy1);
        ob.orders.insert(String::from("buy1"), node_ptr1);

        let buy2 = TestOrder::new("buy2", true, 1000, 50);
        let node_ptr2 = ob.bids.insert_order(buy2);
        ob.orders.insert(String::from("buy2"), node_ptr2);

        // Sell order exactly matches first buy
        let sell_order = TestOrder::new("sell1", false, 1000, 50);
        let (match_result, instructions) = ob.eval_insert(sell_order);

        assert!(match_result.is_some());
        let m = match_result.unwrap();
        assert_eq!(m.taker.1, 50);
        assert_eq!(m.makers.len(), 1);
        assert_eq!(instructions.len(), 1);
    }

    #[test]
    fn test_eval_buy_no_match_empty_book() {
        let mut ob = OrderBook::<TestOrder>::default();

        // Empty ask side - for loop doesn't iterate
        let buy_order = TestOrder::new("buy1", true, 1000, 100);
        let (match_result, instructions) = ob.eval_insert(buy_order);

        assert!(match_result.is_none());
        assert_eq!(instructions.len(), 1);
        assert!(matches!(&instructions[0], Instruction::Insert(_, 100)));
    }

    #[test]
    fn test_eval_sell_no_match_empty_book() {
        let mut ob = OrderBook::<TestOrder>::default();

        // Empty bid side - for loop doesn't iterate
        let sell_order = TestOrder::new("sell1", false, 1000, 100);
        let (match_result, instructions) = ob.eval_insert(sell_order);

        assert!(match_result.is_none());
        assert_eq!(instructions.len(), 1);
        assert!(matches!(&instructions[0], Instruction::Insert(_, 100)));
    }

    #[test]
    fn test_apply_fill_buy_partial() {
        let mut ob = OrderBook::<TestOrder>::default();

        // Insert a buy order
        let order = TestOrder::new("1", true, 1000, 100);
        let node_ptr = ob.bids.insert_order(order);
        ob.orders.insert(String::from("1"), node_ptr);

        // Partially fill it (not removed)
        let instructions = vec![Instruction::Fill(String::from("1"), 30)];
        ob.apply(instructions);

        assert!(ob.orders.contains_key(&String::from("1")));
        let order = ob.bids.iter().next().unwrap();
        assert_eq!(order.remaining(), 70);
    }

    #[test]
    fn test_eval_sell_matches_multiple_buys() {
        let mut ob = OrderBook::<TestOrder>::default();

        // Add multiple buy orders at different prices
        let buy1 = TestOrder::new("buy1", true, 1100, 30);
        let node_ptr1 = ob.bids.insert_order(buy1);
        ob.orders.insert(String::from("buy1"), node_ptr1);

        let buy2 = TestOrder::new("buy2", true, 1050, 40);
        let node_ptr2 = ob.bids.insert_order(buy2);
        ob.orders.insert(String::from("buy2"), node_ptr2);

        // Sell order at 1000 should match both buys (price <= both)
        let sell_order = TestOrder::new("sell1", false, 1000, 100);
        let (match_result, instructions) = ob.eval_insert(sell_order);

        assert!(match_result.is_some());
        let m = match_result.unwrap();
        assert_eq!(m.taker.1, 70); // 30 + 40 matched
        assert_eq!(m.makers.len(), 2);

        // Should have Insert (30 remaining) + 2 Fills
        assert_eq!(instructions.len(), 3);
    }

    #[test]
    fn test_eval_cancel_with_delete_op() {
        let mut ob = OrderBook::<TestOrder>::default();

        // Add order to the book
        let order = TestOrder::new("1", true, 1000, 100);
        let node_ptr = ob.bids.insert_order(order);
        ob.orders.insert(String::from("1"), node_ptr);

        // Use eval with Delete op
        let ops = vec![Op::Delete(String::from("1"))];
        let (matches, instructions) = ob.eval(ops);

        assert!(matches.is_empty());
        assert_eq!(instructions.len(), 1);
        assert!(matches!(instructions[0], Instruction::Delete(_)));
    }

    #[test]
    fn test_eval_then_apply() {
        let mut ob = OrderBook::<TestOrder>::default();

        // Add a sell order to the book
        let sell_order = TestOrder::new("sell1", false, 1000, 100);
        let node_ptr = ob.asks.insert_order(sell_order);
        ob.orders.insert(String::from("sell1"), node_ptr);

        // Eval a matching buy order
        let ops = vec![Op::Insert(TestOrder::new("buy1", true, 1000, 60))];
        let (matches, instructions) = ob.eval(ops);

        // Verify match
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].taker.1, 60);

        // Apply the instructions
        ob.apply(instructions);

        // Sell order should have 40 remaining
        let sell = ob.asks.iter().next().unwrap();
        assert_eq!(sell.remaining(), 40);

        // Buy order should not be in the book (fully matched)
        assert!(!ob.orders.contains_key(&String::from("buy1")));
    }

    #[test]
    fn test_eval_then_apply_with_insert() {
        let mut ob = OrderBook::<TestOrder>::default();

        // Add a sell order to the book
        let sell_order = TestOrder::new("sell1", false, 1000, 50);
        let node_ptr = ob.asks.insert_order(sell_order);
        ob.orders.insert(String::from("sell1"), node_ptr);

        // Eval a buy order that's larger than the sell
        let ops = vec![Op::Insert(TestOrder::new("buy1", true, 1000, 100))];
        let (matches, instructions) = ob.eval(ops);

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].taker.1, 50);

        // Apply
        ob.apply(instructions);

        // Sell order should be fully filled and removed
        assert!(ob.asks.is_empty());
        assert!(!ob.orders.contains_key(&String::from("sell1")));

        // Buy order should be inserted with remaining 50
        assert!(ob.orders.contains_key(&String::from("buy1")));
        let buy = ob.bids.iter().next().unwrap();
        assert_eq!(buy.remaining(), 50);
    }
}
