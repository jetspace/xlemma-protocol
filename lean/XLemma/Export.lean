import Lean
import Lean.Util.CollectAxioms
import Lean.Util.FoldConsts

namespace XLemma

open Lean Lean.Elab Lean.Elab.Command

/-- The versioned encoding consumed by `ClaimID` and `ProofID` derivation. -/
def leanExpressionEncoding : String := "xlemma-lean-expr-v1"

/-- A stable structural encoding of a Lean name, including numeric components. -/
partial def canonicalNameJson : Name → Json
  | .anonymous => Json.arr #["anonymous"]
  | .str parent value => Json.arr #["str", canonicalNameJson parent, value]
  | .num parent value => Json.arr #["num", canonicalNameJson parent, value.repr]

/-- Universe metavariables are not valid in a kernel-checkable protocol object. -/
partial def canonicalLevelJson : Level → Except String Json
  | .zero => pure <| Json.arr #["zero"]
  | .succ level => do
      pure <| Json.arr #["succ", ← canonicalLevelJson level]
  | .max left right => do
      pure <| Json.arr #["max", ← canonicalLevelJson left, ← canonicalLevelJson right]
  | .imax left right => do
      pure <| Json.arr #["imax", ← canonicalLevelJson left, ← canonicalLevelJson right]
  | .param name => pure <| Json.arr #["param", canonicalNameJson name]
  | .mvar _ => throw "unresolved universe metavariable"

def canonicalBinderInfoJson : BinderInfo → Json
  | .default => "default"
  | .implicit => "implicit"
  | .strictImplicit => "strict_implicit"
  | .instImplicit => "instance_implicit"

def canonicalLiteralJson : Literal → Json
  | .natVal value => Json.arr #["nat", value.repr]
  | .strVal value => Json.arr #["string", value]

/--
Serialize the kernel expression tree without source positions, elaborator
metadata, or binder names. Bound variables already use de Bruijn indices, so
omitting binder names makes alpha-equivalent elaborated terms byte-identical.

All natural values are decimal strings, avoiding the JSON/IEEE-754 safe-integer
boundary. Free variables and metavariables fail closed because neither belongs
in a closed, checker-consumable declaration.
-/
partial def canonicalExprJson : Expr → Except String Json
  | .bvar index => pure <| Json.arr #["bound", index.repr]
  | .fvar _ => throw "free variable in exported declaration"
  | .mvar _ => throw "unresolved expression metavariable"
  | .sort level => do
      pure <| Json.arr #["sort", ← canonicalLevelJson level]
  | .const name levels => do
      let encodedLevels ← levels.toArray.mapM canonicalLevelJson
      pure <| Json.arr #["constant", canonicalNameJson name, Json.arr encodedLevels]
  | .app function argument => do
      pure <| Json.arr #["application", ← canonicalExprJson function, ← canonicalExprJson argument]
  | .lam _ type body binderInfo => do
      pure <| Json.arr #[
        "lambda",
        canonicalBinderInfoJson binderInfo,
        ← canonicalExprJson type,
        ← canonicalExprJson body
      ]
  | .forallE _ type body binderInfo => do
      pure <| Json.arr #[
        "forall",
        canonicalBinderInfoJson binderInfo,
        ← canonicalExprJson type,
        ← canonicalExprJson body
      ]
  | .letE _ type value body nondependent => do
      pure <| Json.arr #[
        "let",
        nondependent,
        ← canonicalExprJson type,
        ← canonicalExprJson value,
        ← canonicalExprJson body
      ]
  | .lit literal => pure <| Json.arr #["literal", canonicalLiteralJson literal]
  | .mdata _ expression => canonicalExprJson expression
  | .proj typeName index subject => do
      pure <| Json.arr #[
        "projection",
        canonicalNameJson typeName,
        index.repr,
        ← canonicalExprJson subject
      ]

/-- Canonical bytes represented as a compact JSON string. -/
def canonicalExprString (expression : Expr) : Except String String := do
  pure (Json.compress (← canonicalExprJson expression))

