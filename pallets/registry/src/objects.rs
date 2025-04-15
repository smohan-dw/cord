use crate::{
	pallet::Pallet, CordAccountOf, Error, HashOf, ObjectDetails, ObjectHistory, ObjectIdentifierOf,
	ObjectState, Status, C,
};
use alloc::vec::Vec;
use codec::Encode;
use frame_support::dispatch::DispatchResult;
use frame_support::ensure;
use frame_support::pallet_prelude::*;
use frame_support::BoundedVec;
use sp_runtime::traits::Hash;

pub fn add_object<T: crate::Config>(
	creator: CordAccountOf<T>,
	object_id: ObjectIdentifierOf,
	entry_tx_ref: HashOf<T>,
	entry_doc_id: Option<Vec<u8>>,
	entry_author_id: Option<CordAccountOf<T>>,
	entry_node_id: Option<Vec<u8>>,
) -> DispatchResult {
	let bounded_doc_id = entry_doc_id
		.map(|v| v.try_into())
		.transpose()
		.map_err(|_| Error::<T>::InvalidIdentifierLength)?;
	let bounded_node_id = entry_node_id
		.map(|v| v.try_into())
		.transpose()
		.map_err(|_| Error::<T>::InvalidIdentifierLength)?;
	let encoded_doc_author = entry_author_id.as_ref().map(|author| author.encode());

	let mut data = Vec::with_capacity(256);
	data.extend_from_slice(entry_tx_ref.as_ref());
	Pallet::<T>::push_option(
		&mut data,
		bounded_doc_id.as_ref().map(|v: &BoundedVec<u8, ConstU32<64>>| v.as_slice()),
	);
	Pallet::<T>::push_option(&mut data, encoded_doc_author.as_ref().map(|v| v.as_slice()));
	Pallet::<T>::push_option(
		&mut data,
		bounded_node_id.as_ref().map(|v: &BoundedVec<u8, ConstU32<64>>| v.as_slice()),
	);
	data.extend_from_slice(&creator.encode());

	let digest = T::Hashing::hash(&data);
	let pallet_name = <crate::pallet::Pallet<T> as frame_support::traits::PalletInfoAccess>::name();

	let registry_id =
		<cord_uri::Pallet<T> as Identifier>::build(&(digest).encode()[..], pallet_name)
			.map_err(|_| Error::<T>::InvalidIdentifierLength)?;

	let details = ObjectDetails {
		creator: creator.clone(),
		entry_tx_ref,
		entry_doc_id: bounded_doc_id,
		entry_author_id,
		entry_node_id: bounded_node_id,
		status: Status::Active,
	};
	// Ensure that this specific state does not already exist.
	ensure!(
		!ObjectHistory::<T>::contains_key(&object_id, entry_tx_ref),
		Error::<T>::ObjectAlreadyExists
	);
	ObjectHistory::<T>::insert(&object_id, entry_tx_ref, details);
	ObjectState::<T>::insert(&object_id, entry_tx_ref);
	Ok(())
}

pub fn update_object<T: crate::Config>(
	object_id: ObjectIdentifierOf,
	new_entry_tx_ref: HashOf<T>,
	new_entry_doc_id: Option<Vec<u8>>,
	new_entry_author_id: Option<CordAccountOf<T>>,
	new_entry_node_id: Option<Vec<u8>>,
	updater: CordAccountOf<T>,
) -> DispatchResult {
	// Get current active state.
	let current_hash = ObjectState::<T>::get(&object_id).ok_or(Error::<T>::ObjectNotFound)?;
	// Revoke the current state.
	ObjectHistory::<T>::try_mutate(&object_id, current_hash, |maybe_details| -> DispatchResult {
		if let Some(details) = maybe_details {
			ensure!(details.status == Status::Active, Error::<T>::ObjectNotActive);
			details.status = Status::Revoked;
		}
		Ok(())
	})?;
	let new_bounded_doc_id = new_entry_doc_id
		.map(|v| v.try_into())
		.transpose()
		.map_err(|_| Error::<T>::InvalidIdentifierLength)?;
	let new_bounded_node_id = new_entry_node_id
		.map(|v| v.try_into())
		.transpose()
		.map_err(|_| Error::<T>::InvalidIdentifierLength)?;
	let new_details = ObjectDetails {
		creator: updater.clone(), // You might also preserve the original creator if required.
		entry_tx_ref: new_entry_tx_ref,
		entry_doc_id: new_bounded_doc_id,
		entry_author_id: new_entry_author_id,
		entry_node_id: new_bounded_node_id,
		status: Status::Active,
	};
	ObjectHistory::<T>::insert(&object_id, new_entry_tx_ref, new_details);
	ObjectState::<T>::insert(&object_id, new_entry_tx_ref);
	Ok(())
}

pub fn revoke_object<T: crate::Config>(
	object_id: ObjectIdentifierOf,
	caller: CordAccountOf<T>,
) -> DispatchResult {
	let current_hash = ObjectState::<T>::get(&object_id).ok_or(Error::<T>::ObjectNotFound)?;
	ObjectHistory::<T>::try_mutate(&object_id, current_hash, |maybe_details| -> DispatchResult {
		let details = maybe_details.as_mut().ok_or(Error::<T>::ObjectNotFound)?;
		// Only the object creator or someone with ADMIN rights may revoke.
		ensure!(
			details.creator == caller
				|| crate::pallet::Pallet::<T>::has_permission(
					&object_id,
					&caller,
					crate::Permissions::ADMIN
				),
			Error::<T>::UnauthorizedOperation
		);
		details.status = Status::Revoked;
		Ok(())
	})
}

pub fn restore_object<T: crate::Config>(
	object_id: ObjectIdentifierOf,
	caller: CordAccountOf<T>,
) -> DispatchResult {
	let current_hash = ObjectState::<T>::get(&object_id).ok_or(Error::<T>::ObjectNotFound)?;
	ObjectHistory::<T>::try_mutate(&object_id, current_hash, |maybe_details| -> DispatchResult {
		let details = maybe_details.as_mut().ok_or(Error::<T>::ObjectNotFound)?;
		ensure!(
			details.creator == caller
				|| crate::pallet::Pallet::<T>::has_permission(
					&object_id,
					&caller,
					crate::Permissions::ADMIN
				),
			Error::<T>::UnauthorizedOperation
		);
		details.status = Status::Active;
		Ok(())
	})
}
