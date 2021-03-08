#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Encode, Decode};
use frame_support::{ensure, decl_module, decl_storage, decl_event, decl_error, dispatch, traits::{Get, Currency, ReservableCurrency, ExistenceRequirement}, Parameter};
use frame_system::ensure_signed;
use sp_runtime::{
	DispatchResult, RuntimeDebug,
	traits::{AtLeast32BitUnsigned, MaybeSerializeDeserialize, Bounded, One, CheckedAdd, Zero},
};
use sp_std::prelude::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub trait Trait: frame_system::Trait {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
	type NftId: Parameter + AtLeast32BitUnsigned + Default + Copy + MaybeSerializeDeserialize + Bounded;
	type OrderId: Parameter + AtLeast32BitUnsigned + Default + Copy + MaybeSerializeDeserialize + Bounded;
	type Currency: ReservableCurrency<Self::AccountId>;
}

#[derive(Encode, Decode, Clone, RuntimeDebug, Eq, PartialEq)]
pub struct Order<OrderId, NftId, AccountId, Balance> {
	pub order_id: OrderId,
	#[codec(compact)]
	pub start_price: Balance,
	pub end_price: Balance,
	pub nft_id: NftId,
	pub owner: AccountId,
}

#[derive(Encode, Decode, Clone, RuntimeDebug, Eq, PartialEq)]
pub struct Bid<OrderId, AccountId, Balance> {
	pub order_id: OrderId,
	#[codec(compact)]
	pub price: Balance,
	pub owner: AccountId,
}

type Nft = Vec<u8>;
type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;
type OrderOf<T> = Order<<T as Trait>::OrderId, <T as Trait>::NftId, <T as frame_system::Trait>::AccountId, BalanceOf<T>>;
type BidOf<T> = Bid<<T as Trait>::OrderId, <T as frame_system::Trait>::AccountId, BalanceOf<T>>;

decl_storage! {
	trait Store for Module<T: Trait> as NftModule {
		pub Nfts: map hasher(twox_64_concat) T::NftId => Nft;
		pub NftAccount: map hasher(twox_64_concat) T::NftId => T::AccountId;
		pub NextNftId: T::NftId;

		pub NextOrderId: T::OrderId;
		pub Orders: map hasher(twox_64_concat) T::OrderId => Option<OrderOf<T>>;
		pub Bids: map hasher(twox_64_concat) T::OrderId => Option<BidOf<T>>;
		pub NftOrder: map hasher(twox_64_concat) T::NftId => Option<T::OrderId>;
	}
}

decl_event!(
	pub enum Event<T> where
		<T as Trait>::NftId,
		<T as Trait>::OrderId,
		Order = OrderOf<T>,
		Bid = BidOf<T>,
		AccountId = <T as frame_system::Trait>::AccountId,
	{
		NftCreated(AccountId, NftId),
		NftRemove(AccountId, NftId),
		NftTransfer(AccountId, AccountId, NftId),

		OrderSell(AccountId, Order),
		OrderBuy(AccountId, Bid),

		OrderComplete(AccountId, OrderId),
	}
);

