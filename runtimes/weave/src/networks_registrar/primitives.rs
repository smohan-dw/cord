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

use super::*;
use crate::*;
use codec::{CompactAs, Decode, Encode, MaxEncodedLen};
use core::sync::atomic::{AtomicU16, Ordering};
use networks_registrar::CordAccountOf;
use scale_info::TypeInfo;
use sp_core::{RuntimeDebug, TypeId};

/// Unique identifier of a network.
#[derive(
	Clone,
	CompactAs,
	Copy,
	Decode,
	Default,
	Encode,
	Eq,
	Hash,
	MaxEncodedLen,
	Ord,
	PartialEq,
	PartialOrd,
	RuntimeDebug,
	serde::Serialize,
	serde::Deserialize,
	TypeInfo,
)]
#[cfg_attr(feature = "std", derive(derive_more::Display))]
pub struct Id(u32);

impl TypeId for Id {
	const TYPE_ID: [u8; 4] = *b"para";
}

impl From<Id> for u32 {
	fn from(x: Id) -> Self {
		x.0
	}
}

impl From<u32> for Id {
	fn from(x: u32) -> Self {
		Id(x)
	}
}

impl From<usize> for Id {
	fn from(x: usize) -> Self {
		// can't panic, so need to truncate
		let x = x.try_into().unwrap_or(u32::MAX);
		Id(x)
	}
}

// When we added a second From impl for Id, type inference could no longer
// determine which impl should apply for things like `5.into()`. It therefore
// raised a bunch of errors in our test code, scattered throughout the
// various modules' tests, that there is no impl of `From<i32>` (`i32` being
// the default numeric type).
//
// We can't use `cfg(test)` here, because that configuration directive does not
// propagate between crates, which would fail to fix tests in crates other than
// this one.
//
// Instead, let's take advantage of the observation that what really matters for a
// ParaId within a test context is that it is unique and constant. I believe that
// there is no case where someone does `(-1).into()` anyway, but if they do, it
// never matters whether the actual contained ID is `-1` or `4294967295`. Nobody
// does arithmetic on a `ParaId`; doing so would be a bug.
impl From<i32> for Id {
	fn from(x: i32) -> Self {
		Id(x as u32)
	}
}

// System parachain ID is considered `< 2000`.
// const SYSTEM_INDEX_END: u32 = 1999;
const PUBLIC_INDEX_START: u32 = 2000;

/// The ID of the first publicly registrable parachain.
pub const LOWEST_PUBLIC_ID: Id = Id(PUBLIC_INDEX_START);

impl Id {
	/// Create an `Id`.
	pub const fn new(id: u32) -> Self {
		Self(id)
	}
}

impl core::ops::Add<u32> for Id {
	type Output = Self;

	fn add(self, other: u32) -> Self {
		Self(self.0 + other)
	}
}

impl core::ops::Sub<u32> for Id {
	type Output = Self;

	fn sub(self, other: u32) -> Self {
		Self(self.0 - other)
	}
}

impl Id {
	pub fn inner(&self) -> u32 {
		self.0
	}
}

#[derive(Encode, Decode, MaxEncodedLen, PartialEq, Eq, Clone)]
pub struct NetworkToken<T: Config>(pub(crate) BoundedVec<u8, ConstU32<142>>, PhantomData<T>);

const PREFIX: &[u8] = b"RSRVIDV01";
const R_IDENT: u16 = 24;
const N_IDENT: u16 = 21;

impl<T: Config> NetworkToken<T> {
	/// Generate a checksum using Blake2b hash.
	fn checksum(data: &[u8]) -> Vec<u8> {
		use blake2::{Blake2b512, Digest};

		let mut hasher = Blake2b512::new();
		hasher.update(PREFIX);
		hasher.update(data);
		hasher.finalize().to_vec()
	}

	/// Generate a Base58-encoded proof.
	pub fn generate(
		network_id: NetworkId,
		cord_genesis_hash: &HashOf<T>,
		network_genesis_hash: &HashOf<T>,
		account_id: &CordAccountOf<T>,
		reserve: bool,
	) -> Result<Self, Error<T>> {
		use bs58;

		let ident_value = if reserve { R_IDENT } else { N_IDENT };

		let ident = Self::compact_encode(ident_value & 0b0011_1111_1111_1111)?;
		let nid_inner: u16 = network_id.inner() as u16 & 0b0011_1111_1111_1111;
		let nid = Self::compact_encode(nid_inner)?;

		let mut buffer = Vec::new();
		buffer.extend(ident);
		buffer.extend(cord_genesis_hash.as_ref());
		buffer.extend(nid);
		buffer.extend(network_genesis_hash.as_ref());
		buffer.extend(account_id.encode());

		let checksum = &Self::checksum(&buffer)[..2];
		buffer.extend(checksum);

		let encoded = bs58::encode(&buffer).into_string();
		log::info!("Encoded token: {}", encoded);

		Ok(Self(
			Vec::<u8>::from(encoded).try_into().map_err(|_| Error::<T>::InvalidToken)?,
			PhantomData,
		))
	}

