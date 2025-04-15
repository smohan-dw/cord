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

//! # CORD Identifiers
#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

extern crate alloc;
use alloc::{format, string::String, vec, vec::Vec};
use codec::{Decode, Encode, MaxEncodedLen};
use cord_primitives::Id as NetworkId;
use frame_support::{ensure, pallet_prelude::*, traits::ConstU32, BoundedVec};
use scale_info::TypeInfo;
use sp_runtime::traits::{BlockNumberProvider, UniqueSaturatedInto};

#[cfg(test)]
pub mod mock;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

#[cfg(test)]
mod tests;

/// The prefix used for hash computation.
const PREFIX: &[u8] = b"CURIV02";
/// The starting index for pallets.
const INDEX: u16 = 64;

pub use crate::pallet::*;
pub(crate) type NodeId = BoundedVec<u8, ConstU32<60>>;
pub type HashOf<T> = <T as frame_system::Config>::Hash;

/// EventStamp marks the block and extrinsic where an event occurred.
#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub struct EventStamp {
	pub height: u32,
	pub index: u32,
}

/// EntryTypeOf is a bounded vector (max 128 bytes) that holds part of an event message,
pub type EventTypeOf = BoundedVec<u8, ConstU32<128>>;

/// ActivityRecord stores an update entry and the corresponding event stamp.
#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub struct StateTransitionEvent<Hash> {
	pub event: EventTypeOf,
	pub digest: Hash,
	pub event_stamp: EventStamp,
}

/// Errors for identifier operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdentifierError {
	/// The identifier format is invalid.
	InvalidFormat,
	/// The prefix is invalid or unrecognized.
	InvalidPrefix,
	/// The identifier is not valid.
	InvalidIdentifier,
	/// The checksum validation failed.
	InvalidChecksum,
	/// The identifier length is not valid.
	InvalidIdentifierLength,
	/// The pallet name exceeds the maximum allowed length.
	PalletNameTooLong,
	/// The specified pallet name was not found.
	PalletNotFound,
	/// The specified pallet index is invalid.
	InvalidPalletIndex,
	/// The pallet name format is invalid.
	InvalidPalletNameFormat,
	/// The provided network id does not match the expected value.
	InvalidNetworkId,
	// Max exvents history exceeded
	MaxEventsHistoryExceeded,
	/// The provided digest length is invalid. Expected 32 bytes.
	InvalidDigestLength,
	/// The value is out of the expected range for compact encoding.
	CompactValueOutOfRange,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Provider for the block number.
		type BlockNumberProvider: BlockNumberProvider;
	}

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	pub type PalletIndex<T: Config> =
		StorageMap<_, Blake2_128Concat, BoundedVec<u8, ConstU32<64>>, u16>;

	#[pallet::storage]
	pub type IndexToPallet<T: Config> =
		StorageMap<_, Blake2_128Concat, u16, BoundedVec<u8, ConstU32<64>>>;

	#[pallet::storage]
	pub type NextPalletIndex<T: Config> = StorageValue<_, u16, ValueQuery>;

	#[pallet::storage]
	pub type GenesisNetworkId<T: Config> = StorageValue<_, NetworkId, ValueQuery>;

	#[pallet::storage]
	pub type StateHistory<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		Ss58Identifier,
		Twox64Concat,
		u32,
		StateTransitionEvent<HashOf<T>>,
		OptionQuery,
	>;

	#[pallet::storage]
	pub type StateVersion<T: Config> =
		StorageMap<_, Blake2_128Concat, Ss58Identifier, u32, ValueQuery>;
}

impl<T: Config> Pallet<T> {
	pub fn get_or_add_pallet_index(pallet_name: &str) -> Result<u16, IdentifierError> {
		let bounded_name: BoundedVec<u8, ConstU32<64>> = pallet_name
			.as_bytes()
			.to_vec()
			.try_into()
			.map_err(|_| IdentifierError::PalletNameTooLong)?;

		if let Some(index) = PalletIndex::<T>::get(&bounded_name) {
			return Ok(index);
		}

		let current_index = INDEX + NextPalletIndex::<T>::get() as u16;
		ensure!(current_index <= u16::MAX, IdentifierError::InvalidPalletIndex);

		PalletIndex::<T>::insert(&bounded_name, current_index);
		IndexToPallet::<T>::insert(current_index, bounded_name);
		NextPalletIndex::<T>::put(current_index.saturating_add(1));

		Ok(current_index)
	}

