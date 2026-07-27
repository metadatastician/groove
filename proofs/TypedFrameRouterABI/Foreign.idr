-- SPDX-License-Identifier: MPL-2.0
-- Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
--
-- TypedFrameRouterABI.Foreign: FFI function contracts for the Zig implementation.
--
-- Declares opaque handle types and the C-compatible function signatures
-- that the Zig FFI layer must implement. These are contracts: the Zig
-- code must match these signatures exactly.

module TypedFrameRouterABI.Foreign

import TypedFrameRouter.Types

%default total

---------------------------------------------------------------------------
-- Opaque handle types
---------------------------------------------------------------------------

||| Opaque handle to the router server instance.
||| Created by typed_frame_router_start, consumed by typed_frame_router_stop.
export
data RouterHandle : Type where [external]

||| Opaque handle to a single routed connection.
||| Created on accept, consumed on close.
export
data ConnHandle : Type where [external]

---------------------------------------------------------------------------
-- FFI function contracts
---------------------------------------------------------------------------

||| Start the typed frame router.
||| Returns an opaque handle that MUST be consumed by typed_frame_router_stop.
|||
||| @param src_addr   Source bind address (e.g. "127.0.0.1")
||| @param src_port   Source listen port
||| @param dst_addr   Destination target address (e.g. "::1")
||| @param dst_port   Destination target port
||| @returns          Opaque server handle, or negative error code
export
%foreign "C:typed_frame_router_start,typed_frame_router"
typed_frame_router_start : String -> Bits16 -> String -> Bits16 -> PrimIO RouterHandle

||| Stop the router and release all resources.
||| Consumes the server handle.
|||
||| @param handle  The server handle from typed_frame_router_start
export
%foreign "C:typed_frame_router_stop,typed_frame_router"
typed_frame_router_stop : RouterHandle -> PrimIO ()

||| Get current router statistics.
|||
||| @param handle  The server handle
||| @returns       JSON-encoded stats string
export
%foreign "C:typed_frame_router_stats,typed_frame_router"
typed_frame_router_stats : RouterHandle -> PrimIO String

||| Check if kernel splice(2) is available on this platform.
|||
||| @returns  1 if splice is available (Linux), 0 if fallback (macOS/Windows)
export
%foreign "C:typed_frame_router_has_splice,typed_frame_router"
typed_frame_router_has_splice : PrimIO Bits8

||| Get the number of active connections.
|||
||| @param handle  The server handle
||| @returns       Number of currently active routed connections
export
%foreign "C:typed_frame_router_active_count,typed_frame_router"
typed_frame_router_active_count : RouterHandle -> PrimIO Bits32
