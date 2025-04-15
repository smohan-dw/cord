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

// # CORD Identifier (Ss58Identifier)

// Note: This module is part of cord-primitives and should be imported by all higher-level
// modules that need to interact with identifiers.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;
use alloc::{string::String, vec::Vec};
use blake2::{Blake2b512, Digest};
use bs58;
use codec::{Decode, Encode, MaxEncodedLen};
use core::convert::TryFrom;
use frame_support::{ensure, traits::ConstU32, BoundedVec};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

/// Constant prefix used in checksum calculation.
const PREFIX: &[u8] = b"CURIV02";

/// Identifier errors.
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
	/// The provided digest length is invalid. Expected 32 bytes.
	InvalidDigestLength,
	/// The value is out of the expected range for compact encoding.
	CompactValueOutOfRange,
}

/// The Ss58Identifier type is a persistent identifier built from a bounded vector of bytes.
/// The capacity (here, 52) must be chosen such that the final Base58-encoded value fits the system constraints.
#[derive(Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub struct Ss58Identifier(pub(crate) BoundedVec<u8, ConstU32<52>>);

impl Ss58Identifier {
	/// Compute the ss58 hash: Blake2b512 over PREFIX concatenated with the provided data.
	pub fn ss58hash(data: &[u8]) -> Vec<u8> {
		let mut context = Blake2b512::new();
		context.update(PREFIX);
		context.update(data);
		context.finalize().to_vec()
	}

	/// Construct an Ss58Identifier from a 32-byte digest, a network id, and a pallet id.
	/// The resulting identifier is Base58-encoded.
	pub fn to_encoded<I>(data: I, nid: u16, pid: u16) -> Result<Self, IdentifierError>
	where
		I: AsRef<[u8]> + Into<Vec<u8>>,
	{
		// Validate the digest length.
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

	/// Decode the Ss58Identifier back into its structured components.
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

		let (pid, _) = Self::compact_decode(&data[offset..])?;

		Ok(DecodedIdentifier { nid, pid, gen: format!("0x{}", hex::encode(digest)) })
	}

	/// Decodes a compact-encoded u16 value from the provided data slice.
	/// Returns the decoded value and the number of bytes consumed.
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

	/// Compactly encodes a u16 value into 1 or 2 bytes.
	fn compact_encode(value: u16) -> Result<Vec<u8>, IdentifierError> {
		match value {
			0..=63 => Ok(vec![value as u8]),
			64..=16_383 => {
				let first = ((value & 0b0000_0000_1111_1100) as u8) >> 2;
				let second = ((value >> 8) as u8) | (((value & 0b0000_0000_0000_0011) as u8) << 6);
				Ok(vec![first | 0b01000000, second])
			},
			_ => Err(IdentifierError::CompactValueOutOfRange),
		}
	}

	/// Returns a reference to the underlying bytes of the identifier.
	pub fn as_bytes(&self) -> &[u8] {
		self.0.as_slice()
	}
}

impl TryFrom<Vec<u8>> for Ss58Identifier {
	type Error = IdentifierError;
	fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
		let bounded = BoundedVec::<u8, ConstU32<52>>::try_from(value)
			.map_err(|_| IdentifierError::InvalidIdentifierLength)?;
		let identifier = Ss58Identifier(bounded);
		// Validate by attempting to decode.
		identifier.to_decoded()?;
		Ok(identifier)
	}
}

impl TryFrom<String> for Ss58Identifier {
	type Error = IdentifierError;
	fn try_from(s: String) -> Result<Self, Self::Error> {
		Ss58Identifier::try_from(s.into_bytes())
	}
}

impl AsRef<[u8]> for Ss58Identifier {
	fn as_ref(&self) -> &[u8] {
		self.0.as_slice()
	}
}

/// Represents the structured components of an identifier after decoding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedIdentifier {
	/// Network identifier.
	pub nid: u16,
	/// Pallet (or module) identifier.
	pub pid: u16,
	/// The digest (genesis hash) in hexadecimal format.
	pub gen: String,
}

#[cfg(test)]
mod tests {
	use super::*;
	use alloc::vec::Vec;
	use core::convert::TryFrom;