def declarationKind : ConstantInfo → String
  | .axiomInfo _ => "axiom"
  | .defnInfo _ => "definition"
  | .thmInfo _ => "theorem"
  | .opaqueInfo _ => "opaque"
  | .quotInfo _ => "quotient"
  | .inductInfo _ => "inductive"
  | .ctorInfo _ => "constructor"
  | .recInfo _ => "recursor"

private def sortedConstantNames (expressions : Array Expr) (excluded : Name) : Array String :=
  let names := expressions.foldl (init := ({} : NameSet)) fun result expression =>
    result ++ expression.getUsedConstantsAsSet
  names.toArray
    |>.filter (· != excluded)
    |>.qsort Name.lt
    |>.map toString

private def exportDeclaration (name : Name) : CommandElabM Unit := do
  let info ← getConstInfo name
  if info.isUnsafe then
    throwError "refusing to export unsafe declaration '{name}'"
  if info.isPartial then
    throwError "refusing to export partial declaration '{name}'"
  let some proof := info.value? (allowOpaque := true) |
    throwError "declaration '{name}' has no checker-consumable value"
  let typeString ←
    match canonicalExprString info.type with
    | .ok value => pure value
    | .error reason => throwError "cannot export type of '{name}': {reason}"
  let proofString ←
    match canonicalExprString proof with
    | .ok value => pure value
    | .error reason => throwError "cannot export value of '{name}': {reason}"
  let axioms ← collectAxioms name
  let levelParameters := info.levelParams.toArray.map canonicalNameJson
  let dependencies := sortedConstantNames #[info.type, proof] name
  let record := Json.mkObj [
    ("axioms", Json.arr (axioms.qsort Name.lt |>.map (Json.str ∘ toString))),
    ("canonical_declaration_name", Json.compress (canonicalNameJson name)),
    ("canonical_elaborated_type", typeString),
    ("canonical_encoding", leanExpressionEncoding),
    ("canonical_proof_object", proofString),
    ("declaration_kind", declarationKind info),
    ("declaration_name", name.toString),
    ("direct_dependencies", Json.arr (dependencies.map Json.str)),
    ("is_partial", info.isPartial),
    ("is_unsafe", info.isUnsafe),
    ("lean_commit", Lean.githash),
    ("lean_version", Lean.versionString),
    ("level_parameters", Json.arr levelParameters),
    ("protocol_version", "XLMP/1"),
    ("schema", "xlemma-lean-environment-export/v1")
  ]
  logInfo m!"XLMP_LEAN_EXPORT {record.compress}"

syntax (name := xlemmaExport) "#xlemma_export " ident : command

/--
Emit one deterministic, machine-readable record for a closed declaration.
The record itself is evidence only: it never certifies its own proof.
-/
@[command_elab xlemmaExport] def elabXLemmaExport : CommandElab
  | `(#xlemma_export%$tk $id:ident) => withRef tk do
      addCompletionInfo <| CompletionInfo.id id id.getId (danglingDot := false) {} none
      let names ← liftCoreM <| realizeGlobalConstWithInfos id
      match names with
      | [name] => exportDeclaration name
      | [] => throwError "unknown declaration '{id.getId}'"
      | _ => throwError "'{id.getId}' resolves to multiple declarations; use a fully qualified name"
  | _ => throwUnsupportedSyntax

-- Encoding-level guards: binder spelling is presentation metadata and open
-- expressions cannot silently become protocol objects.
#guard match
    canonicalExprString (.lam `x (.const ``Nat []) (.bvar 0) .default),
    canonicalExprString (.lam `renamed (.const ``Nat []) (.bvar 0) .default) with
  | .ok left, .ok right => left == right
  | _, _ => false
#guard match canonicalExprString (.fvar ⟨`free⟩) with
  | .error _ => true
  | .ok _ => false
#guard match canonicalLevelJson (.mvar ⟨`unresolved⟩) with
  | .error _ => true
  | .ok _ => false

end XLemma