	pub fn resolve(&self) -> Result<(NetworkId, HashOf<T>, HashOf<T>, CordAccountOf<T>), Error<T>> {
		use bs58;

		// Decode the Base58-encoded token
		let decoded = bs58::decode(&self.0).into_vec().map_err(|_| Error::<T>::InvalidToken)?;

		log::info!("Decoded token: {:?}", decoded);
		ensure!(decoded.len() >= 2 && decoded.len() <= 142, Error::<T>::InvalidToken);

		let (ident, mut offset) = Self::compact_decode(&decoded)?;
		ensure!(ident == N_IDENT || ident == R_IDENT, Error::<T>::InvalidPrefix);
		log::info!("Decoded ident: {}", ident);

		let cord_genesis_hash_end = offset + 32;
		ensure!(cord_genesis_hash_end <= decoded.len(), Error::<T>::InvalidCordGenesisHead);
		let cord_genesis_hash_bytes = &decoded[offset..cord_genesis_hash_end];
		let cord_genesis_hash = HashOf::<T>::decode(&mut &*cord_genesis_hash_bytes)
			.map_err(|_| Error::<T>::InvalidCordGenesisHead)?;
		offset = cord_genesis_hash_end;
		log::info!("Decoded cord_genesis_hash: {:?}", cord_genesis_hash);

		let (nid, nid_offset) = Self::compact_decode(&decoded[offset..])?;
		offset += nid_offset;
		log::info!("Decoded network ID (nid): {}", nid);

		let network_genesis_hash_end = offset + 32;
		ensure!(network_genesis_hash_end <= decoded.len(), Error::<T>::InvalidNetworkGenesisHead);
		let network_genesis_hash_bytes = &decoded[offset..network_genesis_hash_end];

		let network_genesis_hash = HashOf::<T>::decode(&mut &*network_genesis_hash_bytes)
			.map_err(|_| Error::<T>::InvalidNetworkGenesisHead)?;

		offset = network_genesis_hash_end;
		log::info!("Decoded network_genesis_hash: {:?}", network_genesis_hash);

		// Extract `account_id` (32 bytes)
		let account_id_end = offset + 32;
		ensure!(account_id_end <= decoded.len(), Error::<T>::InvalidAccountId);
		let account_id_bytes = &decoded[offset..account_id_end];
		let account_id = CordAccountOf::<T>::decode(&mut &*account_id_bytes)
			.map_err(|_| Error::<T>::InvalidAccountId)?;
		// offset = account_id_end;
		log::info!("Decoded account_id: {:?}", account_id);

		// Verify checksum
		let checksum = &decoded[decoded.len() - 2..];
		let expected_checksum = &Self::checksum(&decoded[..decoded.len() - 2])[..2];
		ensure!(checksum == expected_checksum, Error::<T>::InvalidChecksum);
		log::info!("Checksum: {:?}, Expected checksum: {:?}", checksum, expected_checksum);

		// Log successful extraction
		log::info!(
            "Token successfully verified and extracted: reserve_id={}, cord_genesis_hash={:?}, network_genesis_hash={:?}, account_id={:?}",
            nid,
            cord_genesis_hash,
            network_genesis_hash,
            account_id
        );

		// Return extracted elements
		Ok((NetworkId::from(nid as u32), cord_genesis_hash, network_genesis_hash, account_id))
	}

	fn compact_encode(value: u16) -> Result<Vec<u8>, Error<T>> {
		match value {
			0..=63 => Ok(vec![value as u8]),
			64..=16_383 => {
				let first = ((value & 0b0000_0000_1111_1100) as u8) >> 2;
				let second = ((value >> 8) as u8) | ((value & 0b0000_0000_0000_0011) as u8) << 6;
				Ok(vec![first | 0b01000000, second])
			},
			_ => Err(Error::<T>::InvalidPrefix),
		}
	}

	fn compact_decode(data: &[u8]) -> Result<(u16, usize), Error<T>> {
		match data[0] {
			0..=63 => Ok((data[0] as u16, 1)),
			64..=127 => {
				ensure!(data.len() >= 2, Error::<T>::InvalidPrefix);
				let lower = (data[0] << 2) | (data[1] >> 6);
				let upper = data[1] & 0b0011_1111;
				Ok(((lower as u16) | ((upper as u16) << 8), 2))
			},
			_ => Err(Error::<T>::InvalidPrefix),
		}
	}
}

impl<T: Config> scale_info::TypeInfo for NetworkToken<T> {
	type Identity = Self;

	fn type_info() -> scale_info::Type {
		scale_info::Type::builder()
			.path(scale_info::Path::new("NetworkToken", module_path!()))
			.type_params(vec![scale_info::TypeParameter::new("T", None)])
			.composite(
				scale_info::build::Fields::unnamed()
					.field(|f| f.ty::<BoundedVec<u8, ConstU32<142>>>()),
			)
	}
}

impl<T: Config> fmt::Debug for NetworkToken<T> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{:?}", self.0)
	}
}
