#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{ensure, decl_module, decl_storage, decl_event, decl_error, dispatch, traits::Get, Parameter};
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

type Nft = Vec<u8>;

pub trait Trait: frame_system::Trait {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
	type NftId: Parameter + AtLeast32BitUnsigned + Default + Copy + MaybeSerializeDeserialize + Bounded;
}

decl_storage! {
	trait Store for Module<T: Trait> as NftModule {
		pub Nfts: map hasher(twox_64_concat) T::NftId => Nft;
		pub NftAccount: map hasher(twox_64_concat) T::NftId => T::AccountId;
		pub NextNftId: T::NftId;
	}
}

decl_event!(
	pub enum Event<T> where
		<T as Trait>::NftId,
		AccountId = <T as frame_system::Trait>::AccountId
	{
		NftCreated(AccountId, NftId),
		NftRemove(AccountId, NftId),
		NftTransfer(AccountId, AccountId, NftId),
	}
);

decl_error! {
	pub enum Error for Module<T: Trait> {
		NftIdNotExist,
		NftIdOverflow,
		NotNftOwner
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
			ensure!(Nfts::<T>::contains_key(&nft_id), Error::<T>::NftIdNotExist);

			let owner = NftAccount::<T>::get(&nft_id);
			ensure!(owner == who, Error::<T>::NotNftOwner);

			NftAccount::<T>::remove(nft_id);
			Nfts::<T>::remove(nft_id);

			Self::deposit_event(RawEvent::NftRemove(who, nft_id));
			Ok(())
		}

		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		pub fn transfer(origin, target: T::AccountId, nft_id: T::NftId) -> dispatch::DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(Nfts::<T>::contains_key(&nft_id), Error::<T>::NftIdNotExist);

			let owner = NftAccount::<T>::get(&nft_id);
			ensure!(owner == who, Error::<T>::NotNftOwner);

			NftAccount::<T>::insert(nft_id, target.clone());
			Self::deposit_event(RawEvent::NftTransfer(who, target, nft_id));
			Ok(())
		}
	}
}