	pub fn resolve_pallet_name(index: u16) -> Result<String, IdentifierError> {
		IndexToPallet::<T>::get(index).ok_or(IdentifierError::PalletNotFound).and_then(
			|name_bytes| {
				String::from_utf8(name_bytes.into())
					.map_err(|_| IdentifierError::InvalidPalletNameFormat)
			},
		)
	}

	pub fn set_network_id(network_id: NetworkId) {
		GenesisNetworkId::<T>::put(network_id);
	}

	pub fn get_network_id() -> NetworkId {
		GenesisNetworkId::<T>::get()
	}

	/// Record an activity event for the given identifier by appending a new record.
	pub fn update_identifier_state(
		identifier: &Ss58Identifier,
		digest: HashOf<T>,
		event: EventTypeOf,
		stamp: EventStamp,
	) -> Result<(), IdentifierError> {
		let index = StateVersion::<T>::get(identifier);
		let record = StateTransitionEvent { event, digest, event_stamp: stamp };
		StateHistory::<T>::insert(identifier, index, record);
		StateVersion::<T>::insert(identifier, index.saturating_add(1));
		Ok(())
	}
}

#[derive(
	Clone, Eq, PartialEq, Ord, PartialOrd, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo,
)]
pub struct Ss58Identifier(pub(crate) BoundedVec<u8, ConstU32<52>>);

pub trait Identifier {
	type Hash: Encode + Decode + Clone + PartialEq + Eq;
	fn build(digest: &[u8], pallet: &str) -> Result<Ss58Identifier, IdentifierError>;
	fn resolve_identifier(
		identifier: &Ss58Identifier,
	) -> Result<DecodedIdentifier, IdentifierError>;
	fn resolve_pallet(index: u16) -> Result<String, IdentifierError>;
	/// Record a state trsition event for the given identifier.
	fn state_event(
		identifier: &Ss58Identifier,
		digest: Self::Hash,
		event: EventTypeOf,
		stamp: EventStamp,
	) -> Result<(), IdentifierError>;
}

impl Ss58Identifier {
	fn ss58hash(data: &[u8]) -> Vec<u8> {
		use blake2::{Blake2b512, Digest};

		let mut context = Blake2b512::new();
		context.update(PREFIX);
		context.update(data);
		context.finalize().to_vec()
	}

	pub fn to_encoded<I>(data: I, nid: u16, pid: u16) -> Result<Self, IdentifierError>
	where
		I: AsRef<[u8]> + Into<Vec<u8>>,
	{
		// Ensure the digest is exactly 32 bytes long.
		if data.as_ref().len() != 32 {
			return Err(IdentifierError::InvalidDigestLength);
		}

		let mut buffer = Self::compact_encode(nid & 0b0011_1111_1111_1111)?;
		buffer.extend(data.as_ref());
		let p = Self::compact_encode(pid & 0b0011_1111_1111_1111)?;
		buffer.extend(p);
		let checksum = &Self::ss58hash(&buffer)[..2];
		buffer.extend(checksum);

		Ok(Self(
			Vec::<u8>::from(bs58::encode(&buffer).into_string())
				.try_into()
				.map_err(|_| IdentifierError::InvalidIdentifier)?,
		))
	}

	pub fn to_decoded(&self) -> Result<DecodedIdentifier, IdentifierError> {
		let decoded =
			bs58::decode(&self.0).into_vec().map_err(|_| IdentifierError::InvalidFormat)?;

		ensure!(
			decoded.len() >= 36 && decoded.len() <= 38,
			IdentifierError::InvalidIdentifierLength
		);

		let checksum_start = decoded.len() - 2;
		let provided_checksum = &decoded[checksum_start..];
		let expected_checksum = &Self::ss58hash(&decoded[..checksum_start])[..2];
		ensure!(provided_checksum == expected_checksum, IdentifierError::InvalidChecksum);

		let data = &decoded[..checksum_start];
		let (nid, mut offset) = Self::compact_decode(data)?;

		ensure!(data.len() >= offset + 32, IdentifierError::InvalidIdentifierLength);
		let digest = data[offset..offset + 32].to_vec();
		offset += 32;

		let (pid, _compact_len) = Self::compact_decode(&data[offset..])?;

		Ok(DecodedIdentifier { nid, pid, gen: format!("0x{}", hex::encode(digest)) })
	}

	fn compact_decode(data: &[u8]) -> Result<(u16, usize), IdentifierError> {
		if data.is_empty() {
			return Err(IdentifierError::InvalidPrefix);
		}
		match data[0] {
			0..=63 => Ok((data[0] as u16, 1)),
			64..=127 => {
				ensure!(data.len() >= 2, IdentifierError::InvalidPrefix);
				let mid = data[0] & 0b0011_1111;
				let low = data[1] >> 6;
				let high = data[1] & 0b0011_1111;
				let value = ((high as u16) << 8) | ((mid as u16) << 2) | (low as u16);
				Ok((value, 2))
			},
			_ => Err(IdentifierError::InvalidPrefix),
		}
	}

