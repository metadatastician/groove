// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//
// Library surface of the groove CLI, so integration tests and sibling crates
// (the reference provider) can reuse the registry, manifest validation, and
// probing logic without shelling out to the binary.

#![forbid(unsafe_code)]

pub mod canonical;
pub mod detect;
pub mod init;
pub mod probe;
pub mod registry;
pub mod sign;
pub mod timefmt;
pub mod validate;
