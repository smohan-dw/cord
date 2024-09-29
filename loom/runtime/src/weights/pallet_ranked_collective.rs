// Copyright (C) Parity Technologies and the various Polkadot contributors, see Contributions.md
// for a list of specific contributors.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//! Autogenerated weights for `pallet_ranked_collective`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 32.0.0
//! DATE: 2024-03-10, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `ggwpez-ref-hw`, CPU: `Intel(R) Xeon(R) CPU @ 2.60GHz`
//! WASM-EXECUTION: `Compiled`, CHAIN: `Some("./kusama-chain-spec.json")`, DB CACHE: 1024

// Executed Command:
// ./target/production/polkadot
// benchmark
// pallet
// --chain=./kusama-chain-spec.json
// --steps=50
// --repeat=20
// --pallet=pallet_ranked_collective
// --extrinsic=*
// --wasm-execution=compiled
// --heap-pages=4096
// --output=./kusama-weights/
// --header=./file_header.txt

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::Weight};
use core::marker::PhantomData;

/// Weight functions for `pallet_ranked_collective`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_ranked_collective::WeightInfo for WeightInfo<T> {
	/// Storage: `FellowshipCollective::Members` (r:1 w:1)
	/// Proof: `FellowshipCollective::Members` (`max_values`: None, `max_size`: Some(42), added: 2517, mode: `MaxEncodedLen`)
	/// Storage: `FellowshipCollective::MemberCount` (r:1 w:1)
	/// Proof: `FellowshipCollective::MemberCount` (`max_values`: None, `max_size`: Some(14), added: 2489, mode: `MaxEncodedLen`)
	/// Storage: `FellowshipCollective::IndexToId` (r:0 w:1)
	/// Proof: `FellowshipCollective::IndexToId` (`max_values`: None, `max_size`: Some(54), added: 2529, mode: `MaxEncodedLen`)
	/// Storage: `FellowshipCollective::IdToIndex` (r:0 w:1)
	/// Proof: `FellowshipCollective::IdToIndex` (`max_values`: None, `max_size`: Some(54), added: 2529, mode: `MaxEncodedLen`)
	fn add_member() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `76`
		//  Estimated: `3507`
		// Minimum execution time: 13_448_000 picoseconds.
		Weight::from_parts(14_194_000, 0)
			.saturating_add(Weight::from_parts(0, 3507))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	/// Storage: `FellowshipCollective::Members` (r:1 w:1)
	/// Proof: `FellowshipCollective::Members` (`max_values`: None, `max_size`: Some(42), added: 2517, mode: `MaxEncodedLen`)
	/// Storage: `FellowshipCollective::MemberCount` (r:11 w:11)
	/// Proof: `FellowshipCollective::MemberCount` (`max_values`: None, `max_size`: Some(14), added: 2489, mode: `MaxEncodedLen`)
	/// Storage: `FellowshipCollective::IdToIndex` (r:11 w:22)
	/// Proof: `FellowshipCollective::IdToIndex` (`max_values`: None, `max_size`: Some(54), added: 2529, mode: `MaxEncodedLen`)
	/// Storage: `FellowshipCollective::IndexToId` (r:11 w:22)
	/// Proof: `FellowshipCollective::IndexToId` (`max_values`: None, `max_size`: Some(54), added: 2529, mode: `MaxEncodedLen`)
	/// The range of component `r` is `[0, 10]`.
	fn remove_member(r: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `550 + r * (281 ±0)`
		//  Estimated: `3519 + r * (2529 ±0)`
		// Minimum execution time: 28_905_000 picoseconds.
		Weight::from_parts(29_964_830, 0)
			.saturating_add(Weight::from_parts(0, 3519))
			// Standard Error: 23_687
			.saturating_add(Weight::from_parts(14_141_927, 0).saturating_mul(r.into()))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().reads((3_u64).saturating_mul(r.into())))
			.saturating_add(T::DbWeight::get().writes(6))
			.saturating_add(T::DbWeight::get().writes((5_u64).saturating_mul(r.into())))
			.saturating_add(Weight::from_parts(0, 2529).saturating_mul(r.into()))
	}
	/// Storage: `FellowshipCollective::Members` (r:1 w:1)
	/// Proof: `FellowshipCollective::Members` (`max_values`: None, `max_size`: Some(42), added: 2517, mode: `MaxEncodedLen`)
	/// Storage: `FellowshipCollective::MemberCount` (r:1 w:1)
	/// Proof: `FellowshipCollective::MemberCount` (`max_values`: None, `max_size`: Some(14), added: 2489, mode: `MaxEncodedLen`)
	/// Storage: `FellowshipCollective::IndexToId` (r:0 w:1)
	/// Proof: `FellowshipCollective::IndexToId` (`max_values`: None, `max_size`: Some(54), added: 2529, mode: `MaxEncodedLen`)
	/// Storage: `FellowshipCollective::IdToIndex` (r:0 w:1)
	/// Proof: `FellowshipCollective::IdToIndex` (`max_values`: None, `max_size`: Some(54), added: 2529, mode: `MaxEncodedLen`)
	/// The range of component `r` is `[0, 10]`.
	fn promote_member(r: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `248 + r * (17 ±0)`
		//  Estimated: `3507`
		// Minimum execution time: 15_834_000 picoseconds.
		Weight::from_parts(16_860_772, 0)
			.saturating_add(Weight::from_parts(0, 3507))
			// Standard Error: 3_920
			.saturating_add(Weight::from_parts(315_171, 0).saturating_mul(r.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	/// Storage: `FellowshipCollective::Members` (r:1 w:1)
	/// Proof: `FellowshipCollective::Members` (`max_values`: None, `max_size`: Some(42), added: 2517, mode: `MaxEncodedLen`)
	/// Storage: `FellowshipCollective::MemberCount` (r:1 w:1)
	/// Proof: `FellowshipCollective::MemberCount` (`max_values`: None, `max_size`: Some(14), added: 2489, mode: `MaxEncodedLen`)
	/// Storage: `FellowshipCollective::IdToIndex` (r:1 w:2)
	/// Proof: `FellowshipCollective::IdToIndex` (`max_values`: None, `max_size`: Some(54), added: 2529, mode: `MaxEncodedLen`)
	/// Storage: `FellowshipCollective::IndexToId` (r:1 w:2)
	/// Proof: `FellowshipCollective::IndexToId` (`max_values`: None, `max_size`: Some(54), added: 2529, mode: `MaxEncodedLen`)
	/// The range of component `r` is `[0, 10]`.
	fn demote_member(r: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `566 + r * (72 ±0)`
		//  Estimated: `3519`
		// Minimum execution time: 28_675_000 picoseconds.
		Weight::from_parts(31_321_840, 0)
			.saturating_add(Weight::from_parts(0, 3519))
			// Standard Error: 16_060
			.saturating_add(Weight::from_parts(604_750, 0).saturating_mul(r.into()))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(6))
	}
	/// Storage: `FellowshipCollective::Members` (r:1 w:0)
	/// Proof: `FellowshipCollective::Members` (`max_values`: None, `max_size`: Some(42), added: 2517, mode: `MaxEncodedLen`)
	/// Storage: `FellowshipReferenda::ReferendumInfoFor` (r:1 w:1)
	/// Proof: `FellowshipReferenda::ReferendumInfoFor` (`max_values`: None, `max_size`: Some(900), added: 3375, mode: `MaxEncodedLen`)
	/// Storage: `FellowshipCollective::Voting` (r:1 w:1)
	/// Proof: `FellowshipCollective::Voting` (`max_values`: None, `max_size`: Some(65), added: 2540, mode: `MaxEncodedLen`)
	/// Storage: `Scheduler::Agenda` (r:2 w:2)
	/// Proof: `Scheduler::Agenda` (`max_values`: None, `max_size`: Some(38963), added: 41438, mode: `MaxEncodedLen`)
	fn vote() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `637`
		//  Estimated: `83866`
		// Minimum execution time: 38_869_000 picoseconds.
		Weight::from_parts(39_938_000, 0)
			.saturating_add(Weight::from_parts(0, 83866))
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	/// Storage: `FellowshipReferenda::ReferendumInfoFor` (r:1 w:0)
	/// Proof: `FellowshipReferenda::ReferendumInfoFor` (`max_values`: None, `max_size`: Some(900), added: 3375, mode: `MaxEncodedLen`)
	/// Storage: `FellowshipCollective::VotingCleanup` (r:1 w:0)
	/// Proof: `FellowshipCollective::VotingCleanup` (`max_values`: None, `max_size`: Some(114), added: 2589, mode: `MaxEncodedLen`)
	/// Storage: `FellowshipCollective::Voting` (r:100 w:100)
	/// Proof: `FellowshipCollective::Voting` (`max_values`: None, `max_size`: Some(65), added: 2540, mode: `MaxEncodedLen`)
	/// The range of component `n` is `[0, 100]`.
	fn cleanup_poll(n: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `434 + n * (50 ±0)`
		//  Estimated: `4365 + n * (2540 ±0)`
		// Minimum execution time: 15_758_000 picoseconds.
		Weight::from_parts(18_451_862, 0)
			.saturating_add(Weight::from_parts(0, 4365))
			// Standard Error: 1_997
			.saturating_add(Weight::from_parts(1_127_986, 0).saturating_mul(n.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(n.into())))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(n.into())))
			.saturating_add(Weight::from_parts(0, 2540).saturating_mul(n.into()))
	}
	/// Storage: `FellowshipCollective::Members` (r:2 w:2)
	/// Proof: `FellowshipCollective::Members` (`max_values`: None, `max_size`: Some(42), added: 2517, mode: `MaxEncodedLen`)
	/// Storage: `FellowshipCollective::MemberCount` (r:2 w:2)
	/// Proof: `FellowshipCollective::MemberCount` (`max_values`: None, `max_size`: Some(14), added: 2489, mode: `MaxEncodedLen`)
	/// Storage: `FellowshipCollective::IdToIndex` (r:2 w:4)
	/// Proof: `FellowshipCollective::IdToIndex` (`max_values`: None, `max_size`: Some(54), added: 2529, mode: `MaxEncodedLen`)
	/// Storage: `FellowshipCollective::IndexToId` (r:0 w:2)
	/// Proof: `FellowshipCollective::IndexToId` (`max_values`: None, `max_size`: Some(54), added: 2529, mode: `MaxEncodedLen`)
	fn exchange_member() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `371`
		//  Estimated: `6048`
		// Minimum execution time: 44_643_000 picoseconds.
		Weight::from_parts(45_415_000, 0)
			.saturating_add(Weight::from_parts(0, 6048))
			.saturating_add(T::DbWeight::get().reads(6))
			.saturating_add(T::DbWeight::get().writes(10))
	}
}
