module Checks.RejectAppData

import Gossamer.ABI.Types
import Gossamer.ABI.CapabilityAuthenticity

%default total

-- Expected failure: an empty path grant cannot widen to AppData authority.
bad : Attenuated (FileSystem (ReadOnlyPaths [])) (FileSystem AppData)
bad = AttFsAppData
