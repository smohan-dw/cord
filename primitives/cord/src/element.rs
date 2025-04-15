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

// # CORD Element (MultiData Format)

// Note: This module is part of cord-primitives and should be imported by all higher-level
// modules that need to interact with elements.

use crate::identifier::Ss58Identifier;
use alloc::vec::Vec;
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{traits::ConstU32, BoundedVec};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

/// The `Element` enum supports the following variants:
/// - None: Indicates that no data is provided.
/// - Raw: Contains data stored directly as a bounded vector; the capacity is specified by `MAX_CAP`.
/// - Identifier: An embedded Ss58Identifier.
/// - Digest: A fixed 32-byte digest (e.g., computed using BlakeTwo256).
/// - CID: A fixed 64-byte content identifier.
#[derive(Clone, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub enum Element<const MAX_CAP: u32> {
	/// No data provided.
	None,
	/// Raw data stored directly.
	Raw(BoundedVec<u8, ConstU32<MAX_CAP>>),
	/// An embedded Ss58Identifier.
	Identifier(Ss58Identifier),
	/// A 32-byte BlakeTwo256 digest.
	Digest([u8; 32]),
	/// A 64-byte content identifier.
	CID([u8; 64]),
}

// Custom Encode implementation with a one-byte discriminant.
impl<const MAX_CAP: u32> Encode for Element<MAX_CAP> {
	fn encode(&self) -> Vec<u8> {
		match self {
			Element::None => {
				let mut encoded = Vec::with_capacity(1);
				encoded.push(0);
				encoded
			},
			Element::Raw(raw) => {
				let raw_bytes = raw.encode();
				let mut encoded = Vec::with_capacity(1 + raw_bytes.len());
				encoded.push(1);
				encoded.extend(raw_bytes);
				encoded
			},
			Element::Identifier(id) => {
				let id_bytes = id.encode();
				let mut encoded = Vec::with_capacity(1 + id_bytes.len());
				encoded.push(2);
				encoded.extend(id_bytes);
				encoded
			},
			Element::Digest(hash) => {
				let mut encoded = Vec::with_capacity(1 + 32);
				encoded.push(3);
				encoded.extend(hash.encode());
				encoded
			},
			Element::CID(cid) => {
				let mut encoded = Vec::with_capacity(1 + 64);
				encoded.push(4);
				encoded.extend(cid.encode());
				encoded
			},
		}
	}
}

impl<const MAX_CAP: u32> Decode for Element<MAX_CAP> {
	fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
		let variant = input.read_byte()?;
		match variant {
			0 => Ok(Element::None),
			1 => {
				let raw = BoundedVec::<u8, ConstU32<MAX_CAP>>::decode(input)?;
				Ok(Element::Raw(raw))
			},
			2 => {
				let id = Ss58Identifier::decode(input)?;
				Ok(Element::Identifier(id))
			},
			3 => {
				let hash = <[u8; 32]>::decode(input)?;
				Ok(Element::Digest(hash))
			},
			4 => {
				let cid = <[u8; 64]>::decode(input)?;
				Ok(Element::CID(cid))
			},
			_ => Err("Unknown variant for Element".into()),
		}
	}
}

// Provide a unified AsRef<[u8]> implementation to obtain a view of the inner bytes.
impl<const MAX_CAP: u32> AsRef<[u8]> for Element<MAX_CAP> {
	fn as_ref(&self) -> &[u8] {
		match self {
			Element::None => &[],
			Element::Raw(raw) => raw.as_slice(),
			Element::Identifier(id) => id.as_bytes(),
			Element::Digest(digest) => digest,
			Element::CID(cid) => cid,
		}
	}
}

// Explicit accessor methods for each variant.
impl<const MAX_CAP: u32> Element<MAX_CAP> {
	/// Returns `true` if the Element is `None`.
	pub fn is_none(&self) -> bool {
		matches!(self, Element::None)
	}

	/// If the Element is `Raw`, returns a reference to its contents; otherwise, returns `None`.
	pub fn as_raw(&self) -> Option<&[u8]> {
		if let Element::Raw(raw) = self {
			Some(raw.as_slice())
		} else {
			None
		}
	}

	/// Returns the embedded Ss58Identifier if the element is Identifier.
	pub fn as_identifier(&self) -> Option<&Ss58Identifier> {
		if let Element::Identifier(id) = self {
			Some(id)
		} else {
			None
		}
	}

