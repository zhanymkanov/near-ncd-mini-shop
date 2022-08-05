use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedMap, Vector};
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, AccountId, PanicOnDefault};

near_sdk::setup_alloc!();

const ONE_NEAR: u128 = 1_000_000_000_000_000_000_000_000; // 1 near as yoctoNEAR

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum ShopProduct {
    SmallSnack,
    LargeSnack,
    Soda,
    IceCream,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Shop {
    catalog: UnorderedMap<u8, ShopProduct>, // Product ID and Name
    stock: UnorderedMap<u8, u8>,            // Product ID and Amount in stock
    product_prices: LookupMap<u8, U128>,
    purchase_history: Vector<String>,
}

#[near_bindgen]
impl Shop {
    #[init]
    pub fn new() -> Self {
        assert!(!env::state_exists(), "The contract is already initialized");

        let mut shop = Self {
            catalog: UnorderedMap::new(b"c".to_vec()),
            stock: UnorderedMap::new(b"s".to_vec()),
            product_prices: LookupMap::new(b"p".to_vec()),
            purchase_history: Vector::new(b"v".to_vec()),
        };

        shop.init_products();

        shop
    }

    #[payable]
    pub fn buy(&mut self, product: u8) -> &str {
        if self.catalog.get(&product).is_none() {
            env::panic(b"Product not found")
        }
        if self.stock.get(&product).unwrap() <= 0 {
            env::panic(b"Product out of stock")
        }
        let product_price: U128 = match self.product_prices.get(&product) {
            Some(price) => price,
            None => env::panic(b"Product price not found"),
        };
        let product_price: u128 = product_price.into();

        if env::attached_deposit() < product_price {
            env::panic(b"Attached deposit is not enough");
        }

        let buyer_id = env::predecessor_account_id();
        let log_message = format!("User {} has purchased product {}", &buyer_id, &product);
        env::log(log_message.as_bytes());

        self.deliver_product(&product, &buyer_id);

        if env::attached_deposit() > product_price {
            "Thank you for tips!"
        } else {
            "Thank you for purchase"
        }
    }

    #[payable]
    pub fn set_product_availability(&mut self, product: u8, amount: u8) -> (u8, u8) {
        if env::predecessor_account_id() != env::current_account_id() {
            env::panic(b"Only owner can set products availability")
        }
        self.stock.insert(&product, &amount);
        (product, amount)
    }

    pub fn view_catalog(&self, from_index: u64, limit: u64) -> Vec<(u8, ShopProduct)> {
        self.map_to_vec(&self.catalog, from_index, limit)
    }

    pub fn view_stock(&self, from_index: u64, limit: u64) -> Vec<(u8, u8)> {
        self.map_to_vec(&self.stock, from_index, limit)
    }

    pub fn get_product_price(&self, product: u8) -> U128 {
        if !self.product_prices.contains_key(&product) {
            env::panic(b"Product not found")
        }

        self.product_prices.get(&product).unwrap()
    }

    fn deliver_product(&mut self, product: &u8, buyer_id: &AccountId) {
        let in_stock = self.stock.get(product).unwrap();
        self.stock.insert(product, &(in_stock - 1));

        let log_message = format!("Product {} has been delivered to {}", product, buyer_id);
        env::log(log_message.as_bytes());

        let account_action = String::from(format!("{}:{}", env::predecessor_account_id(), product));
        self.purchase_history.push(&account_action);
        env::log("Saved account purchase".as_bytes())
    }

    fn map_to_vec<K: BorshSerialize + BorshDeserialize, V: BorshSerialize + BorshDeserialize>(
        &self,
        map: &UnorderedMap<K, V>,
        from_index: u64,
        limit: u64,
    ) -> Vec<(K, V)> {
        let keys = map.keys_as_vector();
        let values = map.values_as_vector();
        (from_index..std::cmp::min(from_index + limit, self.stock.len()))
            .map(|index| (keys.get(index).unwrap(), values.get(index).unwrap()))
            .collect()
    }

    fn init_products(&mut self) {
        self.init_catalog();
        self.init_stock();
        self.init_product_prices();
    }

    fn init_catalog(&mut self) {
        if !self.catalog.is_empty() {
            env::panic(b"Catalog is already initialized")
        }
        let products = [
            ShopProduct::SmallSnack,
            ShopProduct::LargeSnack,
            ShopProduct::IceCream,
            ShopProduct::Soda,
        ];

        for (idx, product) in products.iter().enumerate() {
            self.catalog.insert(&(idx as u8), &product);
        }
    }

    fn init_stock(&mut self) {
        if !self.stock.is_empty() {
            env::panic(b"Catalog is already initialized")
        }
        let product_availability = [200, 150, 150, 0];
        for (idx, amount) in product_availability.iter().enumerate() {
            self.stock.insert(&(idx as u8), &amount);
        }
    }

    fn init_product_prices(&mut self) {
        let product_prices = [ONE_NEAR, 2 * ONE_NEAR, 3 * ONE_NEAR, 2 * ONE_NEAR];
        for (idx, price) in product_prices.iter().enumerate() {
            self.product_prices.insert(&(idx as u8), &U128(*price));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::get_logs;
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, VMContext};

    fn get_context(attached_deposit: u128, is_view: bool, signer_account_id: &str) -> VMContext {
        VMContext {
            current_account_id: "shop.testnet".to_string(),
            signer_account_id: signer_account_id.to_string(), // initial caller of the contract
            signer_account_pk: vec![0, 1, 2],
            predecessor_account_id: signer_account_id.to_string(), // last caller of the contract
            input: vec![],
            block_index: 0,
            block_timestamp: 0,
            account_balance: 0,
            account_locked_balance: 0,
            storage_usage: 0,
            attached_deposit,
            prepaid_gas: 10u64.pow(18),
            random_seed: vec![0, 1, 2],
            is_view, // true if write operation (chargeable), false if read operation (free)
            output_data_receivers: vec![],
            epoch_height: 19,
        }
    }

    #[test]
    fn test_purchase_success() {
        let context = get_context(ONE_NEAR, false, "buyer.testnet");
        testing_env!(context);

        let mut contract = Shop::new();
        let resp = contract.buy(0);

        println!("{:?}", get_logs());

        assert_eq!(resp, "Thank you for purchase");
    }

    #[test]
    fn test_purchase_with_tips_success() {
        let context = get_context(2u128 * ONE_NEAR, false, "buyer.testnet");
        testing_env!(context);

        let mut contract = Shop::new();
        let resp = contract.buy(0);

        assert_eq!(resp, "Thank you for tips!");
    }

    #[test]
    #[should_panic(expected = "Product not found")]
    fn test_product_not_found() {
        let context = get_context(2u128 * ONE_NEAR, false, "buyer.testnet");
        testing_env!(context);

        Shop::new().buy(255);
    }

    #[test]
    #[should_panic(expected = "Product out of stock")]
    fn test_product_out_of_stock() {
        let context = get_context(2u128 * ONE_NEAR, false, "buyer.testnet");
        testing_env!(context);

        Shop::new().buy(3);
    }
    #[test]
    fn test_view_catalog() {
        let context = get_context(0, false, "buyer.testnet");
        testing_env!(context);

        let contract = Shop::new();
        let resp = contract.view_catalog(0, 10);

        println!("{:?}", &resp);
        assert_eq!(resp.is_empty(), false)
    }
    #[test]
    fn test_view_stock() {
        let context = get_context(0, false, "buyer.testnet");
        testing_env!(context);

        let contract = Shop::new();
        let resp = contract.view_stock(0, 10);

        println!("{:?}", &resp);
        assert_eq!(resp.is_empty(), false)
    }
    #[test]
    fn test_set_availability() {
        let context = get_context(0, false, "shop.testnet");
        testing_env!(context);

        let resp = Shop::new().set_product_availability(0, 10);
        assert_eq!(resp, (0, 10))
    }
    #[test]
    #[should_panic(expected = "Only owner can set products availability")]
    fn test_set_availability_no_access() {
        let context = get_context(0, false, "buyer.testnet");
        testing_env!(context);

        Shop::new().set_product_availability(0, 10);
    }
    #[test]
    fn test_get_product_price() {
        let context = get_context(0, false, "buyer.testnet");
        testing_env!(context);

        let resp = Shop::new().get_product_price(0);
        assert_eq!(resp, U128(ONE_NEAR))
    }
    #[test]
    #[should_panic(expected = "Product not found")]
    fn test_get_product_price_not_found() {
        let context = get_context(0, false, "buyer.testnet");
        testing_env!(context);

        Shop::new().get_product_price(255);
    }
}
