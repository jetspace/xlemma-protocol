//! Content-addressed proof bundles and availability policy helpers.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    fs,
    path::{Component, Path, PathBuf},
};
use thiserror::Error;
use xlemma_core::{ArtifactEntry, ArtifactId, ArtifactManifest, AvailabilityReceipt};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BundleInput {
    pub relative_path: PathBuf,
    pub media_type: String,
    pub encrypted: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BuiltBundle {
    pub artifact_id: ArtifactId,
    pub manifest: ArtifactManifest,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AvailabilityPolicy {
    pub required_replicas: usize,
    pub required_operator_clusters: usize,
    pub required_providers: usize,
    pub required_regions: usize,
}

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("bundle must contain at least one file")]
    EmptyBundle,
    #[error("bundle contains a duplicate path: {0}")]
    DuplicatePath(String),
    #[error("bundle entry resolves outside the declared root: {0}")]
    OutsideRoot(String),
    #[error("bundle path is absolute or contains traversal components: {0}")]
    UnsafePath(String),
    #[error("bundle entry is a symlink: {0}")]
    Symlink(String),
    #[error("bundle entry is not a regular file: {0}")]
    NotFile(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("artifact identifier failed: {0}")]
    Id(#[from] xlemma_core::IdError),
}

#[derive(Serialize)]
struct ArtifactIdentityMaterial<'a> {
    protocol_version: &'a str,
    entries: &'a [ArtifactEntry],
    root: &'a str,
    lean_toolchain: &'a str,
    dependency_lock_hash: &'a str,
    build_image_digest: &'a Option<String>,
}

pub fn build_bundle_manifest(
    root: &Path,
    inputs: &[BundleInput],
    lean_toolchain: impl Into<String>,
    dependency_lock_hash: impl Into<String>,
    source_commit: Option<String>,
    build_image_digest: Option<String>,
) -> Result<BuiltBundle, StorageError> {
    if inputs.is_empty() {
        return Err(StorageError::EmptyBundle);
    }
    let canonical_root = fs::canonicalize(root)?;
    let lean_toolchain = lean_toolchain.into();
    let dependency_lock_hash = dependency_lock_hash.into();
    let mut seen_paths = BTreeSet::new();
    let mut entries = Vec::new();

    for input in inputs {
        validate_relative_path(&input.relative_path)?;
        let normalized_path = input.relative_path.to_string_lossy().replace('\\', "/");
        if !seen_paths.insert(normalized_path.clone()) {
            return Err(StorageError::DuplicatePath(normalized_path));
        }
        let full_path = root.join(&input.relative_path);
        let metadata = fs::symlink_metadata(&full_path)?;
        if metadata.file_type().is_symlink() {
            return Err(StorageError::Symlink(full_path.display().to_string()));
        }
        if !metadata.is_file() {
            return Err(StorageError::NotFile(full_path.display().to_string()));
        }
        let canonical_file = fs::canonicalize(&full_path)?;
        if !canonical_file.starts_with(&canonical_root) {
            return Err(StorageError::OutsideRoot(full_path.display().to_string()));
        }
        let bytes = fs::read(&canonical_file)?;
        entries.push(ArtifactEntry {
            path: normalized_path,
            media_type: input.media_type.clone(),
            content_hash: format!("blake3:{}", blake3::hash(&bytes).to_hex()),
            byte_length: bytes.len() as u64,
            encrypted: input.encrypted,
        });
    }

    entries.sort_by(|left, right| left.path.cmp(&right.path));
    let mut root_hasher = blake3::Hasher::new();
    root_hasher.update(b"xlemma-artifact-root-v1\0");
    for entry in &entries {
        root_hasher.update(entry.path.as_bytes());
        root_hasher.update(b"\0");
        root_hasher.update(entry.content_hash.as_bytes());
        root_hasher.update(b"\0");
        root_hasher.update(&entry.byte_length.to_le_bytes());
    }
    let root_hash = format!("blake3:{}", root_hasher.finalize().to_hex());

    let manifest = ArtifactManifest {
        protocol_version: "xlemma/0.2".to_owned(),
        entries,
        root: root_hash,
        source_commit,
        lean_toolchain,
        dependency_lock_hash,
        build_image_digest,
        created_at: Utc::now(),
    };
    // Timestamps and source-control labels are provenance metadata, not content
    // identity. Repacking identical bytes under the same verified environment
    // therefore produces the same ArtifactId.
    let identity = ArtifactIdentityMaterial {
        protocol_version: &manifest.protocol_version,
        entries: &manifest.entries,
        root: &manifest.root,
        lean_toolchain: &manifest.lean_toolchain,
        dependency_lock_hash: &manifest.dependency_lock_hash,
        build_image_digest: &manifest.build_image_digest,
    };
    let artifact_id = ArtifactId::derive(&identity)?;
    Ok(BuiltBundle {
        artifact_id,
        manifest,
    })
}

