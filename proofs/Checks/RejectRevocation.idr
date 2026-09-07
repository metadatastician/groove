module Checks.RejectRevocation

import Gossamer.ABI.CapabilityAuthenticity

%default total

-- Expected failure: revoking token 1 is not evidence that token 2 is revoked.
bad : RevocationComplete 1 2 (RevAdd 1 RevEmpty)
bad = MkRevComplete RevokedHere RevokedHere
