use crate::{
    canonical_json_bytes, ClaimId, IdError, ProofId, TheoryId, TheoryManifest, XLMP_VERSION,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;
use thiserror::Error;

pub const LEAN_EXPORT_SCHEMA: &str = "xlemma-lean-environment-export/v1";
pub const LEAN_EXPRESSION_ENCODING: &str = "xlemma-lean-expr-v1";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LeanDeclarationKind {
    Axiom,
    Definition,
    Theorem,
    Opaque,
    Quotient,
    Inductive,
    Constructor,
    Recursor,
}

/// Machine-readable evidence emitted by the pinned Lean environment exporter.
///
/// This object does not certify itself. A verifier still binds the export to a
/// TheoryID, artifact, dependency lock, challenge, trust policy and independent
/// checker receipts.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LeanEnvironmentExport {
    pub schema: String,
    pub protocol_version: String,
    pub canonical_encoding: String,
    pub lean_version: String,
    pub lean_commit: String,
    pub declaration_name: String,
    pub canonical_declaration_name: String,
    pub declaration_kind: LeanDeclarationKind,
    pub level_parameters: Vec<Value>,
    pub canonical_elaborated_type: String,
    pub canonical_proof_object: String,
    pub direct_dependencies: Vec<String>,
    pub axioms: Vec<String>,
    pub is_unsafe: bool,
    pub is_partial: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LeanDerivedIds {
    pub theory_id: TheoryId,
    pub claim_id: ClaimId,
    pub proof_id: ProofId,
}

impl LeanEnvironmentExport {
    pub fn validate(&self) -> Result<(), LeanExportError> {
        if self.schema != LEAN_EXPORT_SCHEMA {
            return Err(LeanExportError::UnsupportedSchema);
        }
        if self.protocol_version != XLMP_VERSION {
            return Err(LeanExportError::UnsupportedProtocolVersion);
        }
        if self.canonical_encoding != LEAN_EXPRESSION_ENCODING {
            return Err(LeanExportError::UnsupportedCanonicalEncoding);
        }
        if self.is_unsafe || self.is_partial {
            return Err(LeanExportError::UnsafeOrPartialDeclaration);
        }
        if self.lean_version.trim().is_empty() || self.declaration_name.trim().is_empty() {
            return Err(LeanExportError::MissingEnvironmentIdentity);
        }
        if self.lean_commit.len() != 40
            || !self
                .lean_commit
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(LeanExportError::InvalidLeanCommit);
        }
        let declaration_name = parse_canonical_field(
            "canonical_declaration_name",
            &self.canonical_declaration_name,
        )?;
        if !valid_name(&declaration_name) {
            return Err(LeanExportError::InvalidStructuralEncoding(
                "canonical_declaration_name",
            ));
        }
        for level_parameter in &self.level_parameters {
            if !valid_name(level_parameter) {
                return Err(LeanExportError::InvalidStructuralEncoding(
                    "level_parameters",
                ));
            }
        }
        let elaborated_type =
            parse_canonical_field("canonical_elaborated_type", &self.canonical_elaborated_type)?;
        if !valid_expr(&elaborated_type) {
            return Err(LeanExportError::InvalidStructuralEncoding(
                "canonical_elaborated_type",
            ));
        }
        let proof_object =
            parse_canonical_field("canonical_proof_object", &self.canonical_proof_object)?;
        if !valid_expr(&proof_object) {
            return Err(LeanExportError::InvalidStructuralEncoding(
                "canonical_proof_object",
            ));
        }
        validate_unique_nonempty("direct_dependencies", &self.direct_dependencies)?;
        validate_unique_nonempty("axioms", &self.axioms)?;
        Ok(())
    }

    pub fn derive_ids(&self, theory: &TheoryManifest) -> Result<LeanDerivedIds, LeanExportError> {
        self.validate()?;
        if theory.protocol_version != XLMP_VERSION {
            return Err(LeanExportError::TheoryProtocolMismatch);
        }
        if theory.canonical_encoding != self.canonical_encoding {
            return Err(LeanExportError::TheoryEncodingMismatch);
        }
        let expected_toolchain = format!("leanprover/lean4:v{}", self.lean_version);
        if theory.lean_toolchain != expected_toolchain {
            return Err(LeanExportError::TheoryToolchainMismatch);
        }
        for axiom in &self.axioms {
            if !theory.permitted_axioms.contains(axiom) {
                return Err(LeanExportError::AxiomNotPermitted(axiom.clone()));
            }
        }
        let theory_id = TheoryId::derive(theory)?;
        let claim_id =
            ClaimId::from_canonical_elaborated_type(&theory_id, &self.canonical_elaborated_type)?;
        let proof_id =
            ProofId::from_canonical_proof_object(&claim_id, &self.canonical_proof_object)?;
        Ok(LeanDerivedIds {
            theory_id,
            claim_id,
            proof_id,
        })
    }
}

