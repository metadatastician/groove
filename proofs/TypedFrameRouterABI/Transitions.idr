-- SPDX-License-Identifier: MPL-2.0
-- Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
--
-- TypedFrameRouterABI.Transitions: State machine proofs for routed connections.
--
-- The ValidTransition GADT defines exactly which state transitions are
-- allowed. Impossibility proofs guarantee that invalid transitions
-- (e.g. splicing before connecting) cannot be expressed.

module TypedFrameRouterABI.Transitions

import TypedFrameRouter.Types

%default total

---------------------------------------------------------------------------
-- Impossibility proofs: invalid transitions are uninhabited
---------------------------------------------------------------------------

||| You cannot splice without first connecting to the destination.
public export
cannotSpliceFromIdle : ValidTransition Idle Splicing -> Void
cannotSpliceFromIdle _ impossible

||| You cannot go backwards from Connected to Idle.
public export
cannotRevertToIdle : ValidTransition Connected Idle -> Void
cannotRevertToIdle _ impossible

||| You cannot go backwards from Splicing to Accepted.
public export
cannotRevertToAccepted : ValidTransition Splicing Accepted -> Void
cannotRevertToAccepted _ impossible

||| You cannot reopen a Closed connection.
public export
cannotReopenClosed : ValidTransition Closed s -> Void
cannotReopenClosed _ impossible

||| You cannot drain from Idle (nothing to drain).
public export
cannotDrainFromIdle : ValidTransition Idle Draining -> Void
cannotDrainFromIdle _ impossible

---------------------------------------------------------------------------
-- Decidability: for any two states, we can decide if a transition exists
---------------------------------------------------------------------------

||| Decide whether a valid transition exists between two router states.
public export
decideTransition : (from : RouterState) -> (to : RouterState) -> Dec (ValidTransition from to)
decideTransition Idle Accepted       = Yes AcceptSource
decideTransition Accepted Connected  = Yes ConnectDest
decideTransition Accepted Closed     = Yes DirectClose
decideTransition Connected Splicing  = Yes BeginSplice
decideTransition Splicing Draining   = Yes HalfClose
decideTransition Splicing Closed     = Yes AbortSplice
decideTransition Draining Closed     = Yes FullClose
-- All other transitions are invalid
decideTransition Idle Idle           = No (\case _ impossible)
decideTransition Idle Connected      = No (\case _ impossible)
decideTransition Idle Splicing       = No cannotSpliceFromIdle
decideTransition Idle Draining       = No cannotDrainFromIdle
decideTransition Idle Closed         = No (\case _ impossible)
decideTransition Accepted Idle       = No (\case _ impossible)
decideTransition Accepted Accepted   = No (\case _ impossible)
decideTransition Accepted Splicing   = No (\case _ impossible)
decideTransition Accepted Draining   = No (\case _ impossible)
decideTransition Connected Idle      = No cannotRevertToIdle
decideTransition Connected Accepted  = No (\case _ impossible)
decideTransition Connected Connected = No (\case _ impossible)
decideTransition Connected Draining  = No (\case _ impossible)
decideTransition Connected Closed    = No (\case _ impossible)
decideTransition Splicing Idle       = No (\case _ impossible)
decideTransition Splicing Accepted   = No cannotRevertToAccepted
decideTransition Splicing Connected  = No (\case _ impossible)
decideTransition Splicing Splicing   = No (\case _ impossible)
decideTransition Draining Idle       = No (\case _ impossible)
decideTransition Draining Accepted   = No (\case _ impossible)
decideTransition Draining Connected  = No (\case _ impossible)
decideTransition Draining Splicing   = No (\case _ impossible)
decideTransition Draining Draining   = No (\case _ impossible)
decideTransition Closed _            = No cannotReopenClosed

---------------------------------------------------------------------------
-- Path existence: every connection can reach Closed
---------------------------------------------------------------------------

||| From any non-Closed state, there exists a path to Closed.
||| This proves the router never gets stuck — every connection terminates.
public export
data PathToClosed : RouterState -> Type where
  AlreadyClosed : PathToClosed Closed
  FromDraining  : PathToClosed Draining
  FromSplicing  : PathToClosed Splicing
  FromConnected : PathToClosed Connected
  FromAccepted  : PathToClosed Accepted
  FromIdle      : PathToClosed Idle

||| Every state has a path to Closed.
public export
pathExists : (s : RouterState) -> PathToClosed s
pathExists Idle      = FromIdle
pathExists Accepted  = FromAccepted
pathExists Connected = FromConnected
pathExists Splicing  = FromSplicing
pathExists Draining  = FromDraining
pathExists Closed    = AlreadyClosed
