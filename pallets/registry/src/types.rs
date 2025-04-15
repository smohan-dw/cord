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

use crate::Ss58Identifier;
use bitflags::bitflags;
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::traits::ConstU32;
use frame_support::BoundedVec;
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

bitflags! {
	#[derive(Encode, Decode, TypeInfo, MaxEncodedLen)]
	pub struct Permissions: u32 {
		/// Permission to manage entries.
		const ENTRY = 0b0000_0001;
		/// Permission to add delegates.
		const DELEGATE = 0b0000_0010;
		/// Admin has all rights.
		const ADMIN = 0b0000_0100;
	}
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo)]
pub enum PermissionVariant {
	Entry,
	Delegate,
	Admin,
}

impl Permissions {
	/// Encodes the permission bitflags into a 4-byte array.
	pub fn as_u8(self) -> [u8; 4] {
		let x: u32 = self.bits;
		[
			(x & 0xff) as u8,
			((x >> 8) & 0xff) as u8,
			((x >> 16) & 0xff) as u8,
			((x >> 24) & 0xff) as u8,
		]
	}

	/// Constructs a `Permissions` bitmask from a slice of `PermissionVariant` values.
	pub fn from_variants(variants: &[PermissionVariant]) -> Self {
		let mut perms = Permissions::empty();
		for variant in variants {
			perms |= Permissions::from(variant.clone());
		}
		perms
	}
}

impl Default for Permissions {
	fn default() -> Self {
		Permissions::ENTRY
	}
}

impl From<PermissionVariant> for Permissions {
	fn from(variant: PermissionVariant) -> Self {
		match variant {
			PermissionVariant::Entry => Permissions::ENTRY,
			PermissionVariant::Delegate => Permissions::DELEGATE,
			PermissionVariant::Admin => Permissions::ADMIN,
		}
	}
}

/// A simple status enum.
#[derive(Encode, Decode, Clone, MaxEncodedLen, RuntimeDebug, PartialEq, Eq, TypeInfo)]
pub enum Status {
	Active,
	Archived,
}

#[derive(Encode, Decode, Clone, MaxEncodedLen, RuntimeDebug, PartialEq, Eq, TypeInfo)]
pub struct RegistryDetails<Account, Hash, Status> {
	/// The account that created the registry.
	pub author: Account,
	/// The identifier associated with the spec document.
	pub spec_id: Hash,
	/// Optionally, the document identifier as a bounded vector.
	pub byte_store_id: BoundedVec<u8, ConstU32<32>>,
	/// Optionally, the node identifier as a bounded vector.
	pub schema_id: Option<Ss58Identifier>,
	/// The status of the registry entry.
	pub status: Status,
}

/// This struct holds the details for a Registry Object.
#[derive(Encode, Decode, Clone, MaxEncodedLen, RuntimeDebug, PartialEq, Eq, TypeInfo)]
pub struct ObjectDetails<Account, Hash, Status> {
	/// The account that created (and paid for) this state.
	pub creator: Account,
	/// Unique transaction reference for this state.
	pub entry_tx_ref: Hash,
	/// Optional document identifier.
	pub entry_doc_id: Option<BoundedVec<u8, ConstU32<64>>>,
	/// Optional document author identifier.
	pub entry_author_id: Option<Account>,
	/// Optional node identifier.
	pub entry_node_id: Option<Ss58Identifier>,
	/// The current status of this object state.
	pub status: Status,
}
