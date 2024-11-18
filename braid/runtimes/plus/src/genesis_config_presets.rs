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

//! Genesis configs presets for the CORD Loom runtime

use crate::{MaxMembers, SessionKeys, BABE_GENESIS_EPOCH_CONFIG};
use cord_braid_plus_runtime_constants::currency::UNITS;
pub use cord_primitives::{AccountId, AccountPublic, Balance, NodeId, Signature};
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_core::{sr25519, Pair, Public};
use sp_runtime::{traits::IdentifyAccount, BoundedVec};
#[cfg(not(feature = "std"))]
use sp_std::alloc::format;
use sp_std::vec;
use sp_std::vec::Vec;

/// Helper function to generate a crypto pair from seed
fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

/// Helper function to generate an account ID from seed
fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Helper function to generate stash, controller and session key from seed
fn get_authority_keys_from_seed(
	seed: &str,
) -> (AccountId, BabeId, GrandpaId, AuthorityDiscoveryId) {
	let keys = get_authority_keys_from_seed_no_beefy(seed);
	(keys.0, keys.1, keys.2, keys.3)
}

/// Helper function to generate stash, controller and session key from seed
fn get_authority_keys_from_seed_no_beefy(
	seed: &str,
) -> (AccountId, BabeId, GrandpaId, AuthorityDiscoveryId) {
	(
		get_account_id_from_seed::<sr25519::Public>(seed),
		get_from_seed::<BabeId>(seed),
		get_from_seed::<GrandpaId>(seed),
		get_from_seed::<AuthorityDiscoveryId>(seed),
	)
}

fn braid_plus_session_keys(
	babe: BabeId,
	grandpa: GrandpaId,
	authority_discovery: AuthorityDiscoveryId,
) -> SessionKeys {
	SessionKeys { babe, grandpa, authority_discovery }
}

fn braid_plus_testnet_genesis(
	initial_authorities: Vec<(AccountId, BabeId, GrandpaId, AuthorityDiscoveryId)>,
	initial_well_known_nodes: Vec<(NodeId, AccountId)>,
	root_key: AccountId,
) -> serde_json::Value {
	const ENDOWMENT: u128 = 500_000_000 * UNITS;

	serde_json::json!( {
	"balances": {
		"balances": initial_authorities.iter().map(|k| (k.0.clone(), ENDOWMENT)).collect::<Vec<_>>(),
	},
	"networkParameters": {"permissioned": true},
	"nodeAuthorization":  {
		"nodes": initial_well_known_nodes.iter().map(|x| (x.0.clone(), x.1.clone())).collect::<Vec<_>>(),
	},
	"authorityMembership":  {
		"initialAuthorities": initial_authorities
			.iter()
			.map(|x| x.0.clone())
			.collect::<Vec<_>>(),
	},
	"session":  {
		"keys": initial_authorities
			.iter()
			.map(|x| {
				(
					x.0.clone(),
					x.0.clone(),
					braid_plus_session_keys(
						x.1.clone(),
						x.2.clone(),
						x.3.clone(),
					),
				)
			})
			.collect::<Vec<_>>(),
	},
	"babe":  {
		"epochConfig": Some(BABE_GENESIS_EPOCH_CONFIG),
	},
	"councilMembership":  {
		"members": BoundedVec::<_, MaxMembers>::try_from(
			initial_authorities.iter().map(|x| x.0.clone()).collect::<Vec<_>>(),
		)
		.expect("Too many members"),
	},
	"technicalMembership":  {
		"members": BoundedVec::<_, MaxMembers>::try_from(
			initial_authorities.iter().map(|x| x.0.clone()).collect::<Vec<_>>(),
		)
		.expect("Too many members"),

	},
	"sudo": { "key": Some(root_key) },
	})
}

pub fn braid_plus_local_testnet_genesis() -> serde_json::Value {
	braid_plus_testnet_genesis(
		vec![
			get_authority_keys_from_seed("Alice"),
			get_authority_keys_from_seed("Bob"),
			get_authority_keys_from_seed("Charlie"),
		],
		vec![
			(
				b"12D3KooWBmAwcd4PJNJvfV89HwE48nwkRmAgo8Vy3uQEyNNHBox2".to_vec(),
				get_account_id_from_seed::<sr25519::Public>("Alice"),
			),
			(
				b"12D3KooWQYV9dGMFoRzNStwpXztXaBUjtPqi6aU76ZgUriHhKust".to_vec(),
				get_account_id_from_seed::<sr25519::Public>("Bob"),
			),
			(
				b"12D3KooWJvyP3VJYymTqG7eH4PM5rN4T2agk5cdNCfNymAqwqcvZ".to_vec(),
				get_account_id_from_seed::<sr25519::Public>("Charlie"),
			),
			(
				b"12D3KooWPHWFrfaJzxPnqnAYAoRUyAHHKqACmEycGTVmeVhQYuZN".to_vec(),
				get_account_id_from_seed::<sr25519::Public>("Dave"),
			),
		],
		get_account_id_from_seed::<sr25519::Public>("Alice"),
	)
}

pub fn braid_plus_development_config_genesis() -> serde_json::Value {
	braid_plus_testnet_genesis(
		vec![get_authority_keys_from_seed("Alice")],
		vec![(
			b"12D3KooWBmAwcd4PJNJvfV89HwE48nwkRmAgo8Vy3uQEyNNHBox2".to_vec(),
			get_account_id_from_seed::<sr25519::Public>("Alice"),
		)],
		get_account_id_from_seed::<sr25519::Public>("Alice"),
	)
}
