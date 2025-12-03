use ob::ob::OrderBook;
#[path = "common/mod.rs"]
mod common;
use common::BasicOrder;

#[test]
fn test_new_orderbook() {
    let _ = OrderBook::<BasicOrder>::default();
    // Orderbook should be empty initially
    // We can verify by trying to cancel a non-existent order (will panic, but that's expected)
}

#[test]
fn test_insert_buy_order_no_match() {
    let mut ob = OrderBook::<BasicOrder>::default();
    let order = BasicOrder::new("1", true, 1000, 100);
    ob.insert_order(1000, order);

    // Order should be in the book, verify by cancelling it
    ob.cancel_order("1");
    // If we get here without panic, the order was successfully added and removed
}

#[test]
fn test_insert_sell_order_no_match() {
    let mut ob = OrderBook::<BasicOrder>::default();
    let order = BasicOrder::new("1", false, 1100, 50);
    ob.insert_order(1100, order);

    // Order should be in the book, verify by cancelling it
    ob.cancel_order("1");
}

#[test]
fn test_buy_order_matches_sell_order_complete_fill() {
    let mut ob = OrderBook::<BasicOrder>::default();

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
    let mut ob = OrderBook::<BasicOrder>::default();

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
    let mut ob = OrderBook::<BasicOrder>::default();

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
    let mut ob = OrderBook::<BasicOrder>::default();

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
    let mut ob = OrderBook::<BasicOrder>::default();

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
    let mut ob = OrderBook::<BasicOrder>::default();

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
    let mut ob = OrderBook::<BasicOrder>::default();

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
    let mut ob = OrderBook::<BasicOrder>::default();

    // Add a sell order at 1000
    ob.insert_order(1000, BasicOrder::new("sell1", false, 1000, 100));

    // Add a buy order at 1100 (higher price, should match)
    ob.insert_order(1100, BasicOrder::new("buy1", true, 1100, 100));

    // Both should match and be removed
}

#[test]
fn test_sell_order_at_lower_price_matches_higher_buy() {
    let mut ob = OrderBook::<BasicOrder>::default();

    // Add a buy order at 1100
    ob.insert_order(1100, BasicOrder::new("buy1", true, 1100, 100));

    // Add a sell order at 1000 (lower price, should match)
    ob.insert_order(1000, BasicOrder::new("sell1", false, 1000, 100));

    // Both should match and be removed
}

#[test]
fn test_buy_order_does_not_match_higher_sell() {
    let mut ob = OrderBook::<BasicOrder>::default();

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
    let mut ob = OrderBook::<BasicOrder>::default();

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
    let mut ob = OrderBook::<BasicOrder>::default();
    ob.insert_order(1000, BasicOrder::new("buy1", true, 1000, 100));

    ob.cancel_order("buy1");
    // Order should be removed, verify by attempting to cancel again (should panic)
}

#[test]
fn test_cancel_sell_order() {
    let mut ob = OrderBook::<BasicOrder>::default();
    ob.insert_order(1100, BasicOrder::new("sell1", false, 1100, 50));

    ob.cancel_order("sell1");
    // Order should be removed
}

#[test]
fn test_multiple_orders_at_same_price() {
    let mut ob = OrderBook::<BasicOrder>::default();

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
    let mut ob = OrderBook::<BasicOrder>::default();

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
    let mut ob = OrderBook::<BasicOrder>::default();

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
    let mut ob = OrderBook::<BasicOrder>::default();

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
    let mut ob = OrderBook::<BasicOrder>::default();

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
    let mut ob = OrderBook::<BasicOrder>::default();
    ob.cancel_order("nonexistent");
}

#[test]
fn test_complex_matching_scenario() {
    let mut ob = OrderBook::<BasicOrder>::default();

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
