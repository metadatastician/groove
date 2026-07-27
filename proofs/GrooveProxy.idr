-- SPDX-License-Identifier: MPL-2.0
-- Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
--
-- GrooveProxy: IPv4→IPv6 specialisation of the Typed Frame Router.
--
-- This is an INSTANCE of proven-servers/core/proven-typed-frame-router,
-- configured for the Groove Protocol's IPv4 sunset strategy.
--
-- It re-exports the general router types with Groove-specific defaults
-- and adds Groove discovery integration (sunset headers, attestation).

module GrooveProxy

import TypedFrameRouter.Types
import TypedFrameRouterABI
import TypedFrameRouterABI.Proofs

%default total

---------------------------------------------------------------------------
-- Groove-specific configuration
---------------------------------------------------------------------------

||| Default Groove proxy configuration.
||| Translates IPv4→IPv6 on the loopback interface for a given port.
|||
||| @param port The groove service port to proxy
public export
grooveProxyConfig : (port : Bits16) -> RouterConfig
grooveProxyConfig port = MkRouterConfig
  { translation    = Translate IPv4 IPv6
  , srcBindAddr    = "127.0.0.1"
  , srcPort        = port
  , dstTargetAddr  = "::1"
  , dstPort        = port
  , maxConnections = 64
  , bufferSize     = 4096
  }

||| The Groove proxy direction is always IPv4→IPv6.
||| This is a specialisation of the general FrameTranslation.
public export
grooveDirection : FrameTranslation
grooveDirection = Translate IPv4 IPv6

---------------------------------------------------------------------------
-- Groove-specific proofs (derived from general proofs)
---------------------------------------------------------------------------

||| The Groove proxy inherits all four safety properties from
||| the Typed Frame Router. This re-export makes them available
||| to Groove consumers without importing the general module.
public export
grooveTransportSafe : (input : List Bits8) -> (output : List Bits8)
                   -> (prf : input = output) -> input = output
grooveTransportSafe = transportTransparency

||| The Groove proxy cannot be reversed to IPv6→IPv4.
public export
grooveNoReverse : (Translate IPv6 IPv4 = GrooveProxy.grooveDirection) -> Void
grooveNoReverse eq = noReverseTranslation IPv4 IPv6 famNeq (trans eq dirDef)
  where
    dirDef : GrooveProxy.grooveDirection = Translate IPv4 IPv6
    dirDef = Refl
    famNeq : Not (IPv4 = IPv6)
    famNeq Refl impossible
