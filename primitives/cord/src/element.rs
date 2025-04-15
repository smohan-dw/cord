use alloc::vec::Vec;
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{traits::ConstU32, BoundedVec};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

/// The `Element` enum supports the following variants:
/// - None: Indicates that no data is provided.
/// - Raw: Contains data stored directly as a bounded vector; the capacity is specified by `MAX_CAP`.
/// - Digest: A fixed 32-byte digest (e.g., computed using BlakeTwo256).
/// - CID: A fixed 64-byte content identifier.
#[derive(Clone, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub enum Element<const MAX_CAP: u32> {
	/// No data provided.
	None,
	/// Raw data stored directly.
	Raw(BoundedVec<u8, ConstU32<MAX_CAP>>),
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
			Element::Digest(hash) => {
				let mut encoded = Vec::with_capacity(1 + 32);
				encoded.push(2);
				encoded.extend(hash.encode());
				encoded
			},
			Element::CID(cid) => {
				let mut encoded = Vec::with_capacity(1 + 64);
				encoded.push(3);
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
				let hash = <[u8; 32]>::decode(input)?;
				Ok(Element::Digest(hash))
			},
			3 => {
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

	// Use the default element for most tests.
	pub type DefaultElement = Element<1024>;

	#[test]
	fn test_element_none_encode_decode() {
		let element: DefaultElement = DefaultElement::None;
		let encoded = element.encode();
		// For Element::None, expect a single byte: [0].
		assert_eq!(encoded, vec![0]);

		let decoded = DefaultElement::decode(&mut &encoded[..])
			.expect("Decoding should succeed for Element::None");
		assert_eq!(decoded, element);

		// Test unified AsRef:
		assert_eq!(element.as_ref(), &[] as &[u8]);
		// Test explicit accessors:
		assert!(element.is_none());
		assert!(element.as_raw().is_none());
		assert!(element.as_digest().is_none());
		assert!(element.as_cid().is_none());
	}

	#[test]
	fn test_element_raw_encode_decode() {
		let raw_data: Vec<u8> = vec![1, 2, 3, 4, 5];
		// Convert the raw vector to a BoundedVec with a capacity of 1024.
		let bounded: BoundedVec<u8, ConstU32<1024>> =
			raw_data.clone().try_into().expect("Conversion to BoundedVec should succeed");
		let element: DefaultElement = DefaultElement::Raw(bounded);

		let encoded = element.encode();
		let decoded = DefaultElement::decode(&mut &encoded[..])
			.expect("Decoding should succeed for Element::Raw");
		assert_eq!(decoded, element);

		// Check explicit accessor:
		assert_eq!(element.as_raw(), Some(&raw_data[..]));
		// Unified AsRef should return the contained raw bytes.
		assert_eq!(element.as_ref(), &raw_data[..]);
	}

	#[test]
	fn test_element_digest_encode_decode() {
		let digest: [u8; 32] = [0xAA; 32];
		let element: DefaultElement = DefaultElement::Digest(digest);

		let encoded = element.encode();
		// For Digest, encoding length should be 1 (tag) + 32 bytes.
		assert_eq!(encoded.len(), 33);

		let decoded = DefaultElement::decode(&mut &encoded[..])
			.expect("Decoding should succeed for Element::Digest");
		assert_eq!(decoded, element);

		// Check explicit accessor:
		assert_eq!(element.as_digest(), Some(&digest));
		// Unified AsRef should return the digest as a byte slice.
		assert_eq!(element.as_ref(), &digest[..]);
		// Other accessors should return None.
		assert!(element.as_raw().is_none());
		assert!(element.as_cid().is_none());
	}

	#[test]
	fn test_element_cid_encode_decode() {
		let cid: [u8; 64] = [0x55; 64];
		let element: DefaultElement = DefaultElement::CID(cid);

		let encoded = element.encode();
		// For CID, encoding length should be 1 + 64.
		assert_eq!(encoded.len(), 65);

		let decoded = DefaultElement::decode(&mut &encoded[..])
			.expect("Decoding should succeed for Element::CID");
		assert_eq!(decoded, element);

		// Check explicit accessor:
		assert_eq!(element.as_cid(), Some(&cid));
		// Unified AsRef should return the CID as a byte slice.
		assert_eq!(element.as_ref(), &cid[..]);
		// Other accessors:
		assert!(element.as_raw().is_none());
		assert!(element.as_digest().is_none());
	}

	#[test]
	fn test_as_ref_for_all_variants() {
		// Check unified AsRef across all variants.
		let element_none: DefaultElement = DefaultElement::None;
		assert_eq!(element_none.as_ref(), &[] as &[u8]);

		let raw_data: Vec<u8> = vec![10, 20, 30];
		let bounded: BoundedVec<u8, ConstU32<1024>> =
			raw_data.clone().try_into().expect("Conversion to BoundedVec should succeed");
		let element_raw: DefaultElement = DefaultElement::Raw(bounded);
		assert_eq!(element_raw.as_ref(), &raw_data[..]);

		let digest: DefaultElement = DefaultElement::Digest([1; 32]);
		assert_eq!(digest.as_ref(), &[1; 32][..]);

		let cid: DefaultElement = DefaultElement::CID([2; 64]);
		assert_eq!(cid.as_ref(), &[2; 64][..]);
	}

	#[test]
	fn test_decode_unknown_variant() {
		// Prepare an invalid encoded element with an unknown discriminant tag.
		let invalid_encoded: Vec<u8> = vec![255];
		let result = DefaultElement::decode(&mut &invalid_encoded[..]);
		assert!(result.is_err(), "Decoding should fail for an unknown variant tag");
	}

	// Negative test: decoding an incomplete encoding for Raw variant.
	#[test]
	fn test_decode_incomplete_raw() {
		let raw_data: Vec<u8> = vec![1, 2, 3, 4, 5];
		let bounded: BoundedVec<u8, ConstU32<1024>> =
			raw_data.clone().try_into().expect("Conversion to BoundedVec should succeed");
		let element: DefaultElement = DefaultElement::Raw(bounded);
		let mut encoded = element.encode();
		// Simulate an incomplete encoding by removing the last byte.
		encoded.pop();
		let result = DefaultElement::decode(&mut &encoded[..]);
		assert!(result.is_err(), "Decoding should fail for incomplete Raw encoding");
	}

	// Negative test: decoding an incomplete encoding for Digest variant.
	#[test]
	fn test_decode_incomplete_digest() {
		let digest: [u8; 32] = [0xAA; 32];
		let element: DefaultElement = DefaultElement::Digest(digest);
		let mut encoded = element.encode();
		// Remove a few bytes to truncate the digest data.
		for _ in 0..5 {
			encoded.pop();
		}
		let result = DefaultElement::decode(&mut &encoded[..]);
		assert!(result.is_err(), "Decoding should fail for incomplete Digest encoding");
	}

	// Negative test: decoding an incomplete encoding for CID variant.
	#[test]
	fn test_decode_incomplete_cid() {
		let cid: [u8; 64] = [0x55; 64];
		let element: DefaultElement = DefaultElement::CID(cid);
		let mut encoded = element.encode();
		// Remove a few bytes to simulate incomplete CID data.
		for _ in 0..10 {
			encoded.pop();
		}
		let result = DefaultElement::decode(&mut &encoded[..]);
		assert!(result.is_err(), "Decoding should fail for incomplete CID encoding");
	}

	// Negative test: ensure that creating a BoundedVec with too many elements fails.
	#[test]
	fn test_boundedvec_too_long() {
		// Create a vector longer than the allowed capacity (1024).
		let long_vec: Vec<u8> = vec![0u8; 1024 + 1];
		let bounded: Result<BoundedVec<u8, ConstU32<1024>>, _> = long_vec.try_into();
		assert!(bounded.is_err(), "Creating a BoundedVec with too many elements should fail");
	}

	#[test]
	fn test_integration_with_container() {
		// As an example of integrating DefaultElement in another structure,
		// define a simple container that holds an id and an element.
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