	/// If the Element is `Digest`, returns a reference to the 32-byte digest; otherwise, returns `None`.
	pub fn as_digest(&self) -> Option<&[u8; 32]> {
		if let Element::Digest(digest) = self {
			Some(digest)
		} else {
			None
		}
	}

	/// If the Element is `CID`, returns a reference to the 64-byte content identifier; otherwise, returns `None`.
	pub fn as_cid(&self) -> Option<&[u8; 64]> {
		if let Element::CID(cid) = self {
			Some(cid)
		} else {
			None
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use alloc::vec::Vec;
	use codec::{Decode, Encode};
	use core::convert::TryInto;
	use frame_support::{traits::ConstU32, BoundedVec};

	// Use a default Element type with MAX_CAP = 1024
	pub type DefaultElement = Element<1024>;

	#[test]
	fn test_element_none_encode_decode() {
		let element: DefaultElement = DefaultElement::None;
		let encoded = element.encode();
		assert_eq!(encoded, vec![0]);
		let decoded = DefaultElement::decode(&mut &encoded[..])
			.expect("Decoding should succeed for Element::None");
		assert_eq!(decoded, element);
		assert_eq!(element.as_ref(), &[] as &[u8]);
		assert!(element.is_none());
		assert!(element.as_raw().is_none());
		assert!(element.as_digest().is_none());
		assert!(element.as_cid().is_none());
		assert!(element.as_identifier().is_none());
	}

	#[test]
	fn test_element_raw_encode_decode() {
		let raw_data: Vec<u8> = vec![1, 2, 3, 4, 5];
		let bounded: BoundedVec<u8, ConstU32<1024>> =
			raw_data.clone().try_into().expect("Conversion should succeed");
		let element: DefaultElement = DefaultElement::Raw(bounded);
		let encoded = element.encode();
		let decoded = DefaultElement::decode(&mut &encoded[..])
			.expect("Decoding should succeed for Element::Raw");
		assert_eq!(decoded, element);
		assert_eq!(element.as_raw(), Some(&raw_data[..]));
		assert_eq!(element.as_ref(), &raw_data[..]);
	}

	#[test]
	fn test_element_identifier_encode_decode() {
		// For this test, we need a valid Ss58Identifier.
		// Here we assume Ss58Identifier::to_encoded exists and works as expected.
		let digest: Vec<u8> = vec![0xAB; 32];
		let nid: u16 = 100;
		let pid: u16 = 5;
		let ss58_id = Ss58Identifier::to_encoded(digest.clone(), nid, pid)
			.expect("Ss58Identifier should be created successfully");
		let element: DefaultElement = DefaultElement::Identifier(ss58_id.clone());
		let encoded = element.encode();
		let decoded = DefaultElement::decode(&mut &encoded[..])
			.expect("Decoding should succeed for Element::Identifier");
		assert_eq!(decoded, element);
		// Test accessor.
		assert_eq!(element.as_identifier(), Some(&ss58_id));
		// Also check unified AsRef returns the same bytes as from Ss58Identifier.
		assert_eq!(element.as_ref(), ss58_id.as_bytes());
	}

	#[test]
	fn test_element_digest_encode_decode() {
		let digest: [u8; 32] = [0xAA; 32];
		let element: DefaultElement = DefaultElement::Digest(digest);
		let encoded = element.encode();
		assert_eq!(encoded.len(), 33);
		let decoded = DefaultElement::decode(&mut &encoded[..])
			.expect("Decoding should succeed for Element::Digest");
		assert_eq!(decoded, element);
		assert_eq!(element.as_digest(), Some(&digest));
		assert_eq!(element.as_ref(), &digest[..]);
		assert!(element.as_raw().is_none());
		assert!(element.as_cid().is_none());
		assert!(element.as_identifier().is_none());
	}

	#[test]
	fn test_element_cid_encode_decode() {
		let cid: [u8; 64] = [0x55; 64];
		let element: DefaultElement = DefaultElement::CID(cid);
		let encoded = element.encode();
		assert_eq!(encoded.len(), 65);
		let decoded = DefaultElement::decode(&mut &encoded[..])
			.expect("Decoding should succeed for Element::CID");
		assert_eq!(decoded, element);
		assert_eq!(element.as_cid(), Some(&cid));
		assert_eq!(element.as_ref(), &cid[..]);
		assert!(element.as_raw().is_none());
		assert!(element.as_digest().is_none());
		assert!(element.as_identifier().is_none());
	}

	#[test]
	fn test_as_ref_for_all_variants() {
		let none_elem: DefaultElement = DefaultElement::None;
		assert_eq!(none_elem.as_ref(), &[] as &[u8]);

		let raw_data: Vec<u8> = vec![10, 20, 30];
		let bounded: BoundedVec<u8, ConstU32<1024>> =
			raw_data.clone().try_into().expect("Conversion should succeed");
		let raw_elem: DefaultElement = DefaultElement::Raw(bounded);
		assert_eq!(raw_elem.as_ref(), &raw_data[..]);

		let digest_elem: DefaultElement = DefaultElement::Digest([1; 32]);
		assert_eq!(digest_elem.as_ref(), &[1; 32][..]);

		let cid_elem: DefaultElement = DefaultElement::CID([2; 64]);
		assert_eq!(cid_elem.as_ref(), &[2; 64][..]);

		// Create an Ss58Identifier for testing the Identifier variant.
		let valid_digest = vec![0xAB; 32];
		let ss58_id = Ss58Identifier::to_encoded(valid_digest, 100, 5)
			.expect("Ss58Identifier creation should succeed");
		let id_elem: DefaultElement = DefaultElement::Identifier(ss58_id.clone());
		assert_eq!(id_elem.as_identifier(), Some(&ss58_id));
		assert_eq!(id_elem.as_ref(), ss58_id.as_bytes());
	}

	#[test]
	fn test_decode_unknown_variant() {
		// Prepare an invalid encoding with an unknown discriminant.
		let invalid_encoded: Vec<u8> = vec![255];
		let result = DefaultElement::decode(&mut &invalid_encoded[..]);
		assert!(result.is_err(), "Decoding should fail for an unknown variant tag");
	}

	#[test]
	fn test_decode_incomplete_raw() {
		let raw_data: Vec<u8> = vec![1, 2, 3, 4, 5];
		let bounded: BoundedVec<u8, ConstU32<1024>> =
			raw_data.clone().try_into().expect("Conversion should succeed");
		let element: DefaultElement = DefaultElement::Raw(bounded);
		let mut encoded = element.encode();
		encoded.pop();
		let result = DefaultElement::decode(&mut &encoded[..]);
		assert!(result.is_err(), "Decoding should fail for incomplete Raw encoding");
	}

	#[test]
	fn test_decode_incomplete_digest() {
		let digest: [u8; 32] = [0xAA; 32];
		let element: DefaultElement = DefaultElement::Digest(digest);
		let mut encoded = element.encode();
		for _ in 0..5 {
			encoded.pop();
		}
		let result = DefaultElement::decode(&mut &encoded[..]);
		assert!(result.is_err(), "Decoding should fail for incomplete Digest encoding");
	}

	#[test]
	fn test_decode_incomplete_cid() {
		let cid: [u8; 64] = [0x55; 64];
		let element: DefaultElement = DefaultElement::CID(cid);
		let mut encoded = element.encode();
		for _ in 0..10 {
			encoded.pop();
		}
		let result = DefaultElement::decode(&mut &encoded[..]);
		assert!(result.is_err(), "Decoding should fail for incomplete CID encoding");
	}

	#[test]
	fn test_boundedvec_too_long() {
		let long_vec: Vec<u8> = vec![0u8; 1024 + 1];
		let bounded: Result<BoundedVec<u8, ConstU32<1024>>, _> = long_vec.try_into();
		assert!(bounded.is_err(), "Creating a BoundedVec with too many elements should fail");
	}

	#[test]
	fn test_integration_with_container() {
		#[derive(Clone, PartialEq, Eq, Debug, Encode, Decode)]
		struct Container {
			pub id: u32,
			pub element: DefaultElement,
		}

		let element = DefaultElement::Raw(vec![42, 43].try_into().unwrap());
		let container = Container { id: 7, element };
		let encoded = container.encode();
		let decoded =
			Container::decode(&mut &encoded[..]).expect("Decoding container should succeed");
		assert_eq!(container, decoded);
	}
}
