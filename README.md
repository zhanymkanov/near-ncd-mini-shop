## About
Smart contract represents a vending machine like shop, where customer can buy products from catalog. 
- Customer inputs the product id and attaches required deposit. 
    - Money is not returnable, so check if attachment is enough to buy it. 
      If it exceeds the product price, it's kept as tips.
- Only the owner of the shop (shop itself) can change the number of available products in the stock.

## Available methods
### `view_catalog`
\- get slice of the list of products with their corresponding id.

**Required Parameters:**
1. `from_index` - start index of slice
2. `limit` - length of the slice

**Request Sample:**
```
near view shop.ezer.testnet view_catalog '{"from_index": 0, "limit": 10}'
```
**Response Sample:**
```rust
[
    [ 0, 'SmallSnack' ],
    [ 1, 'LargeSnack' ],
    [ 2, 'Soda' ],
    [ 3, 'IceCream' ]
]
```

### `view_stock`
\- get slice of the list of products with their amount in stock.

**Required Parameters:**
1. `from_index` - start index of slice
2. `limit` - length of the slice

**Request Sample:**
```
near view shop.ezer.testnet view_stock '{"from_index": 0, "limit": 10}'
```
**Response Sample:**
```rust
[
    [ 0, 100 ],
    [ 1, 150 ],
    [ 2, 200 ],
    [ 3, 100 ]
]
```

### `get_product_price`  
\- get price of the product in yocto Nears

**Required Parameters:**
1. `product` - product id

**Request Sample:**
```
near view shop.ezer.testnet get_product_price '{"product": 0}'
```
**Response Sample:**
```rust
'1000000000000000000000000'
```
**Raises**
1. `Product not found` - if product id does not exist

### `buy` 
\- buy the product by id

**Required Parameters:**
1. `product` - product id

**Request Sample with deposit of 2 NEAR**
```
near call shop.ezer.testnet buy '{"product": 0}' --accountId *YOUR ACCOUNT* --deposit 2
```
**Request Sample with deposit of 2 NEAR in yocto**
```
near call shop.ezer.testnet buy '{"product": 0}' --accountId *YOUR ACCOUNT* --depositYocto 2000000000000000000000000
```
**Response Sample:**
```
'Thank you for purchase'
```
**Raises**
1. `Product not found`
2. `Product out of stock`
3. `Product price not found` - if product is in catalog and in stock, but has no information about its price (if owner forgets to put it there)
4. `Attached deposit not enough`

### `set_product_availability` 
\- set amount of available products for purchase. Only owner of the shop can do this operation.

**Required Parameters:**
1. `product` - product id
2. `amount` - amount of products to set for sale

**Request Sample**
```
near call shop.ezer.testnet set_product_availability '{"product": 0, "amount": 100}' --accountId *OWNER_ACCOUNT*
```
**Response Sample:**
```rust
[0, 100]
```
**Raises**
1. `Only owner can set products availability`

## How to build it locally
### Requirements
1. rust with cargo
### Steps
1. git clone `https://github.com/zhanymkanov/near-ncd-mini-shop`
2. `cd near-ncd-mini-shop`
3. Make some changes to code & add tests
4. Run tests with `cargo test`
5. If everything is ok, build it with

```
RUSTFLAGS='-C link-arg=-s' cargo build --target wasm32-unknown-unknown --release
```

6. Deploy the contract to your account with
```
near deploy --wasmFile target/wasm32-unknown-unknown/release/mini_shop.wasm --accountId *YOUR ACCOUNT*
```
