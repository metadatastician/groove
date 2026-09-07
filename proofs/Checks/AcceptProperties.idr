module Checks.AcceptProperties

import Gossamer.ABI.Types
import Gossamer.ABI.CapabilityAuthenticity

%default total

-- These calls exercise the public propositions with concrete witnesses.
composeScope : Attenuated (Network AllNetwork) (Network (AllowHosts ["example.test"]))
composeScope = attenuateTransitive AttSame AttNetHosts

bothRevoked : RevocationComplete 1 2 (revokePair 1 2 RevEmpty)
bothRevoked = revokePairComplete 1 2 RevEmpty