fn parse_canonical_field(field: &'static str, encoded: &str) -> Result<Value, LeanExportError> {
    if encoded.is_empty() {
        return Err(LeanExportError::InvalidCanonicalJson(field));
    }
    let value: Value =
        serde_json::from_str(encoded).map_err(|_| LeanExportError::InvalidCanonicalJson(field))?;
    let canonical =
        canonical_json_bytes(&value).map_err(|_| LeanExportError::InvalidCanonicalJson(field))?;
    if canonical != encoded.as_bytes() {
        return Err(LeanExportError::NonCanonicalJson(field));
    }
    Ok(value)
}

fn tagged_array(value: &Value) -> Option<&[Value]> {
    value.as_array().map(Vec::as_slice)
}

fn has_tag(value: &Value, expected: &str) -> bool {
    value.as_str() == Some(expected)
}

fn valid_decimal_nat(value: &Value) -> bool {
    let Some(value) = value.as_str() else {
        return false;
    };
    value == "0"
        || (value
            .bytes()
            .next()
            .is_some_and(|byte| (b'1'..=b'9').contains(&byte))
            && value.bytes().all(|byte| byte.is_ascii_digit()))
}

fn valid_name(value: &Value) -> bool {
    match tagged_array(value) {
        Some([tag]) => has_tag(tag, "anonymous"),
        Some([tag, parent, name]) if has_tag(tag, "str") => {
            valid_name(parent) && name.as_str().is_some()
        }
        Some([tag, parent, index]) if has_tag(tag, "num") => {
            valid_name(parent) && valid_decimal_nat(index)
        }
        _ => false,
    }
}

fn valid_level(value: &Value) -> bool {
    match tagged_array(value) {
        Some([tag]) => has_tag(tag, "zero"),
        Some([tag, level]) if has_tag(tag, "succ") => valid_level(level),
        Some([tag, left, right]) if has_tag(tag, "max") || has_tag(tag, "imax") => {
            valid_level(left) && valid_level(right)
        }
        Some([tag, name]) if has_tag(tag, "param") => valid_name(name),
        _ => false,
    }
}

fn valid_binder(value: &Value) -> bool {
    matches!(
        value.as_str(),
        Some("default" | "implicit" | "strict_implicit" | "instance_implicit")
    )
}

fn valid_literal(value: &Value) -> bool {
    match tagged_array(value) {
        Some([tag, value]) if has_tag(tag, "nat") => valid_decimal_nat(value),
        Some([tag, value]) if has_tag(tag, "string") => value.as_str().is_some(),
        _ => false,
    }
}

fn valid_expr(value: &Value) -> bool {
    match tagged_array(value) {
        Some([tag, index]) if has_tag(tag, "bound") => valid_decimal_nat(index),
        Some([tag, level]) if has_tag(tag, "sort") => valid_level(level),
        Some([tag, name, levels]) if has_tag(tag, "constant") => {
            valid_name(name)
                && levels
                    .as_array()
                    .is_some_and(|levels| levels.iter().all(valid_level))
        }
        Some([tag, function, argument]) if has_tag(tag, "application") => {
            valid_expr(function) && valid_expr(argument)
        }
        Some([tag, binder, type_, body]) if has_tag(tag, "lambda") || has_tag(tag, "forall") => {
            valid_binder(binder) && valid_expr(type_) && valid_expr(body)
        }
        Some([tag, nondependent, type_, assigned, body]) if has_tag(tag, "let") => {
            nondependent.as_bool().is_some()
                && valid_expr(type_)
                && valid_expr(assigned)
                && valid_expr(body)
        }
        Some([tag, literal]) if has_tag(tag, "literal") => valid_literal(literal),
        Some([tag, type_name, index, subject]) if has_tag(tag, "projection") => {
            valid_name(type_name) && valid_decimal_nat(index) && valid_expr(subject)
        }
        _ => false,
    }
}

fn validate_unique_nonempty(field: &'static str, values: &[String]) -> Result<(), LeanExportError> {
    let mut seen = BTreeSet::new();
    for value in values {
        if value.trim().is_empty() || !seen.insert(value) {
            return Err(LeanExportError::InvalidNameInventory(field));
        }
    }
    Ok(())
}

