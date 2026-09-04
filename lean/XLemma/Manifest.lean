namespace XLemma

structure TheoryMetadata where
  protocolVersion : String := "XLMP/1"
  trustPolicyId : String
  checkerPolicyId : String
  dependencyRoot : String
  permittedAxioms : Array String := #[]
  deriving Repr, Inhabited

structure PresentationMetadata where
  latexLabel : String
  licenseId : String
  visibility : String := "public"
  deriving Repr, Inhabited

structure ExportMetadata where
  theory : TheoryMetadata
  presentation : Option PresentationMetadata := none
  rightsManifestHash : String
  contributionManifestHash : String
  deriving Repr, Inhabited

end XLemma