decl_error! {
	pub enum Error for Module<T: Trait> {
		NftIdNotExist,
		NftIdOverflow,
		NotNftOwner,
		NftOrderExist,
		OrderNotExist,
		OrderPriceIllegal,
		OrderPriceTooSmall,
		OrderIdOverflow,
		OrderIdNotExist,
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		fn deposit_event() = default;

		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		pub fn create(origin, url: Vec<u8>) -> dispatch::DispatchResult {
			let who = ensure_signed(origin)?;
			NextNftId::<T>::try_mutate(|id| -> DispatchResult {
				let nft_id = *id;
				*id = id.checked_add(&One::one()).ok_or(Error::<T>::NftIdOverflow)?;
				// 创建nft并建立 nft索引、账户索引
				Nfts::<T>::insert(nft_id, &url);
				NftAccount::<T>::insert(nft_id, who.clone());
				Self::deposit_event(RawEvent::NftCreated(who, nft_id));
				Ok(())
			})?;
			Ok(())
		}

		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		pub fn remove(origin, nft_id: T::NftId) -> dispatch::DispatchResult {
			let who = ensure_signed(origin)?;
			// 检查nft是否存在
			ensure!(Nfts::<T>::contains_key(&nft_id), Error::<T>::NftIdNotExist);

			let owner = NftAccount::<T>::get(&nft_id);
			// 检查nft所有者
			ensure!(owner == who, Error::<T>::NotNftOwner);
			// 检查nft是否处于订单中
			ensure!(!NftOrder::<T>::contains_key(&nft_id), Error::<T>::NftOrderExist);

			// 移除nft的两个索引
			NftAccount::<T>::remove(nft_id);
			Nfts::<T>::remove(nft_id);

			Self::deposit_event(RawEvent::NftRemove(who, nft_id));
			Ok(())
		}

		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		pub fn transfer(origin, target: T::AccountId, nft_id: T::NftId) -> dispatch::DispatchResult {
			let who = ensure_signed(origin)?;
			// 检查nft是否存在
			ensure!(Nfts::<T>::contains_key(&nft_id), Error::<T>::NftIdNotExist);

			// 检查nft的所有者
			let owner = NftAccount::<T>::get(&nft_id);
			ensure!(owner == who, Error::<T>::NotNftOwner);

			// 检查nft是否处于订单中
			ensure!(!NftOrder::<T>::contains_key(&nft_id), Error::<T>::NftOrderExist);

			// 更改nft账户索引
			NftAccount::<T>::insert(nft_id, target.clone());
			Self::deposit_event(RawEvent::NftTransfer(who, target, nft_id));
			Ok(())
		}

		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		pub fn order_sell(origin, nft_id: T::NftId, start_price: BalanceOf<T>, end_price: BalanceOf<T>) -> dispatch::DispatchResult {
			let who = ensure_signed(origin)?;
			// 检查nft是否存在
			ensure!(Nfts::<T>::contains_key(&nft_id), Error::<T>::NftIdNotExist);

			// 检查nft的所有者
			let owner = NftAccount::<T>::get(&nft_id);
			ensure!(owner == who, Error::<T>::NotNftOwner);

			// 检查nft是否处于订单中
			ensure!(!NftOrder::<T>::contains_key(&nft_id), Error::<T>::NftOrderExist);

			// 检查价格是否合法
			ensure!(start_price <= end_price, Error::<T>::OrderPriceIllegal);

			// 创建订单
			NextOrderId::<T>::try_mutate(|id| -> DispatchResult {
				let order_id = *id;
				let order = Order {
					order_id,
					start_price,
					end_price,
					nft_id,
					owner: who.clone(),
				};
				*id = id.checked_add(&One::one()).ok_or(Error::<T>::OrderIdOverflow)?;
				// 插入订单索引
				Orders::<T>::insert(order_id, order.clone());
				NftOrder::<T>::insert(nft_id, order_id);
				Self::deposit_event(RawEvent::OrderSell(who, order));
				Ok(())
			})?;
			Ok(())
		}

		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		pub fn order_buy(origin, order_id: T::OrderId, price: BalanceOf<T>) -> dispatch::DispatchResult {
			let who = ensure_signed(origin)?;

			// 检查订单是否存在
			let order: OrderOf<T> = Orders::<T>::get(order_id).ok_or(Error::<T>::OrderNotExist)?;

			// 检查价格是否合法
			ensure!(order.start_price <= price, Error::<T>::OrderPriceTooSmall);

			// 检查是否比上个竞价要大
			let bidopt: Option<BidOf<T>> = Bids::<T>::get(order_id);
			if let Some(bid) = bidopt {
				ensure!(bid.price < price, Error::<T>::OrderPriceTooSmall);
			}

			// 检查是否到了最大价格
			if price >= order.end_price {
				// 达到最大价格，拍卖成功
				let owner = NftAccount::<T>::get(&order.nft_id);
				// 进行转账
				T::Currency::transfer(&who, &owner, order.end_price, ExistenceRequirement::KeepAlive)?;
				// 移除订单索引
				Orders::<T>::remove(order_id);
				NftOrder::<T>::remove(order.nft_id);
				// 更新nft账户索引
				NftAccount::<T>::insert(order.nft_id, who.clone());
				// 移除之前的bid
				Self::clean_order_bid(order_id)?;
				Self::deposit_event(RawEvent::OrderComplete(who, order_id));
			} else {
				// 参与竞价
				// 锁定价格
				T::Currency::reserve(&who, price)?;
				// 移除之前的bid
				Self::clean_order_bid(order_id)?;
				// 创建新的bid
				let bid = Bid {
					order_id,
					price,
					owner: who.clone()
				};
				Bids::<T>::insert(order_id, bid.clone());
				Self::deposit_event(RawEvent::OrderBuy(who, bid));
			}
			Ok(())
		}
	}
}

impl<T: Trait> Module<T> {
	pub fn clean_order_bid(order_id: T::OrderId) -> dispatch::DispatchResult {
		let bid_opt: Option<BidOf<T>> = Bids::<T>::get(order_id);
		if let Some(bid) = bid_opt {
			// 解锁之前的锁定的钱
			T::Currency::unreserve(&bid.owner, bid.price);
		}
		Ok(())
	}
}