#[derive(Debug, Error)]
pub enum LeanExportError {
    #[error("unsupported Lean export schema")]
    UnsupportedSchema,
    #[error("unsupported Lean export protocol version")]
    UnsupportedProtocolVersion,
    #[error("unsupported Lean canonical expression encoding")]
    UnsupportedCanonicalEncoding,
    #[error("unsafe or partial declaration cannot enter protocol identity derivation")]
    UnsafeOrPartialDeclaration,
    #[error("Lean version and declaration name must be present")]
    MissingEnvironmentIdentity,
    #[error("Lean commit must be a 40-character lowercase hexadecimal digest")]
    InvalidLeanCommit,
    #[error("{0} is not valid safe canonical JSON")]
    InvalidCanonicalJson(&'static str),
    #[error("{0} is valid JSON but not canonical JSON")]
    NonCanonicalJson(&'static str),
    #[error("{0} is not a valid xlemma-lean-expr-v1 structural value")]
    InvalidStructuralEncoding(&'static str),
    #[error("{0} contains an empty or duplicate Lean name")]
    InvalidNameInventory(&'static str),
    #[error("theory protocol does not match the Lean export")]
    TheoryProtocolMismatch,
    #[error("theory canonical encoding does not match the Lean export")]
    TheoryEncodingMismatch,
    #[error("theory toolchain does not match the exporting Lean release")]
    TheoryToolchainMismatch,
    #[error("exported axiom is not permitted by the theory: {0}")]
    AxiomNotPermitted(String),
    #[error(transparent)]
    Identifier(#[from] IdError),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn export_vector() -> LeanEnvironmentExport {
        serde_json::from_str(include_str!(
            "../../../examples/lean-export/expected-add-zero.json"
        ))
        .unwrap()
    }

    fn theory() -> TheoryManifest {
        serde_json::from_str(include_str!("../../../examples/no-arbitrage/theory.json")).unwrap()
    }

    #[test]
    fn checked_in_export_derives_domain_separated_ids() {
        let ids = export_vector().derive_ids(&theory()).unwrap();
        let expected: LeanDerivedIds = serde_json::from_str(include_str!(
            "../../../examples/lean-export/expected-ids.json"
        ))
        .unwrap();
        assert_eq!(ids, expected);
        assert!(ids.claim_id.as_str().starts_with(ClaimId::PREFIX));
        assert!(ids.proof_id.as_str().starts_with(ProofId::PREFIX));
        assert_ne!(
            ids.claim_id.as_str().split(':').next_back(),
            ids.proof_id.as_str().split(':').next_back()
        );
    }

    #[test]
    fn presentation_name_does_not_change_formal_identity() {
        let export = export_vector();
        let expected = export.derive_ids(&theory()).unwrap();
        let mut renamed = export;
        renamed.declaration_name = "Presentation.Only.Rename".to_owned();
        renamed.canonical_declaration_name = "[\"str\",[\"anonymous\"],\"renamed\"]".to_owned();
        assert_eq!(renamed.derive_ids(&theory()).unwrap(), expected);
    }

    #[test]
    fn type_and_proof_changes_affect_the_correct_identity_layers() {
        let export = export_vector();
        let expected = export.derive_ids(&theory()).unwrap();

        let mut changed_proof = export.clone();
        changed_proof.canonical_proof_object = "[\"bound\",\"0\"]".to_owned();
        let proof_ids = changed_proof.derive_ids(&theory()).unwrap();
        assert_eq!(proof_ids.claim_id, expected.claim_id);
        assert_ne!(proof_ids.proof_id, expected.proof_id);

        let mut changed_type = export;
        changed_type.canonical_elaborated_type = "[\"sort\",[\"zero\"]]".to_owned();
        let type_ids = changed_type.derive_ids(&theory()).unwrap();
        assert_ne!(type_ids.claim_id, expected.claim_id);
        assert_ne!(type_ids.proof_id, expected.proof_id);
    }

    #[test]
    fn unsafe_noncanonical_and_duplicate_evidence_fail_closed() {
        let mut export = export_vector();
        export.is_unsafe = true;
        assert!(matches!(
            export.validate(),
            Err(LeanExportError::UnsafeOrPartialDeclaration)
        ));

        let mut export = export_vector();
        export.canonical_elaborated_type = "[ \"sort\", [\"zero\"] ]".to_owned();
        assert!(matches!(
            export.validate(),
            Err(LeanExportError::NonCanonicalJson(
                "canonical_elaborated_type"
            ))
        ));

        let mut export = export_vector();
        export.axioms = vec!["Classical.choice".to_owned(), "Classical.choice".to_owned()];
        assert!(matches!(
            export.validate(),
            Err(LeanExportError::InvalidNameInventory("axioms"))
        ));

        let mut export = export_vector();
        export.canonical_proof_object = "[\"free\",\"forged\"]".to_owned();
        assert!(matches!(
            export.validate(),
            Err(LeanExportError::InvalidStructuralEncoding(
                "canonical_proof_object"
            ))
        ));

        let export = export_vector();
        let mut mismatched_theory = theory();
        mismatched_theory.lean_toolchain = "leanprover/lean4:v0.0.0".to_owned();
        assert!(matches!(
            export.derive_ids(&mismatched_theory),
            Err(LeanExportError::TheoryToolchainMismatch)
        ));
    }
}