	fn compact_encode(value: u16) -> Result<Vec<u8>, IdentifierError> {
		match value {
			0..=63 => Ok(vec![value as u8]),
			64..=16_383 => {
				let first = ((value & 0b0000_0000_1111_1100) as u8) >> 2;
				let second = ((value >> 8) as u8) | ((value & 0b0000_0000_0000_0011) as u8) << 6;
				Ok(vec![first | 0b01000000, second])
			},
			_ => Err(IdentifierError::CompactValueOutOfRange),
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedIdentifier {
	pub nid: u16,
	pub pid: u16,
	pub gen: String,
}

impl<T: Config> Identifier for Pallet<T> {
	type Hash = HashOf<T>;
	fn build(digest: &[u8], pallet: &str) -> Result<Ss58Identifier, IdentifierError> {
		let pid = Self::get_or_add_pallet_index(pallet)?;
		let network_id = Self::get_network_id();
		let nid = network_id.inner() as u16;

		Ss58Identifier::to_encoded(digest, nid, pid)
	}

	fn resolve_identifier(
		identifier: &Ss58Identifier,
	) -> Result<DecodedIdentifier, IdentifierError> {
		identifier.to_decoded()
	}

	fn resolve_pallet(index: u16) -> Result<String, IdentifierError> {
		Self::resolve_pallet_name(index)
	}

	fn state_event(
		identifier: &Ss58Identifier,
		digest: HashOf<T>,
		event: EventTypeOf,
		stamp: EventStamp,
	) -> Result<(), IdentifierError> {
		Self::update_identifier_state(identifier, digest, event, stamp)
	}
}

impl TryFrom<Vec<u8>> for Ss58Identifier {
	type Error = IdentifierError;

	fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
		let bounded = BoundedVec::<u8, ConstU32<52>>::try_from(value)
			.map_err(|_| IdentifierError::InvalidIdentifierLength)?;
		let identifier = Ss58Identifier(bounded);
		identifier.to_decoded()?;
		Ok(identifier)
	}
}

impl TryFrom<String> for Ss58Identifier {
	type Error = IdentifierError;
	fn try_from(s: String) -> Result<Self, Self::Error> {
		let decoded = bs58::decode(&s).into_vec().map_err(|_| IdentifierError::InvalidFormat)?;
		// Delegate to the Vec<u8> conversion, which now performs the full validation.
		Ss58Identifier::try_from(decoded)
	}
}

impl AsRef<[u8]> for Ss58Identifier {
	fn as_ref(&self) -> &[u8] {
		self.0.as_slice()
	}
}

impl EventStamp {
	/// Returns the current event stamp from the caller’s runtime context.
	pub fn current<T: frame_system::Config>() -> Self {
		Self {
			height: frame_system::Pallet::<T>::current_block_number().unique_saturated_into(),
			index: frame_system::Pallet::<T>::extrinsic_index().unwrap_or_default(),
		}
	}
}

/// A trait that can be used to ensure that a registry identifier exists and is active..
pub trait RegistryIdentifierStatus {
	/// Checks that the registry identified by `registry_id` exists and is active.
	fn ensure_active_registry(registry_id: &Ss58Identifier) -> DispatchResult;
}

pub trait StorageNodeStatus {
	type AccountId;

	/// Get the details of a storage node by its `node_id`.
	fn ensure_active_storage_node(
		node_id: &NodeId,
		owner: &Self::AccountId,
	) -> Result<Ss58Identifier, DispatchError>;

	fn get_storage_node_details(
		node_id: &NodeId,
	) -> Option<(Ss58Identifier, Self::AccountId, bool)>;
}

#[cfg(test)]
mod compact_tests {
	use super::*;

	/// Test round-trip for values in the valid range for compact encoding (0–16,383).
	#[test]
	fn compact_roundtrip_valid_range() {
		// Test every value in one- and two-byte modes.
		for value in 0u16..=16_383 {
			let encoded = Ss58Identifier::compact_encode(value)
				.expect(&format!("Encoding should succeed for value {}", value));
			let (decoded, consumed) = Ss58Identifier::compact_decode(&encoded)
				.expect(&format!("Decoding should succeed for value {}", value));
			assert_eq!(decoded, value, "Roundtrip failed for value {}", value);
			assert_eq!(consumed, encoded.len(), "Not all bytes consumed for value {}", value);
		}
	}

	/// Test that encoding a value out of range returns an error.
	#[test]
	fn compact_encode_out_of_range() {
		// The next value (16,384) should be out of the supported range.
		let value = 16_384u16;
		let result = Ss58Identifier::compact_encode(value);
		assert!(
			result.is_err(),
			"Value {} should be out of range but encoded as {:?}",
			value,
			result.ok()
		);
	}

	/// Test boundary values for compact encoding/decoding.
	#[test]
	fn compact_boundary_values() {
		// Test value at the upper end of one-byte mode: 63.
		let value = 63u16;
		let encoded = Ss58Identifier::compact_encode(value)
			.expect(&format!("Encoding should succeed for value {}", value));
		let (decoded, consumed) = Ss58Identifier::compact_decode(&encoded)
			.expect(&format!("Decoding should succeed for value {}", value));
		assert_eq!(decoded, value, "Roundtrip failed for one-byte mode value {}", value);
		assert_eq!(
			consumed,
			encoded.len(),
			"Not all bytes consumed for one-byte mode value {}",
			value
		);

		// Test value at the lower end of two-byte mode: 64.
		let value = 64u16;
		let encoded = Ss58Identifier::compact_encode(value)
			.expect(&format!("Encoding should succeed for value {}", value));
		// Verify that two-byte encoding is used.
		assert_eq!(
			encoded.len(),
			2,
			"Expected 2-byte encoding for value {} but got {} bytes",
			value,
			encoded.len()
		);
		// For value 64, the expected encoding is [80, 0]:
		//   - First byte: ((64 & 0b11111100) >> 2) = 16, then 16 | 0b01000000 = 80.
		//   - Second byte: (64 >> 8) | ((64 & 3) << 6) = 0 | (0 << 6) = 0.
		assert_eq!(encoded[0], 80, "First byte mismatch for value {}", value);
		assert_eq!(encoded[1], 0, "Second byte mismatch for value {}", value);
		let (decoded, consumed) = Ss58Identifier::compact_decode(&encoded)
			.expect(&format!("Decoding should succeed for value {}", value));
		assert_eq!(decoded, value);
		assert_eq!(consumed, encoded.len());

		// Test the upper boundary of two-byte mode: 16,383.
		let value = 16_383u16;
		let encoded = Ss58Identifier::compact_encode(value)
			.expect(&format!("Encoding should succeed for value {}", value));
		assert_eq!(
			encoded.len(),
			2,
			"Expected 2-byte encoding for value {} but got {} bytes",
			value,
			encoded.len()
		);
		let (decoded, consumed) = Ss58Identifier::compact_decode(&encoded)
			.expect(&format!("Decoding should succeed for upper boundary value {}", value));
		assert_eq!(
			decoded, value,
			"Roundtrip failed for upper boundary two-byte mode value {}",
			value
		);
		assert_eq!(
			consumed,
			encoded.len(),
			"Not all bytes consumed for upper boundary value {}",
			value
		);
	}

	/// Test that decoding an incomplete byte slice in two-byte mode returns an error.
	#[test]
	fn compact_decode_incomplete_slice() {
		// Use a value that forces a two-byte encoding.
		let value = 64u16;
		let encoded =
			Ss58Identifier::compact_encode(value).expect("Encoding should succeed for value 64");
		assert_eq!(encoded.len(), 2, "Encoded length should be 2 bytes for value 64");

		// Pass only the first byte – data is incomplete.
		let incomplete_slice = &encoded[0..1];
		let result = Ss58Identifier::compact_decode(incomplete_slice);
		assert!(result.is_err(), "Decoding should fail for an incomplete byte slice");
	}

	/// Test that decoding an empty slice returns an error.
	#[test]
	fn compact_decode_empty_slice() {
		let empty: &[u8] = &[];
		let result = Ss58Identifier::compact_decode(empty);
		assert!(result.is_err(), "Decoding should fail when provided an empty slice");
	}

	/// Test that decoding with an invalid prefix (first byte ≥ 128) returns an error.
	#[test]
	fn compact_decode_invalid_prefix() {
		// Construct a slice with an invalid prefix (>= 128).
		let invalid_data: [u8; 2] = [130, 0];
		let result = Ss58Identifier::compact_decode(&invalid_data);
		assert!(result.is_err(), "Decoding should fail for a prefix value of {}", invalid_data[0]);
	}

	/// Test that a specific value is encoded exactly as expected.
	#[test]
	fn compact_specific_value_encoding() {
		// For value 66:
		// Expected:
		//   - First byte: ((66 & 0b11111100) >> 2) = ((66 & 252) >> 2) = 16, then OR with 0b01000000 gives 80.
		//   - Second byte: (66 >> 8) | ((66 & 3) << 6) = 0 | (2 << 6) = 128.
		let value: u16 = 66;
		let encoded = Ss58Identifier::compact_encode(value)
			.expect(&format!("Encoding should succeed for value {}", value));
		assert_eq!(encoded.len(), 2, "Expected 2-byte encoding for value {}", value);
		let expected: [u8; 2] = [80, 128];
		assert_eq!(encoded, expected, "Encoded bytes did not match expected for value {}", value);
		let (decoded, consumed) =
			Ss58Identifier::compact_decode(&encoded).expect("Decoding should succeed");
		assert_eq!(decoded, value, "Decoded value mismatch for value {}", value);
		assert_eq!(consumed, encoded.len(), "Not all bytes consumed for specific value {}", value);
	}

	/// Create a valid 32-byte digest (for example, by filling with a fixed pattern)
	fn valid_digest() -> [u8; 32] {
		[0xAB; 32]
	}

	/// Create a valid identifier using a 32-byte digest along with some network (nid) and pallet (pid) type values.
	#[test]
	fn identifier_roundtrip_valid() {
		// Given a valid 32-byte digest, and valid network and pallet ID
		let digest = valid_digest();
		let nid: u16 = 100;
		let pid: u16 = 5;

		let identifier = Ss58Identifier::to_encoded(&digest, nid, pid)
			.expect("Identifier encoding should succeed for valid digest");
		// Check that the decoded identifier returns the correct fields.
		let decoded = identifier.to_decoded().expect("Identifier decoding should succeed");
		assert_eq!(decoded.gen, format!("0x{}", hex::encode(&digest)));
		assert_eq!(decoded.nid, nid);
		assert_eq!(decoded.pid, pid);

		// Also check that the overall length is within our expected bounds.
		// Minimum length is 36 bytes (1-byte compact IDs) and maximum 38 bytes (if pallet and network require 2-byte compact encoding).
		let decoded_bytes =
			bs58::decode(&identifier.0).into_vec().expect("Base58 decoding must succeed");
		assert!(decoded_bytes.len() >= 36, "Identifier too short: {} bytes", decoded_bytes.len());
		assert!(decoded_bytes.len() <= 38, "Identifier too long: {} bytes", decoded_bytes.len());
	}

	/// Negative test: using an invalid digest length (e.g. 31 bytes) should fail.
	#[test]
	fn identifier_invalid_digest_length() {
		let short_digest = vec![0xAB; 31]; // 31 bytes instead of 32
		let nid: u16 = 10;
		let pid: u16 = 2;

		let result = Ss58Identifier::to_encoded(&*short_digest, nid, pid);
		assert!(result.is_err(), "Encoding should fail with invalid digest length");
	}

	#[test]
	fn identifier_checksum_failure() {
		let digest = valid_digest(); // generates a valid 32-byte digest, e.g., [0xAB; 32]
		let nid: u16 = 100;
		let pid: u16 = 5;
		let identifier = Ss58Identifier::to_encoded(&digest, nid, pid)
			.expect("Identifier encoding should succeed");

		// Convert the identifier to its raw bytes by base58-decoding.
		let mut decoded_bytes =
			bs58::decode(&identifier.0).into_vec().expect("Base58 decoding should succeed");

		// Tamper with the checksum: flip the last bit.
		let last_index = decoded_bytes.len() - 1;
		decoded_bytes[last_index] ^= 1;

		// Re-encode the tampered bytes to base58.
		let tampered = bs58::encode(&decoded_bytes).into_string();

		// Now, the conversion from String to Ss58Identifier should fail because
		// the full validation in TryFrom (via to_decoded) will detect the checksum mismatch.
		let result: Result<Ss58Identifier, _> = tampered.try_into();
		assert!(
			result.is_err(),
			"Conversion should fail for an identifier with a tampered checksum"
		);
	}

	/// Negative test: decoding an identifier with an invalid overall length should fail.
	#[test]
	fn identifier_invalid_length() {
		// Create an identifier with an excessively short encoded value.
		let invalid_data: Vec<u8> = vec![1, 2, 3]; // clearly too short to be valid
		let encoded = bs58::encode(&invalid_data).into_string();
		let result: Result<Ss58Identifier, _> = encoded.try_into();
		// Depending on your Ss58Identifier constructor, either this conversion or later decoding should fail.
		assert!(result.is_err(), "Conversion should fail for an identifier with invalid length");
	}
}
