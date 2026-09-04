import Lean

namespace XLemma

/-- Marks a declaration for inclusion in an xLemma proof bundle. -/
initialize xlemmaAttr : Lean.TagAttribute ←
  Lean.registerTagAttribute `xlemma "Marks declarations exported to xLemma"

/-- Marks a declaration whose informal LaTeX mapping has been reviewed. -/
initialize xlemmaPresentationReviewedAttr : Lean.TagAttribute ←
  Lean.registerTagAttribute `xlemma_presentation_reviewed
    "Marks declarations whose presentation mapping received explicit review"

end XLemma
