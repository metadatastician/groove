-- SPDX-License-Identifier: MPL-2.0
-- Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
--
-- TypedFrameRouterABI: Top-level ABI module for proven-typed-frame-router.
--
-- Re-exports all ABI sub-modules:
--   - Layout:      Bits8 tag encodings with roundtrip proofs
--   - Transitions: ValidTransition GADT, impossibility proofs, decidability
--   - Foreign:     Opaque handle types and FFI function contracts
--   - Proofs:      Transport transparency, bounded memory, liveness

module TypedFrameRouterABI

import public TypedFrameRouterABI.Layout
import public TypedFrameRouterABI.Foreign
import public TypedFrameRouterABI.Transitions
import public TypedFrameRouterABI.Proofs

%default total
