-- SPDX-License-Identifier: MPL-2.0
-- Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
--
-- TypedFrameRouterABI.Layout: C-ABI-compatible tag encodings with roundtrip proofs.
--
-- Maps every constructor of the router's sum types to fixed Bits8 values
-- for C interop. Each type gets:
--   * a total encoder  (xToTag : X -> Bits8)
--   * a partial decoder (tagToX : Bits8 -> Maybe X)
--   * a roundtrip lemma proving encoding then decoding is identity
--
-- Tag values MUST match generated/abi/typed_frame_router.h and
-- ffi/zig/src/typed_frame_router.zig.

module TypedFrameRouterABI.Layout

import TypedFrameRouter.Types

%default total

---------------------------------------------------------------------------
-- FrameFamily (7 constructors, tags 0-6)
---------------------------------------------------------------------------

public export
frameFamilySize : Nat
frameFamilySize = 1

public export
frameFamilyToTag : FrameFamily -> Bits8
frameFamilyToTag IPv4         = 0
frameFamilyToTag IPv6         = 1
frameFamilyToTag FibreChannel = 2
frameFamilyToTag ISCSI        = 3
frameFamilyToTag InfiniBand   = 4
frameFamilyToTag BLE          = 5
frameFamilyToTag Raw          = 6

public export
tagToFrameFamily : Bits8 -> Maybe FrameFamily
tagToFrameFamily 0 = Just IPv4
tagToFrameFamily 1 = Just IPv6
tagToFrameFamily 2 = Just FibreChannel
tagToFrameFamily 3 = Just ISCSI
tagToFrameFamily 4 = Just InfiniBand
tagToFrameFamily 5 = Just BLE
tagToFrameFamily 6 = Just Raw
tagToFrameFamily _ = Nothing

public export
frameFamilyRoundtrip : (f : FrameFamily) -> tagToFrameFamily (frameFamilyToTag f) = Just f
frameFamilyRoundtrip IPv4         = Refl
frameFamilyRoundtrip IPv6         = Refl
frameFamilyRoundtrip FibreChannel = Refl
frameFamilyRoundtrip ISCSI        = Refl
frameFamilyRoundtrip InfiniBand   = Refl
frameFamilyRoundtrip BLE          = Refl
frameFamilyRoundtrip Raw          = Refl

---------------------------------------------------------------------------
-- SpliceMode (2 constructors, tags 0-1)
---------------------------------------------------------------------------

public export
spliceModeSize : Nat
spliceModeSize = 1

public export
spliceModeToTag : SpliceMode -> Bits8
spliceModeToTag KernelSplice  = 0
spliceModeToTag UserspaceCopy = 1

public export
tagToSpliceMode : Bits8 -> Maybe SpliceMode
tagToSpliceMode 0 = Just KernelSplice
tagToSpliceMode 1 = Just UserspaceCopy
tagToSpliceMode _ = Nothing

public export
spliceModeRoundtrip : (m : SpliceMode) -> tagToSpliceMode (spliceModeToTag m) = Just m
spliceModeRoundtrip KernelSplice  = Refl
spliceModeRoundtrip UserspaceCopy = Refl

---------------------------------------------------------------------------
-- RouterState (6 constructors, tags 0-5)
---------------------------------------------------------------------------

public export
routerStateSize : Nat
routerStateSize = 1

public export
routerStateToTag : RouterState -> Bits8
routerStateToTag Idle      = 0
routerStateToTag Accepted  = 1
routerStateToTag Connected = 2
routerStateToTag Splicing  = 3
routerStateToTag Draining  = 4
routerStateToTag Closed    = 5

public export
tagToRouterState : Bits8 -> Maybe RouterState
tagToRouterState 0 = Just Idle
tagToRouterState 1 = Just Accepted
tagToRouterState 2 = Just Connected
tagToRouterState 3 = Just Splicing
tagToRouterState 4 = Just Draining
tagToRouterState 5 = Just Closed
tagToRouterState _ = Nothing

public export
routerStateRoundtrip : (s : RouterState) -> tagToRouterState (routerStateToTag s) = Just s
routerStateRoundtrip Idle      = Refl
routerStateRoundtrip Accepted  = Refl
routerStateRoundtrip Connected = Refl
routerStateRoundtrip Splicing  = Refl
routerStateRoundtrip Draining  = Refl
routerStateRoundtrip Closed    = Refl