pub fn availability_satisfied(
    artifact_id: &ArtifactId,
    now: DateTime<Utc>,
    policy: &AvailabilityPolicy,
    receipts: &[AvailabilityReceipt],
) -> bool {
    let valid: Vec<_> = receipts
        .iter()
        .filter(|receipt| {
            receipt.artifact_id == *artifact_id
                && receipt.available_until > now
                && receipt.observed_at <= now
        })
        .collect();
    let operators: BTreeSet<_> = valid
        .iter()
        .map(|receipt| receipt.operator_cluster_id.clone())
        .collect();
    let providers: BTreeSet<_> = valid
        .iter()
        .map(|receipt| receipt.provider.as_str())
        .collect();
    let regions: BTreeSet<_> = valid
        .iter()
        .map(|receipt| receipt.region.as_str())
        .collect();

    valid.len() >= policy.required_replicas
        && operators.len() >= policy.required_operator_clusters
        && providers.len() >= policy.required_providers
        && regions.len() >= policy.required_regions
}

fn validate_relative_path(path: &Path) -> Result<(), StorageError> {
    if path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(StorageError::UnsafePath(path.display().to_string()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use std::time::{SystemTime, UNIX_EPOCH};
    use xlemma_core::{NodeId, OperatorClusterId, ReceiptId};

    fn temp_root(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "xlemma-storage-{label}-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn identical_content_and_environment_have_stable_artifact_id() {
        let root = temp_root("stable");
        fs::write(root.join("proof.lean"), "theorem id (p : Prop) (h : p) : p := h\n")
            .unwrap();
        let inputs = vec![BundleInput {
            relative_path: PathBuf::from("proof.lean"),
            media_type: "text/x-lean".into(),
            encrypted: false,
        }];

        let first = build_bundle_manifest(
            &root,
            &inputs,
            "v4.33.1",
            "blake3:lock",
            Some("commit-a".into()),
            Some("sha256:image".into()),
        )
        .unwrap();
        let second = build_bundle_manifest(
            &root,
            &inputs,
            "v4.33.1",
            "blake3:lock",
            Some("commit-b".into()),
            Some("sha256:image".into()),
        )
        .unwrap();

        assert_eq!(first.artifact_id, second.artifact_id);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn traversal_and_duplicate_paths_are_rejected() {
        let root = temp_root("unsafe");
        fs::write(root.join("proof.lean"), "example : True := by trivial\n").unwrap();
        let unsafe_input = BundleInput {
            relative_path: PathBuf::from("../proof.lean"),
            media_type: "text/x-lean".into(),
            encrypted: false,
        };
        assert!(matches!(
            build_bundle_manifest(
                &root,
                &[unsafe_input],
                "lean",
                "lock",
                None,
                None,
            ),
            Err(StorageError::UnsafePath(_))
        ));

        let safe = BundleInput {
            relative_path: PathBuf::from("proof.lean"),
            media_type: "text/x-lean".into(),
            encrypted: false,
        };
        assert!(matches!(
            build_bundle_manifest(
                &root,
                &[safe.clone(), safe],
                "lean",
                "lock",
                None,
                None,
            ),
            Err(StorageError::DuplicatePath(_))
        ));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn availability_requires_matching_live_independent_receipts() {
        let artifact_id = ArtifactId::derive(&"artifact").unwrap();
        let other_artifact = ArtifactId::derive(&"other-artifact").unwrap();
        let now = Utc::now();
        let make_receipt = |label: &str, artifact_id: ArtifactId| AvailabilityReceipt {
            receipt_id: ReceiptId::derive(&format!("receipt-{label}")).unwrap(),
            artifact_id,
            storage_node_id: NodeId::derive(&format!("node-{label}")).unwrap(),
            operator_cluster_id: OperatorClusterId::derive(&format!("operator-{label}"))
                .unwrap(),
            provider: format!("provider-{label}"),
            region: format!("region-{label}"),
            custody_challenge_root: format!("blake3:custody-{label}"),
            available_until: now + Duration::days(1),
            observed_at: now - Duration::minutes(1),
            signature: "test".into(),
        };
        let receipts = vec![
            make_receipt("a", artifact_id.clone()),
            make_receipt("b", artifact_id.clone()),
            make_receipt("c", other_artifact),
        ];
        let policy = AvailabilityPolicy {
            required_replicas: 2,
            required_operator_clusters: 2,
            required_providers: 2,
            required_regions: 2,
        };
        assert!(availability_satisfied(
            &artifact_id,
            now,
            &policy,
            &receipts
        ));
    }
}
