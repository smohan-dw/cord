// This file is part of Cord – https://cord.network

// Copyright (C) 2019-2022 Dhiway Networks Pvt. Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later

// Cord is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Cord is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Cord. If not, see <https://www.gnu.org/licenses/>.

#![warn(unused_crate_dependencies)]

pub mod chain_spec;
#[macro_use]
mod service;

#[cfg(feature = "cli")]
mod cli;
#[cfg(feature = "cli")]
mod command;
#[cfg(feature = "cli")]
mod command_helper;

#[cfg(feature = "cli")]
pub use cli::*;
#[cfg(feature = "cli")]
pub use command::*;
