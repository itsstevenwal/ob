use crate::{list::Node, order::OrderInterface, side::Side};
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
            orders: HashMap::default(),
            temp: HashMap::default(),
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
#[derive(Debug, PartialEq, Eq)]
pub enum Msg {
    OrderNotFound,
    OrderAlreadyExists,
}

#[derive(Debug, PartialEq, Eq)]
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
    #[inline]
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
    #[inline(always)]
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
    #[inline(always)]
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
    #[inline(always)]
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
    #[inline]
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

    #[inline(always)]
    pub fn eval_insert(&mut self, order: O) -> (Option<Match<O>>, Vec<Instruction<O>>) {
        if self.orders.contains_key(order.id()) {
            return self.eval_insert_duplicate();
        }

        let mut remaining_quantity = order.remaining();
        let mut taker_quantity = O::N::default();
        let mut maker_quantities = Vec::new();
        let mut instructions = Vec::with_capacity(16);
        let is_buy = order.is_buy();
        let price = order.price();

        // Match against the opposite side and collect orders to remove
        let opposite_book = if is_buy {
            &mut self.asks
        } else {
            &mut self.bids
        };

        for resting_order in opposite_book.iter_mut() {
            // Inline price check - for buys: price >= resting, for sells: price <= resting
            let price_matches = if is_buy {
                price >= resting_order.price()
            } else {
                price <= resting_order.price()
            };

            if price_matches && remaining_quantity > O::N::default() {
                // Check if the resting order has been partially filled in temp state
                let remaining = self
                    .temp
                    .get(resting_order.id())
                    .copied()
                    .unwrap_or_else(|| resting_order.remaining());

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

        // Insert at front without O(n) shift - push then rotate
        if remaining_quantity > O::N::default() {
            instructions.push(Instruction::Insert(order, remaining_quantity));
            // Rotate last element to front: O(n) but avoids reallocation
            instructions.rotate_right(1);
        }

        (match_result, instructions)
    }

    #[cold]
    #[inline(never)]
    fn eval_insert_duplicate(&self) -> (Option<Match<O>>, Vec<Instruction<O>>) {
        (None, vec![Instruction::NoOp(Msg::OrderAlreadyExists)])
    }

    #[inline(always)]
    pub fn eval_cancel(&mut self, order_id: O::T) -> Instruction<O> {
        if !self.orders.contains_key(&order_id) {
            return Self::eval_cancel_not_found();
        }

        self.temp.insert(order_id.clone(), O::N::default());

        Instruction::Delete(order_id)
    }

    #[cold]
    #[inline(never)]
    fn eval_cancel_not_found() -> Instruction<O> {
        Instruction::NoOp(Msg::OrderNotFound)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::order::TestOrder;

    fn setup_order(ob: &mut OrderBook<TestOrder>, id: &str, is_buy: bool, price: u64, qty: u64) {
        let order = TestOrder::new(id, is_buy, price, qty);
        let node_ptr = if is_buy {
            ob.bids.insert_order(order)
        } else {
            ob.asks.insert_order(order)
        };
        ob.orders.insert(String::from(id), node_ptr);
    }

    // === Eval Tests ===

    #[test]
    fn test_eval_insert_no_match() {
        let mut ob = OrderBook::<TestOrder>::default();

        // Buy into empty book
        let order = TestOrder::new("1", true, 1000, 100);
        let (m, i) = ob.eval_insert(order.clone());
        assert!(m.is_none());
        assert_eq!(i[0], Instruction::Insert(order, 100));

        // Sell into empty book
        let mut ob = OrderBook::<TestOrder>::default();
        let order = TestOrder::new("1", false, 1000, 50);
        let (m, i) = ob.eval_insert(order.clone());
        assert!(m.is_none());
        assert_eq!(i[0], Instruction::Insert(order, 50));
    }

    #[test]
    fn test_eval_insert_duplicate() {
        let mut ob = OrderBook::<TestOrder>::default();
        setup_order(&mut ob, "1", true, 1000, 100);

        let (m, i) = ob.eval_insert(TestOrder::new("1", true, 1000, 50));
        assert!(m.is_none());
        assert_eq!(i[0], Instruction::NoOp(Msg::OrderAlreadyExists));
    }

    #[test]
    fn test_eval_cancel() {
        let mut ob = OrderBook::<TestOrder>::default();

        // Cancel non-existent
        let i = ob.eval_cancel(String::from("x"));
        assert_eq!(i, Instruction::NoOp(Msg::OrderNotFound));

        // Cancel existing
        setup_order(&mut ob, "1", true, 1000, 100);
        let i = ob.eval_cancel(String::from("1"));
        assert_eq!(i, Instruction::Delete(String::from("1")));
        assert_eq!(*ob.temp.get("1").unwrap(), 0);
    }

    #[test]
    fn test_eval_matching() {
        let mut ob = OrderBook::<TestOrder>::default();
        setup_order(&mut ob, "s1", false, 1000, 100);

        // Complete fill
        let (m, i) = ob.eval_insert(TestOrder::new("b1", true, 1000, 100));
        assert_eq!(m.unwrap().taker.1, 100);
        assert_eq!(i.len(), 1);
        assert_eq!(i[0], Instruction::Fill(String::from("s1"), 100));

        // Partial fill with taker remaining
        let mut ob = OrderBook::<TestOrder>::default();
        setup_order(&mut ob, "s1", false, 1000, 50);
        let order = TestOrder::new("b1", true, 1000, 100);
        let (m, i) = ob.eval_insert(order.clone());
        assert_eq!(m.unwrap().taker.1, 50);
        assert_eq!(i.len(), 2);
        assert_eq!(i[0], Instruction::Insert(order, 50));
    }

    #[test]
    fn test_eval_price_crossing() {
        // Buy doesn't match higher sell
        let mut ob = OrderBook::<TestOrder>::default();
        setup_order(&mut ob, "s1", false, 1100, 100);
        let (m, _) = ob.eval_insert(TestOrder::new("b1", true, 1000, 100));
        assert!(m.is_none());

        // Buy at higher price matches lower sell
        let mut ob = OrderBook::<TestOrder>::default();
        setup_order(&mut ob, "s1", false, 1000, 100);
        let (m, _) = ob.eval_insert(TestOrder::new("b1", true, 1100, 100));
        assert_eq!(m.unwrap().taker.1, 100);

        // Sell doesn't match lower buy
        let mut ob = OrderBook::<TestOrder>::default();
        setup_order(&mut ob, "b1", true, 1000, 100);
        let (m, _) = ob.eval_insert(TestOrder::new("s1", false, 1100, 100));
        assert!(m.is_none());

        // Sell at lower price matches higher buy
        let mut ob = OrderBook::<TestOrder>::default();
        setup_order(&mut ob, "b1", true, 1100, 100);
        let (m, _) = ob.eval_insert(TestOrder::new("s1", false, 1000, 100));
        assert_eq!(m.unwrap().taker.1, 100);
    }

    #[test]
    fn test_eval_multi_maker_match() {
        let mut ob = OrderBook::<TestOrder>::default();
        setup_order(&mut ob, "b1", true, 1100, 30);
        setup_order(&mut ob, "b2", true, 1050, 40);

        let (m, i) = ob.eval_insert(TestOrder::new("s1", false, 1000, 100));
        assert_eq!(m.unwrap().makers.len(), 2);
        assert_eq!(i.len(), 3); // Insert + 2 Fills
    }

    #[test]
    fn test_eval_quantity_exhausted() {
        // Buy exhausts mid-match
        let mut ob = OrderBook::<TestOrder>::default();
        setup_order(&mut ob, "s1", false, 1000, 50);
        setup_order(&mut ob, "s2", false, 1000, 50);
        let (m, i) = ob.eval_insert(TestOrder::new("b1", true, 1000, 50));
        assert_eq!(m.unwrap().makers.len(), 1);
        assert_eq!(i.len(), 1);

        // Sell exhausts mid-match
        let mut ob = OrderBook::<TestOrder>::default();
        setup_order(&mut ob, "b1", true, 1000, 50);
        setup_order(&mut ob, "b2", true, 1000, 50);
        let (m, i) = ob.eval_insert(TestOrder::new("s1", false, 1000, 50));
        assert_eq!(m.unwrap().makers.len(), 1);
        assert_eq!(i.len(), 1);
    }

    #[test]
    fn test_eval_with_ops() {
        let mut ob = OrderBook::<TestOrder>::default();
        let ops = vec![
            Op::Insert(TestOrder::new("b1", true, 1000, 100)),
            Op::Insert(TestOrder::new("s1", false, 1100, 50)),
            Op::Delete(String::from("b1")),
        ];
        let (matches, instructions) = ob.eval(ops);
        assert!(matches.is_empty());
        assert_eq!(instructions.len(), 3);
    }

    #[test]
    fn test_temp_state() {
        let mut ob = OrderBook::<TestOrder>::default();
        setup_order(&mut ob, "s1", false, 1000, 100);

        ob.eval_insert(TestOrder::new("b1", true, 1000, 30));
        assert_eq!(*ob.temp.get("s1").unwrap(), 70);

        ob.eval_insert(TestOrder::new("b2", true, 1000, 20));
        assert_eq!(*ob.temp.get("s1").unwrap(), 50);

        // Cancelled order skipped
        ob.eval_cancel(String::from("s1"));
        let (m, _) = ob.eval_insert(TestOrder::new("b3", true, 1000, 50));
        assert!(m.is_none());
    }

    // === Apply Tests ===

    #[test]
    fn test_apply_insert() {
        let mut ob = OrderBook::<TestOrder>::default();
        ob.apply(vec![Instruction::Insert(
            TestOrder::new("1", true, 1000, 100),
            100,
        )]);
        assert!(ob.orders.contains_key("1"));
        assert!(ob.temp.is_empty());

        // Sell order
        let mut ob = OrderBook::<TestOrder>::default();
        ob.apply(vec![Instruction::Insert(
            TestOrder::new("1", false, 1000, 100),
            100,
        )]);
        assert!(!ob.asks.is_empty());

        // Partial fill on insert
        let mut ob = OrderBook::<TestOrder>::default();
        ob.apply(vec![Instruction::Insert(
            TestOrder::new("1", true, 1000, 100),
            70,
        )]);
        assert_eq!(ob.bids.iter().next().unwrap().remaining(), 70);
    }

    #[test]
    fn test_apply_delete() {
        let mut ob = OrderBook::<TestOrder>::default();
        setup_order(&mut ob, "1", true, 1000, 100);
        ob.apply(vec![Instruction::Delete(String::from("1"))]);
        assert!(ob.bids.is_empty());

        // Sell order
        let mut ob = OrderBook::<TestOrder>::default();
        setup_order(&mut ob, "1", false, 1000, 100);
        ob.apply(vec![Instruction::Delete(String::from("1"))]);
        assert!(ob.asks.is_empty());

        // Non-existent (no panic)
        let mut ob = OrderBook::<TestOrder>::default();
        ob.apply(vec![Instruction::Delete(String::from("x"))]);
    }

    #[test]
    fn test_apply_fill() {
        // Partial fill sell
        let mut ob = OrderBook::<TestOrder>::default();
        setup_order(&mut ob, "1", false, 1000, 100);
        ob.apply(vec![Instruction::Fill(String::from("1"), 30)]);
        assert_eq!(ob.asks.iter().next().unwrap().remaining(), 70);

        // Complete fill sell
        let mut ob = OrderBook::<TestOrder>::default();
        setup_order(&mut ob, "1", false, 1000, 100);
        ob.apply(vec![Instruction::Fill(String::from("1"), 100)]);
        assert!(ob.asks.is_empty());

        // Partial fill buy
        let mut ob = OrderBook::<TestOrder>::default();
        setup_order(&mut ob, "1", true, 1000, 100);
        ob.apply(vec![Instruction::Fill(String::from("1"), 30)]);
        assert_eq!(ob.bids.iter().next().unwrap().remaining(), 70);

        // Complete fill buy
        let mut ob = OrderBook::<TestOrder>::default();
        setup_order(&mut ob, "1", true, 1000, 100);
        ob.apply(vec![Instruction::Fill(String::from("1"), 100)]);
        assert!(ob.bids.is_empty());

        // Non-existent (no panic)
        let mut ob = OrderBook::<TestOrder>::default();
        ob.apply(vec![Instruction::Fill(String::from("x"), 50)]);
    }

    #[test]
    fn test_apply_noop() {
        let mut ob = OrderBook::<TestOrder>::default();
        ob.apply(vec![
            Instruction::NoOp(Msg::OrderNotFound),
            Instruction::NoOp(Msg::OrderAlreadyExists),
        ]);
        assert!(ob.bids.is_empty());
    }

    #[test]
    fn test_apply_clears_temp() {
        let mut ob = OrderBook::<TestOrder>::default();
        ob.temp.insert(String::from("1"), 50);
        ob.apply(vec![]);
        assert!(ob.temp.is_empty());
    }

    // === Integration Tests ===

    #[test]
    fn test_eval_then_apply() {
        let mut ob = OrderBook::<TestOrder>::default();
        setup_order(&mut ob, "s1", false, 1000, 100);

        let ops = vec![Op::Insert(TestOrder::new("b1", true, 1000, 60))];
        let (matches, instructions) = ob.eval(ops);
        assert_eq!(matches[0].taker.1, 60);

        ob.apply(instructions);
        assert_eq!(ob.asks.iter().next().unwrap().remaining(), 40);
        assert!(!ob.orders.contains_key("b1"));
    }

    #[test]
    fn test_eval_then_apply_with_insert() {
        let mut ob = OrderBook::<TestOrder>::default();
        setup_order(&mut ob, "s1", false, 1000, 50);

        let ops = vec![Op::Insert(TestOrder::new("b1", true, 1000, 100))];
        let (_, instructions) = ob.eval(ops);

        ob.apply(instructions);
        assert!(ob.asks.is_empty());
        assert_eq!(ob.bids.iter().next().unwrap().remaining(), 50);
    }
}
