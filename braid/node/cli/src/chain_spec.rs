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

//! CORD chain configurations.
#![allow(missing_docs)]

pub mod bootstrap;

pub use cord_primitives::{AccountId, Balance, NodeId, Signature};
// use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use sc_chain_spec::ChainSpecExtension;
// use sc_consensus_grandpa::AuthorityId as GrandpaId;
pub use sc_service::{ChainType, Properties};
use serde::{Deserialize, Serialize};
// use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
// use sp_consensus_babe::AuthorityId as BabeId;
use sp_core::{Pair, Public};
use sp_runtime::traits::Verify;
type AccountPublic = <Signature as Verify>::Signer;
use sc_telemetry::TelemetryEndpoints;

#[cfg(feature = "braid-plus-native")]
pub use cord_braid_plus_runtime::genesis_config_presets::{
	braid_plus_development_config_genesis, braid_plus_local_testnet_genesis,
};

#[cfg(feature = "braid-base-native")]
pub use cord_braid_base_runtime::genesis_config_presets::{
	braid_base_development_config_genesis, braid_base_local_testnet_genesis,
};

#[cfg(feature = "braid-twist-native")]
pub use cord_braid_twist_runtime::genesis_config_presets::{
	braid_twist_development_config_genesis, braid_twist_local_testnet_genesis,
};

#[cfg(any(
	feature = "braid-base-native",
	feature = "braid-plus-native",
	feature = "braid-twist-native"
))]
const CORD_TELEMETRY_URL: &str = "wss://telemetry.cord.network/submit/";

const DEFAULT_PROTOCOL_ID: &str = "cord";

/// Node `ChainSpec` extensions.
///
/// Additional parameters for some Substrate core modules,
/// customizable from the chain spec.
#[derive(Default, Clone, Serialize, Deserialize, ChainSpecExtension)]
#[serde(rename_all = "camelCase")]
pub struct Extensions {
	/// Block numbers with known hashes.
	pub fork_blocks: sc_client_api::ForkBlocks<cord_primitives::Block>,
	/// Known bad block hashes.
	pub bad_blocks: sc_client_api::BadBlocks<cord_primitives::Block>,
	/// The light sync state extension used by the sync-state rpc.
	pub light_sync_state: sc_sync_state_rpc::LightSyncStateExtension,
}

// Generic chain spec, in case when we don't have the native runtime.
pub type GenericChainSpec = sc_service::GenericChainSpec<Extensions>;

/// The `ChainSpec` parameterized for the braid base runtime.
#[cfg(feature = "braid-base-native")]
pub type BraidBaseChainSpec = sc_service::GenericChainSpec<Extensions>;

/// The `ChainSpec` parameterized for the braid base runtime.
// Dummy chain spec, but that is fine when we don't have the native runtime.
#[cfg(not(feature = "braid-base-native"))]
pub type BraidBaseChainSpec = GenericChainSpec;

/// The `ChainSpec` parameterized for the braid plus runtime.
#[cfg(feature = "braid-plus-native")]
pub type BraidPlusChainSpec = sc_service::GenericChainSpec<Extensions>;

/// The `ChainSpec` parameterized for braid flow runtime.
// Dummy chain spec, but that is fine when we don't have the native runtime.
#[cfg(not(feature = "braid-plus-native"))]
pub type BraidPlusChainSpec = GenericChainSpec;

/// The `ChainSpec` parameterized for the braid twist runtime.
#[cfg(feature = "braid-twist-native")]
pub type BraidTwistChainSpec = sc_service::GenericChainSpec<Extensions>;

// The `ChainSpec` parameterized for braid flow runtime.
// Dummy chain spec, but that is fine when we don't have the native runtime.
#[cfg(not(feature = "braid-twist-native"))]
pub type BraidTwistChainSpec = GenericChainSpec;

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

/// Helper function to set properties
pub fn get_properties(symbol: &str, decimals: u32, ss58format: u32) -> Properties {
	let mut properties = Properties::new();
	properties.insert("tokenSymbol".into(), symbol.into());
	properties.insert("tokenDecimals".into(), decimals.into());
	properties.insert("ss58Format".into(), ss58format.into());

	properties
}

#[cfg(feature = "braid-base-native")]
pub fn braid_base_development_config() -> Result<BraidBaseChainSpec, String> {
	let properties = get_properties("UNITS", 12, 3893);
	Ok(BraidBaseChainSpec::builder(
		cord_braid_base_runtime::WASM_BINARY.ok_or("Braid Base development wasm not available")?,
		Default::default(),
	)
	.with_name("Braid Base Development")
	.with_id("braid-base-dev")
	.with_chain_type(ChainType::Development)
	.with_genesis_config_patch(braid_base_development_config_genesis())
	.with_telemetry_endpoints(
		TelemetryEndpoints::new(vec![(CORD_TELEMETRY_URL.to_string(), 0)])
			.expect("Cord telemetry url is valid; qed"),
	)
	.with_protocol_id(DEFAULT_PROTOCOL_ID)
	.with_properties(properties)
	.build())
}

