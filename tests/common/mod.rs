use ob::order::OrderInterface;

/// A basic order implementation for testing
#[derive(Default, Debug, PartialEq, Eq)]
pub struct BasicOrder {
    id: String,
    is_buy: bool,
    price: u64,
    quantity: u64,
    remaining: u64,
}

impl BasicOrder {
    pub fn new(id: &str, is_buy: bool, price: u64, quantity: u64) -> Self {
        Self {
            id: id.to_string(),
            is_buy,
            price,
            quantity,
            remaining: quantity,
        }
    }
}

impl OrderInterface<String, u64> for BasicOrder {
    fn id<'a>(&'a self) -> &'a String {
        &self.id
    }

    fn price(&self) -> &u64 {
        &self.price
    }

    fn is_buy(&self) -> bool {
        self.is_buy
    }

    fn quantity(&self) -> &u64 {
        &self.quantity
    }

    fn remaining(&self) -> &u64 {
        &self.remaining
    }

    fn fill(&mut self, quantity: u64) {
        self.remaining -= quantity;
    }
}

