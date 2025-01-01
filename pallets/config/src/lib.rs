// This file is part of CORD – https://cord.network

// Copyright (C) Dhiway Networks Pvt. Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later

// CORD is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// CORD is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with CORD. If not, see <https://www.gnu.org/licenses/>.

//! # Network Config Information

#![warn(unused_extern_crates)]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;
use alloc::{str, vec::Vec};
use codec::{Decode, Encode, MaxEncodedLen};
use cord_primitives::{Id as NetworkId, NetworkInfoProvider};
use cord_uri::{Identifier, Ss58Identifier};
use fluent_uri::Uri;
use frame_support::{
	pallet_prelude::*,
	traits::{Get, StorageVersion},
	BoundedVec,
};
use frame_system::pallet_prelude::BlockNumberFor;
use frame_system::pallet_prelude::*;
use sp_runtime::traits::{Hash, Zero};

pub use pallet::*;

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum NetworkType {
	Test,
	Production,
}

impl Default for NetworkType {
	fn default() -> Self {
		NetworkType::Test
	}
}

/// Identifier
pub type IdentifierOf = Ss58Identifier;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	/// The current storage version.
	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config + cord_uri::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type CordIdentifier: Identifier;
		#[pallet::constant]
		type DefaultNetworkId: Get<u32>;
	}

	#[derive(
		Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, Default, MaxEncodedLen,
	)]
	pub struct NetworkInfo {
		pub id: NetworkId,
		pub name: BoundedVec<u8, ConstU32<64>>,
		pub mode: NetworkType,
		pub endpoints: BoundedVec<BoundedVec<u8, ConstU32<256>>, ConstU32<50>>,
		pub website: Option<BoundedVec<u8, ConstU32<256>>>,
	}

	#[pallet::error]
	pub enum Error<T> {
		NetworkInfoNotFound,
		InvalidInput,
		InvalidUri,
		InvalidIdentifierLength,
		NetworkConfigAlreadyAdded,
	}

	#[pallet::storage]
	pub type NetworkConfigInfo<T: Config> =
		StorageMap<_, Blake2_128Concat, IdentifierOf, NetworkInfo, OptionQuery>;

	#[pallet::storage]
	pub type NetworkIdToIdentifier<T: Config> =
		StorageMap<_, Blake2_128Concat, NetworkId, IdentifierOf, OptionQuery>;

	#[pallet::storage]
	pub(super) type NetworkIdentifier<T: Config> = StorageValue<_, NetworkId, ValueQuery>;

	// Stores the network configuration type
	#[pallet::storage]
	pub type NetworkPermissioned<T> = StorageValue<_, bool, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Network Info added.
		NetworkInfo {
			identifier: IdentifierOf,
			network: NetworkId,
		},
		NetworkNameUpdate {
			identifier: IdentifierOf,
		},
		NetworkRpcUpdate {
			identifier: IdentifierOf,
		},
		NetworkWebUpdate {
			identifier: IdentifierOf,
		},
		NetworkRpcRemove {
			identifier: IdentifierOf,
		},
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		#[serde(skip)]
		pub _config: core::marker::PhantomData<T>,
		pub permissioned: bool,
		pub network_id: NetworkId,
	}

	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self {
				network_id: NetworkId::from(1000u32),
				permissioned: false,
				_config: Default::default(),
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			NetworkIdentifier::<T>::put(self.network_id);
			NetworkPermissioned::<T>::put(&self.permissioned);
			cord_uri::Pallet::<T>::set_network_id(self.network_id);
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight({200_000})]
		pub fn network_info(
			origin: OriginFor<T>,
			name: Vec<u8>,
			mode: NetworkType,
			endpoints: Vec<Vec<u8>>,
			website: Option<Vec<u8>>,
		) -> DispatchResult {
			ensure_root(origin)?;
			ensure!(
				NetworkConfigInfo::<T>::iter_keys().next().is_none(),
				Error::<T>::NetworkConfigAlreadyAdded
			);

			let pallet_name = <Self as PalletInfoAccess>::name();
			let genesis_hash = <frame_system::Pallet<T>>::block_hash(BlockNumberFor::<T>::zero());

			let network_id = NetworkIdentifier::<T>::get();

			let name_bounded = BoundedVec::<u8, ConstU32<64>>::try_from(name)
				.map_err(|_| Error::<T>::InvalidInput)?;

			// Id Digest = concat (H(<scale_encoded_genesis_hash>, <scale_encoded_network_name>, <scale_encoded_network_type>,))
			let digest = <T as frame_system::Config>::Hashing::hash(
				&[
					&genesis_hash.encode()[..],
					&name_bounded.encode()[..],
					&mode.encode()[..],
					&network_id.encode()[..],
				]
				.concat()[..],
			);

			let identifier = T::CordIdentifier::build(&(digest).encode()[..], pallet_name)
				.map_err(|_| Error::<T>::InvalidIdentifierLength)?;

			// let identifier = <Ss58Identifier as cord_uri::IdentifierCreator>::cr_encode::<T>(
			// 	&(digest).encode()[..],
			// 	pallet_name,
			// )
			// .map_err(|_| Error::<T>::InvalidIdentifierLength)?;
			// // let identifier = Ss58Identifier::cr_encode::<T>(&(digest).encode()[..], pallet_name)
			// 	.map_err(|_| Error::<T>::InvalidIdentifierLength)?;

			let rpc_endpoints_vec: Vec<BoundedVec<u8, ConstU32<256>>> = endpoints
				.into_iter()
				.map(|uri| {
					let uri_bounded = BoundedVec::<u8, ConstU32<256>>::try_from(uri)
						.map_err(|_| Error::<T>::InvalidInput)?;
					if Self::validate_rpc_bounded_uri(&uri_bounded) {
						Ok(uri_bounded)
					} else {
						Err(Error::<T>::InvalidUri)
					}
				})
				.collect::<Result<Vec<BoundedVec<u8, ConstU32<256>>>, _>>()?;

			let rpc_endpoints = BoundedVec::<_, ConstU32<50>>::try_from(rpc_endpoints_vec)
				.map_err(|_| Error::<T>::InvalidInput)?;

			let website = match website {
				Some(uri) => {
					let bounded_home = BoundedVec::<u8, ConstU32<256>>::try_from(uri)
						.map_err(|_| Error::<T>::InvalidInput)?;
					if Self::validate_http_bounded_uri(&bounded_home) {
						Some(bounded_home)
					} else {
						return Err(Error::<T>::InvalidUri.into());
					}
				},
				None => None,
			};

			let network_info = NetworkInfo {
				id: network_id,
				name: name_bounded,
				mode,
				endpoints: rpc_endpoints,
				website,
			};

			NetworkConfigInfo::<T>::insert(identifier.clone(), network_info.clone());
			NetworkIdToIdentifier::<T>::insert(network_id.clone(), identifier.clone());

			Self::deposit_event(Event::NetworkInfo { identifier, network: network_id });

			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight({200_000})]
		pub fn update_name(origin: OriginFor<T>, name: Vec<u8>) -> DispatchResult {
			ensure_root(origin)?;

			let identifier = NetworkConfigInfo::<T>::iter_keys()
				.next()
				.ok_or(Error::<T>::NetworkInfoNotFound)?;

			let mut network_info =
				NetworkConfigInfo::<T>::get(&identifier).ok_or(Error::<T>::NetworkInfoNotFound)?;

			let bounded_name = BoundedVec::<u8, ConstU32<64>>::try_from(name)
				.map_err(|_| Error::<T>::InvalidInput)?;
			network_info.name = bounded_name.clone();

			NetworkConfigInfo::<T>::insert(&identifier, network_info);

			Self::deposit_event(Event::NetworkNameUpdate { identifier: identifier.clone() });

			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight({200_000})]
		pub fn update_rpc_endpoints(
			origin: OriginFor<T>,
			endpoints: Vec<Vec<u8>>,
		) -> DispatchResult {
			ensure_root(origin)?;

			let identifier = NetworkConfigInfo::<T>::iter_keys()
				.next()
				.ok_or(Error::<T>::NetworkInfoNotFound)?;

			let mut network_info =
				NetworkConfigInfo::<T>::get(&identifier).ok_or(Error::<T>::NetworkInfoNotFound)?;

			let rpc_endpoints_vec: Vec<BoundedVec<u8, ConstU32<256>>> = endpoints
				.into_iter()
				.map(|uri| {
					let uri_bounded = BoundedVec::<u8, ConstU32<256>>::try_from(uri)
						.map_err(|_| Error::<T>::InvalidInput)?;
					if Self::validate_rpc_bounded_uri(&uri_bounded) {
						Ok(uri_bounded)
					} else {
						Err(Error::<T>::InvalidUri)
					}
				})
				.collect::<Result<Vec<BoundedVec<u8, ConstU32<256>>>, _>>()?;

			network_info.endpoints = BoundedVec::<_, ConstU32<50>>::try_from(rpc_endpoints_vec)
				.map_err(|_| Error::<T>::InvalidInput)?;

			NetworkConfigInfo::<T>::insert(&identifier, network_info);

			Self::deposit_event(Event::NetworkRpcUpdate { identifier: identifier.clone() });

			Ok(())
		}

		#[pallet::call_index(3)]
		#[pallet::weight({200_000})]
		pub fn update_website(origin: OriginFor<T>, website: Option<Vec<u8>>) -> DispatchResult {
			ensure_root(origin)?;

			let identifier = NetworkConfigInfo::<T>::iter_keys()
				.next()
				.ok_or(Error::<T>::NetworkInfoNotFound)?;

			let mut network_info =
				NetworkConfigInfo::<T>::get(&identifier).ok_or(Error::<T>::NetworkInfoNotFound)?;

			network_info.website = match website {
				Some(uri) => {
					let bounded_web = BoundedVec::<u8, ConstU32<256>>::try_from(uri)
						.map_err(|_| Error::<T>::InvalidInput)?;
					if Self::validate_http_bounded_uri(&bounded_web) {
						Some(bounded_web.clone())
					} else {
						return Err(Error::<T>::InvalidUri.into());
					}
				},
				None => None,
			};

			NetworkConfigInfo::<T>::insert(&identifier, network_info);

			Self::deposit_event(Event::NetworkWebUpdate { identifier: identifier.clone() });

			Ok(())
		}

		#[pallet::call_index(4)]
		#[pallet::weight({200_000})]
		pub fn remove_rpc_endpoint(origin: OriginFor<T>, endpoint: Vec<u8>) -> DispatchResult {
			ensure_root(origin)?;

			let identifier = NetworkConfigInfo::<T>::iter_keys()
				.next()
				.ok_or(Error::<T>::NetworkInfoNotFound)?;

			let mut network_info =
				NetworkConfigInfo::<T>::get(&identifier).ok_or(Error::<T>::NetworkInfoNotFound)?;

			ensure!(network_info.endpoints.len() > 1, Error::<T>::InvalidInput);

			let uri_bounded = BoundedVec::<u8, ConstU32<256>>::try_from(endpoint)
				.map_err(|_| Error::<T>::InvalidInput)?;

			network_info.endpoints.retain(|uri| uri != &uri_bounded);

			NetworkConfigInfo::<T>::insert(&identifier, network_info);

			Self::deposit_event(Event::NetworkRpcRemove { identifier: identifier.clone() });

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub(crate) fn validate_rpc_uri(uri: &str) -> bool {
			match Uri::parse(uri) {
				Ok(parsed_uri) => {
					let scheme = parsed_uri.scheme().map(|s| s.as_str());
					matches!(scheme, Some("ws") | Some("wss"))
				},
				Err(_) => false,
			}
		}

		pub(crate) fn validate_http_uri(uri: &str) -> bool {
			match Uri::parse(uri) {
				Ok(parsed_uri) => {
					let scheme = parsed_uri.scheme().map(|s| s.as_str());
					matches!(scheme, Some("http") | Some("https"))
				},
				Err(_) => false,
			}
		}

		pub(crate) fn validate_rpc_bounded_uri(uri: &BoundedVec<u8, ConstU32<256>>) -> bool {
			if let Ok(uri_str) = str::from_utf8(uri) {
				Self::validate_rpc_uri(uri_str)
			} else {
				false
			}
		}

		pub(crate) fn validate_http_bounded_uri(uri: &BoundedVec<u8, ConstU32<256>>) -> bool {
			if let Ok(uri_str) = str::from_utf8(uri) {
				Self::validate_http_uri(uri_str)
			} else {
				false
			}
		}
	}
}

impl<T: Config> NetworkInfoProvider for Pallet<T> {
	fn get_network_id() -> NetworkId {
		NetworkIdentifier::<T>::get()
	}
}
impl<T: Config> Pallet<T> {
	/// check if the network is permissioned
	pub fn is_permissioned() -> bool {
		NetworkPermissioned::<T>::get()
	}
}

impl<T: Config> cord_primitives::IsPermissioned for Pallet<T> {
	fn is_permissioned() -> bool {
		Self::is_permissioned()
	}
}