#[cfg(feature = "braid-base-native")]
pub fn braid_base_local_config() -> Result<BraidBaseChainSpec, String> {
	let properties = get_properties("UNITS", 12, 3893);
	Ok(BraidBaseChainSpec::builder(
		cord_braid_base_runtime::WASM_BINARY.ok_or("Braid Base wasm not available")?,
		Default::default(),
	)
	.with_name("Braid Base Local Testnet")
	.with_id("braid-base-local")
	.with_chain_type(ChainType::Local)
	.with_genesis_config_preset_name("braid-base-local")
	.with_telemetry_endpoints(
		TelemetryEndpoints::new(vec![(CORD_TELEMETRY_URL.to_string(), 0)])
			.expect("Cord telemetry url is valid; qed"),
	)
	.with_protocol_id(DEFAULT_PROTOCOL_ID)
	.with_properties(properties)
	.build())
}

#[cfg(feature = "braid-plus-native")]
pub fn braid_plus_development_config() -> Result<BraidPlusChainSpec, String> {
	let properties = get_properties("UNITS", 12, 4926);
	Ok(BraidPlusChainSpec::builder(
		cord_braid_plus_runtime::WASM_BINARY.ok_or("Braid Plus development wasm not available")?,
		Default::default(),
	)
	.with_name("Braid Plus Development")
	.with_id("braid-plus-dev")
	.with_chain_type(ChainType::Development)
	.with_genesis_config_patch(braid_plus_local_testnet_genesis())
	.with_telemetry_endpoints(
		TelemetryEndpoints::new(vec![(CORD_TELEMETRY_URL.to_string(), 0)])
			.expect("Cord telemetry url is valid; qed"),
	)
	.with_protocol_id(DEFAULT_PROTOCOL_ID)
	.with_properties(properties)
	.build())
}

#[cfg(feature = "braid-plus-native")]
pub fn braid_plus_local_config() -> Result<BraidPlusChainSpec, String> {
	let properties = get_properties("UNITS", 12, 4926);
	Ok(BraidPlusChainSpec::builder(
		cord_braid_plus_runtime::WASM_BINARY.ok_or("Braid Plus wasm not available")?,
		Default::default(),
	)
	.with_name("Braid Plus Local Testnet")
	.with_id("braid-plus-local")
	.with_chain_type(ChainType::Local)
	.with_genesis_config_patch(braid_plus_local_testnet_genesis())
	.with_telemetry_endpoints(
		TelemetryEndpoints::new(vec![(CORD_TELEMETRY_URL.to_string(), 0)])
			.expect("Cord telemetry url is valid; qed"),
	)
	.with_protocol_id(DEFAULT_PROTOCOL_ID)
	.with_properties(properties)
	.build())
}

#[cfg(feature = "braid-twist-native")]
pub fn braid_twist_development_config() -> Result<BraidTwistChainSpec, String> {
	let properties = get_properties("UNITS", 12, 5126);
	Ok(BraidTwistChainSpec::builder(
		cord_braid_twist_runtime::WASM_BINARY
			.ok_or("Braid Twist development wasm not available")?,
		Default::default(),
	)
	.with_name("Braid Twist Development")
	.with_id("braid-twist-dev")
	.with_chain_type(ChainType::Development)
	.with_genesis_config_patch(braid_twist_local_testnet_genesis())
	.with_telemetry_endpoints(
		TelemetryEndpoints::new(vec![(CORD_TELEMETRY_URL.to_string(), 0)])
			.expect("Cord telemetry url is valid; qed"),
	)
	.with_protocol_id(DEFAULT_PROTOCOL_ID)
	.with_properties(properties)
	.build())
}

#[cfg(feature = "braid-twist-native")]
pub fn braid_twist_local_config() -> Result<BraidTwistChainSpec, String> {
	let properties = get_properties("UNITS", 12, 5126);
	Ok(BraidTwistChainSpec::builder(
		cord_braid_twist_runtime::WASM_BINARY.ok_or("Braid Twist wasm not available")?,
		Default::default(),
	)
	.with_name("Braid Twist Local Testnet")
	.with_id("braid-twist-local")
	.with_chain_type(ChainType::Local)
	.with_genesis_config_patch(braid_twist_local_testnet_genesis())
	.with_telemetry_endpoints(
		TelemetryEndpoints::new(vec![(CORD_TELEMETRY_URL.to_string(), 0)])
			.expect("Cord telemetry url is valid; qed"),
	)
	.with_protocol_id(DEFAULT_PROTOCOL_ID)
	.with_properties(properties)
	.build())
}
