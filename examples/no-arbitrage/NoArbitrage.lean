import XLemma

namespace XLemma.NoArbitrage

/-- Minimal example market. This is a protocol demonstration, not a substantive
    theorem of financial economics. -/
structure Market where
  hasArbitrage : Prop

/-- For the example only, no-free-lunch is defined as absence of arbitrage. -/
def Market.noFreeLunch (market : Market) : Prop := ¬ market.hasArbitrage

@[xlemma]
theorem noArbitrage (market : Market) (h : market.noFreeLunch) :
    ¬ market.hasArbitrage := by
  exact h

#xlemma_export noArbitrage
#print axioms noArbitrage

end XLemma.NoArbitrage
