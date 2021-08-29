#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::inherent::Vec;
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[derive(Encode, Decode, Default, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Resource<BlockNumber> {
	pub resource_hash: Vec<u8>,
	pub ttl: BlockNumber,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{
		dispatch::DispatchResult, pallet_prelude::*, sp_runtime::traits::Saturating,
	};
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(_: T::BlockNumber) -> Weight {
			0
		}

		fn on_finalize(now: T::BlockNumber) {
			// we remove expired keys
			let expiry_info = <ResouceExpiry<T>>::get();
			let mut pruned_expiry_info: Vec<Resource<T::BlockNumber>> = Vec::new();

			for item in expiry_info {
				if item.ttl > now {
					pruned_expiry_info.push(item);
				} else {
					// clean up keys from map
					<ResourceToAccountMapping<T>>::remove(item.resource_hash);
				}
			}

			// it will already be sorted since we have put it in sorted manner
			<ResouceExpiry<T>>::put(pruned_expiry_info);
		}
	}
	#[pallet::storage]
	#[pallet::getter(fn get_ownership)]
	pub(super) type ResourceToAccountMapping<T: Config> =
		StorageMap<_, Blake2_128Concat, Vec<u8>, T::AccountId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_resource_expiry)]
	pub(super) type ResouceExpiry<T: Config> =
		StorageValue<_, Vec<Resource<T::BlockNumber>>, ValueQuery>;

	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Created(Vec<u8>, T::BlockNumber, T::AccountId),
		Updated(Vec<u8>, T::BlockNumber, T::AccountId),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// ResourceAlreadyCreated.
		ResourceAlreadyCreated,
		/// ResourceNotFound.
		ResourceNotFound,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn create_ownership(
			origin: OriginFor<T>,
			resource_hash: Vec<u8>,
			ttl: T::BlockNumber,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// if already exist, then dont create
			ensure!(
				!<ResourceToAccountMapping<T>>::contains_key(&resource_hash),
				<Error<T>>::ResourceAlreadyCreated
			);
			<ResourceToAccountMapping<T>>::insert(&resource_hash, &who);

			// store the ttl info as well
			let expiry_block_number = <frame_system::Pallet<T>>::block_number().saturating_add(ttl);

			// TODO : Implement binary search here
			let mut expiry_info = <ResouceExpiry<T>>::get();

			//let mut vec: Vec<u8> = Vec::new();
			//vec.copy_from_slice(&resource_hash[..]);
			let resource =
				Resource { resource_hash: resource_hash.clone(), ttl: expiry_block_number };
			expiry_info.push(resource);
			// sort it by ttl, NlogN operation
			expiry_info.sort_by(|lhs, rhs| lhs.ttl.cmp(&rhs.ttl));
			<ResouceExpiry<T>>::put(expiry_info);

			Self::deposit_event(Event::Created(resource_hash, expiry_block_number, who));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn update_ownership(
			origin: OriginFor<T>,
			resource_hash: Vec<u8>,
			ttl: T::BlockNumber,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// if already exist, then dont create
			ensure!(
				<ResourceToAccountMapping<T>>::contains_key(&resource_hash),
				<Error<T>>::ResourceNotFound
			);

			// update the account mapping
			<ResourceToAccountMapping<T>>::insert(&resource_hash, &who);

			// update the ttl
			let new_expiry_block_number =
				<frame_system::Pallet<T>>::block_number().saturating_add(ttl);

			let mut expiry_info = <ResouceExpiry<T>>::get();

			for item in &mut expiry_info {
				if item.resource_hash == resource_hash {
					(*item).ttl = new_expiry_block_number;
				}
			}
			// sort it by ttl, NlogN operation
			expiry_info.sort_by(|lhs, rhs| lhs.ttl.cmp(&rhs.ttl));
			<ResouceExpiry<T>>::put(expiry_info);

			Self::deposit_event(Event::Updated(resource_hash, new_expiry_block_number, who));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}
	}
}
