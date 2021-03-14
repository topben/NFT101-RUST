# NFT101-RUST

### 基于substrate的NFT开发

#### 一、trait Type: 类型信息/常数

##### 类型

- NftId: Nft艺术品Id
- OrderId: 订单Id

##### 常数

- MinKeepBlockNumber: 拍卖订单最小保留区块数
- MaxKeepBlockNumber: 拍卖订单最大保留区块数
- MinimumPrice: 最小拍卖价格
- MinimumVotingLock: 最小质押投票数量
- FixRate: 用于分润算法的固定利润常数
- ProfitRate: 参与质押的分润比例



#### 二、Storage: 存储数据结构

1. Map nftId -> nft详情， 用于存储所有nft

```rust
pub Nfts: map hasher(twox_64_concat) T::NftId => Option<Nft>;
```

2. Map nftId -> 账户Id， 用于记录nft所有者

```rust
pub NftAccount: map hasher(twox_64_concat) T::NftId => T::AccountId;
```

3. Map nftId -> 订单Id， 用于记录Nft对应的订单数据

```rust
pub NftOrder: map hasher(twox_64_concat) T::NftId => Option<T::OrderId>;
```

4. Map  订单Id -> 订单详情, 用于存储所有待完成的拍卖订单

```rust
pub Orders: map hasher(twox_64_concat) T::OrderId => Option<OrderOf<T>>;
```

5. Map 订单Id -> 当前最大出价，用于存储当前订单的最大出价

```rust
pub Bids: map hasher(twox_64_concat) T::OrderId => Option<BidOf<T>>;
```

6. Map 订单Id -> 质押投票列表, 用于存储质押列表

```rust
pub Votes: map hasher(twox_64_concat) T::OrderId => Vec<VoteOf<T>>;
```

7. NftId生成器，递增

```rust
pub NextNftId: T::NftId;
```

8. 拍卖订单Id生成器，递增

```
pub NextOrderId: T::OrderId;
```



#### 三、Module: 交易接口

1. 创建Nft艺术品

title: 标题

url: 链接

desc: 详情

```rust
pub fn create(origin, title: Vec<u8>, url: Vec<u8>, desc: Vec<u8>)
```

2. 移除Nft

nft_id: 艺术品Id

```rust
pub fn remove(origin, nft_id: T::NftId)
```

3. 转移Nft艺术品

target: 转移目标

nft_id: 艺术品Id

```rust
pub fn transfer(origin, target: T::AccountId, nft_id: T::NftId)
```

4. 下拍卖单出售艺术品

nft_id: 艺术品Id

start_price: 起拍价格

end_price: 最大价格

keep_block_num: 拍卖最大保留区块数量

```rust
pub fn order_sell(origin, nft_id: T::NftId, start_price: BalanceOf<T>, end_price: BalanceOf<T>, keep_block_num: T::BlockNumber)
```

5. 竞拍Nft艺术品

order_id: 订单Id

price: 竞拍价格

```rust
pub fn order_buy(origin, order_id: T::OrderId, price: BalanceOf<T>)
```

6. 主动结算拍卖 // 用于到期结算

order_id: 订单Id

```rust
pub fn order_settlement(origin, order_id: T::OrderId)
```

7. 进行投票质押

order_id: 订单Id

amount: 质押数量

```rust
pub fn vote_order(origin, order_id: T::OrderId, amount: BalanceOf<T>)
```