	// Helper function to generate a valid 32-byte digest.
	fn valid_digest() -> [u8; 32] {
		[0xAB; 32]
	}

	/// Test that an identifier built with valid input can be encoded and decoded correctly.
	#[test]
	fn encode_decode_roundtrip() {
		let digest = valid_digest();
		let nid: u16 = 100;
		let pid: u16 = 5;
		let identifier = Ss58Identifier::to_encoded(digest.clone(), nid, pid)
			.expect("Identifier encoding should succeed");
		let decoded = identifier.to_decoded().expect("Identifier decoding should succeed");

		// Verify that the decoded network and pallet identifiers match the input.
		assert_eq!(decoded.nid, nid, "The network id should match");
		assert_eq!(decoded.pid, pid, "The pallet id should match");
		// Verify that the genesis hash (digest) matches.
		assert_eq!(decoded.gen, format!("0x{}", hex::encode(digest)), "The digest should match");
	}

	/// Test that creating an identifier with an invalid digest length fails.
	#[test]
	fn fails_invalid_digest_length() {
		let short_digest = vec![0xAB; 31]; // 31 bytes, which is invalid.
		let nid: u16 = 100;
		let pid: u16 = 5;
		let result = Ss58Identifier::to_encoded(short_digest, nid, pid);
		assert!(result.is_err(), "Encoding should fail when digest is not 32 bytes long");
		// if let Err(IdentifierError::InvalidDigestLength) = result {
		// 	// Expected error.
		// } else {
		// 	panic!("Expected InvalidDigestLength error");
		// }
	}

	/// Test that an identifier with a tampered checksum fails to decode.
	#[test]
	fn fails_tampered_checksum() {
		let digest = valid_digest();
		let nid: u16 = 100;
		let pid: u16 = 5;
		let identifier = Ss58Identifier::to_encoded(digest, nid, pid)
			.expect("Identifier encoding should succeed");

		// Base58-decode the identifier to obtain the raw bytes.
		let mut decoded_bytes =
			bs58::decode(&identifier.0).into_vec().expect("Base58 decoding should succeed");
		// Tamper with the checksum (flip a bit in the last byte).
		let last_index = decoded_bytes.len() - 1;
		decoded_bytes[last_index] ^= 1;
		// Re-encode the tampered bytes.
		let tampered = bs58::encode(&decoded_bytes).into_string();
		// Convert the tampered Base58 string into an identifier.
		let result: Result<Ss58Identifier, _> = tampered.try_into();
		assert!(result.is_err(), "Decoding should fail for an identifier with a tampered checksum");
	}

	/// Test conversion from Vec<u8> using the TryFrom implementation.
	#[test]
	fn try_from_vec_success() {
		let digest = valid_digest();
		let nid: u16 = 100;
		let pid: u16 = 5;
		let identifier = Ss58Identifier::to_encoded(digest, nid, pid)
			.expect("Identifier encoding should succeed");
		// Get the raw Base58-encoded vector.
		let raw: Vec<u8> = identifier.0.clone().into();
		let identifier2 =
			Ss58Identifier::try_from(raw).expect("Conversion from Vec<u8> should succeed");
		let decoded = identifier2.to_decoded().expect("Decoding should succeed after conversion");
		assert_eq!(decoded.nid, nid, "Network id should match");
		assert_eq!(decoded.pid, pid, "Pallet id should match");
	}

	/// Test conversion from a Base58-encoded String using the TryFrom implementation.
	#[test]
	fn try_from_string_success() {
		let digest = valid_digest();
		let nid: u16 = 100;
		let pid: u16 = 5;
		let identifier = Ss58Identifier::to_encoded(digest, nid, pid)
			.expect("Identifier encoding should succeed");
		// Convert the identifier's inner BoundedVec to a Base58 string.
		let as_string = String::from_utf8(identifier.0.clone().into())
			.expect("Base58 string should be valid UTF-8");
		let identifier2 =
			Ss58Identifier::try_from(as_string).expect("Conversion from String should succeed");
		let decoded = identifier2.to_decoded().expect("Decoding should succeed after conversion");
		assert_eq!(decoded.nid, nid, "Network id should match");
		assert_eq!(decoded.pid, pid, "Pallet id should match");
	}
}
