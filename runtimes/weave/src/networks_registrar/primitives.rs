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

use crate::*;

use codec::{CompactAs, Decode, Encode, MaxEncodedLen};
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
