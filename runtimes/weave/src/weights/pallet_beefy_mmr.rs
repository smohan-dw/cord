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

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::Weight};
use core::marker::PhantomData;

/// Weight functions for `pallet_balances`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_beefy_mmr::WeightInfo for WeightInfo<T> {
	fn extract_validation_context() -> Weight {
	  Weight::from_parts(0, 0)
		.saturating_add(Weight::from_parts(0, 0))
		.saturating_add(T::DbWeight::get().reads(1))
		.saturating_add(T::DbWeight::get().writes(1))
	}

	fn read_peak() -> Weight {
	  Weight::from_parts(0, 0)
		.saturating_add(Weight::from_parts(0, 0))
		.saturating_add(T::DbWeight::get().reads(1))
		.saturating_add(T::DbWeight::get().writes(1))
	}

	fn n_items_proof_is_non_canonical(_: u32, ) -> Weight {
	  Weight::from_parts(0, 0)
		.saturating_add(Weight::from_parts(0, 0))
		.saturating_add(T::DbWeight::get().reads(1))
		.saturating_add(T::DbWeight::get().writes(1))
	}
}
