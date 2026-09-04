import XLemma.Metadata
import XLemma.Export

namespace XLemma.Example

@[xlemma]
theorem add_zero_verified (n : Nat) : n + 0 = n := by
  exact Nat.add_zero n

#xlemma_export add_zero_verified
#print axioms add_zero_verified

end XLemma.Example
