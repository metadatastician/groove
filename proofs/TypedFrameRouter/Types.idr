-- SPDX-License-Identifier: MPL-2.0
-- Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
--
-- TypedFrameRouter.Types: Core type definitions for the typed frame-level router.
--
-- These types model the router's state machine, frame families,
-- translation directions, splice modes, and connection lifecycle.
-- All types are total.

module TypedFrameRouter.Types

%default total

---------------------------------------------------------------------------
-- Frame family (the two sides of the router)
---------------------------------------------------------------------------

||| The frame family of a socket/link endpoint.
||| The router translates between any two frame families.
public export
data FrameFamily
  = IPv4          -- ^ Internet Protocol version 4
  | IPv6          -- ^ Internet Protocol version 6
  | FibreChannel  -- ^ Fibre Channel (SAN)
  | ISCSI         -- ^ iSCSI (IP-based SCSI)
  | InfiniBand    -- ^ InfiniBand (HPC interconnect)
  | BLE           -- ^ Bluetooth Low Energy
  | Raw           -- ^ Raw frame passthrough

||| A frame translation direction: from one family to another.
||| The type system tracks both source and destination families.
public export
data FrameTranslation = Translate FrameFamily FrameFamily

||| Construct a frame translation from source to destination families.
public export
mkTranslation : FrameFamily -> FrameFamily -> FrameTranslation
mkTranslation src dst = Translate src dst

---------------------------------------------------------------------------
-- Splice mode (zero-copy vs buffered)
---------------------------------------------------------------------------

||| How bytes are transferred between sockets.
public export
data SpliceMode
  = KernelSplice   -- ^ Linux splice(2): zero-copy via kernel pipe buffers
  | UserspaceCopy   -- ^ Fallback: 4KB userspace buffer (macOS, Windows)

||| Maximum userspace buffer size in bytes.
||| This is a provable upper bound on memory usage per connection.
public export
maxBufferSize : Nat
maxBufferSize = 4096

---------------------------------------------------------------------------
-- Router state machine
---------------------------------------------------------------------------

||| States of a single routed connection.
public export
data RouterState
  = Idle          -- ^ No connection, waiting for source accept
  | Accepted      -- ^ Source connection accepted, destination not yet connected
  | Connected     -- ^ Both sides connected, ready to splice
  | Splicing      -- ^ Actively transferring bytes
  | Draining      -- ^ One side closed, draining remaining bytes
  | Closed        -- ^ Both sides closed, resources released

||| Valid state transitions for a routed connection.
||| The type system enforces that connections follow this lifecycle.
public export
data ValidTransition : RouterState -> RouterState -> Type where
  AcceptSource    : ValidTransition Idle Accepted
  ConnectDest     : ValidTransition Accepted Connected
  BeginSplice     : ValidTransition Connected Splicing
  HalfClose       : ValidTransition Splicing Draining
  FullClose       : ValidTransition Draining Closed
  DirectClose     : ValidTransition Accepted Closed  -- Destination connect failed
  AbortSplice     : ValidTransition Splicing Closed   -- Error during splice

---------------------------------------------------------------------------
-- Connection handle (linear)
---------------------------------------------------------------------------

||| A routed connection handle. Linear: must be consumed exactly once.
||| Parameterised by its current state, so the type system tracks the
||| connection lifecycle.
public export
data RouterConn : RouterState -> Type where
  MkRouterConn : (srcHandle  : Bits64)
              -> (dstHandle  : Bits64)
              -> (spliceMode : SpliceMode)
              -> (bytesForward : Nat)   -- bytes sent source→destination
              -> (bytesReverse : Nat)   -- bytes sent destination→source
              -> RouterConn state

---------------------------------------------------------------------------
-- Router configuration
---------------------------------------------------------------------------

||| Configuration for the typed frame router.
public export
record RouterConfig where
  constructor MkRouterConfig
  translation    : FrameTranslation  -- which frame families to translate
  srcBindAddr    : String    -- source bind address
  srcPort        : Bits16
  dstTargetAddr  : String    -- destination target address
  dstPort        : Bits16
  maxConnections : Nat       -- concurrent connection limit
  bufferSize     : Nat       -- userspace buffer size (≤ maxBufferSize)

---------------------------------------------------------------------------
-- Router statistics
---------------------------------------------------------------------------

||| Runtime statistics for the router.
public export
record RouterStats where
  constructor MkRouterStats
  totalConnections  : Nat
  activeConnections : Nat
  bytesRouted       : Nat
  failedConnections : Nat
  spliceMode        : SpliceMode
