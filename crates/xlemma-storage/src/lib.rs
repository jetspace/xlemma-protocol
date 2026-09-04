//! Content-addressed storage adapter and availability policy helpers.
//!
//! Storage providers preserve and retrieve XLMP artifacts; they do not define
//! artifact validity, research consensus, attribution, or rights.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Component, Path, PathBuf},
    sync::Mutex,
};
use thiserror::Error;
use xlemma_core::{
    canonical_json_bytes, ArtifactEntry, ArtifactId, ArtifactManifest, AvailabilityReceipt, NodeId,
    OperatorClusterId, XLMP_VERSION,
};
use xlemma_xlmp::{AdapterError, ArtifactPayload, StorageAdapter, StoredArtifactBundle};

pub const MAX_BUNDLE_ENTRIES: usize = 4_096;
pub const MAX_BUNDLE_FILE_BYTES: u64 = 64 * 1024 * 1024;
pub const MAX_BUNDLE_BYTES: u64 = 256 * 1024 * 1024;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BundleInput {
    pub relative_path: PathBuf,
    pub media_type: String,
    pub encrypted: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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

pub trait AvailabilityReceiptVerifier {
    fn verify(&self, receipt: &AvailabilityReceipt) -> bool;
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
    #[error("bundle exceeds the configured entry or byte limit")]
    BundleTooLarge,
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("artifact identifier failed: {0}")]
    Id(#[from] xlemma_core::IdError),
    #[error("stored bundle does not exactly match its content-addressed manifest")]
    InvalidBundle,
    #[error("artifact already exists and immutable storage refuses replacement")]
    AlreadyExists,
    #[error("artifact was not found")]
    NotFound,
    #[error("availability receipt signing failed")]
    ReceiptSigning,
    #[error("immutable storage writer lock is unavailable")]
    WriteLock,
    #[error("stored manifest serialization failed: {0}")]
    Json(#[from] serde_json::Error),
}

pub trait AvailabilityReceiptSigner: Send + Sync {
    fn sign(&self, signing_bytes: &[u8]) -> Result<String, StorageError>;
}

/// A local, content-addressed storage implementation suitable for development,
/// clean-room reconstruction tests, and single-operator archival nodes. It is
/// not evidence of multi-provider availability by itself.
pub struct FilesystemStorageAdapter<S> {
    root: PathBuf,
    storage_node_id: NodeId,
    operator_cluster_id: OperatorClusterId,
    provider: String,
    region: String,
    retention: chrono::Duration,
    signer: S,
    write_lock: Mutex<()>,
}

impl<S: AvailabilityReceiptSigner> FilesystemStorageAdapter<S> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        root: PathBuf,
        storage_node_id: NodeId,
        operator_cluster_id: OperatorClusterId,
        provider: String,
        region: String,
        retention: chrono::Duration,
        signer: S,
    ) -> Result<Self, StorageError> {
        storage_node_id.validate()?;
        operator_cluster_id.validate()?;
        if provider.trim().is_empty()
            || region.trim().is_empty()
            || retention <= chrono::Duration::zero()
        {
            return Err(StorageError::InvalidBundle);
        }
        fs::create_dir_all(&root)?;
        let metadata = fs::symlink_metadata(&root)?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err(StorageError::Symlink(root.display().to_string()));
        }
        let root = fs::canonicalize(root)?;
        Ok(Self {
            root,
            storage_node_id,
            operator_cluster_id,
            provider,
            region,
            retention,
            signer,
            write_lock: Mutex::new(()),
        })
    }

    fn artifact_directory(&self, artifact_id: &ArtifactId) -> PathBuf {
        let digest = artifact_id
            .as_str()
            .rsplit(':')
            .next()
            .expect("validated ArtifactID contains a digest");
        self.root.join(digest)
    }

    fn put_bundle(
        &self,
        bundle: StoredArtifactBundle,
    ) -> Result<AvailabilityReceipt, StorageError> {
        validate_stored_bundle(&bundle)?;
        // Production processes still coordinate through a transactional
        // store; this lock prevents calls on one adapter from racing the
        // immutable existence check and final directory rename.
        let _write_guard = self
            .write_lock
            .lock()
            .map_err(|_| StorageError::WriteLock)?;
        let final_directory = self.artifact_directory(&bundle.artifact_id);
        if final_directory.exists() {
            return Err(StorageError::AlreadyExists);
        }
        let observed_at = Utc::now();
        let custody_material = serde_json::json!({
            "artifact_id": bundle.artifact_id,
            "manifest_root": bundle.manifest.root,
            "provider": self.provider,
            "region": self.region,
            "observed_at": observed_at,
        });
        let mut receipt = AvailabilityReceipt {
            receipt_id: xlemma_core::ReceiptId::derive(&"placeholder")?,
            artifact_id: bundle.artifact_id.clone(),
            storage_node_id: self.storage_node_id.clone(),
            operator_cluster_id: self.operator_cluster_id.clone(),
            provider: self.provider.clone(),
            region: self.region.clone(),
            custody_challenge_root: format!(
                "blake3:{}",
                blake3::hash(
                    &canonical_json_bytes(&custody_material)
                        .map_err(|_| StorageError::InvalidBundle)?
                )
                .to_hex()
            ),
            available_until: observed_at
                .checked_add_signed(self.retention)
                .ok_or(StorageError::InvalidBundle)?,
            observed_at,
            signature: String::new(),
        };
        receipt.receipt_id = receipt
            .derive_receipt_id()
            .map_err(|_| StorageError::InvalidBundle)?;
        receipt.signature = self.signer.sign(
            &receipt
                .signing_bytes()
                .map_err(|_| StorageError::InvalidBundle)?,
        )?;
        receipt
            .validate_integrity()
            .map_err(|_| StorageError::ReceiptSigning)?;
        let temporary_directory = self.root.join(format!(
            ".{}.tmp-{}-{}",
            bundle
                .artifact_id
                .as_str()
                .rsplit(':')
                .next()
                .expect("validated ArtifactID contains a digest"),
            std::process::id(),
            Utc::now().timestamp_nanos_opt().unwrap_or_default()
        ));
        fs::create_dir(&temporary_directory)?;
        let write_result = (|| -> Result<(), StorageError> {
            for payload in &bundle.payloads {
                let path = temporary_directory.join(&payload.path);
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent)?;
                }
                let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
                file.write_all(&payload.bytes)?;
                file.sync_all()?;
            }
            let manifest_bytes =
                canonical_json_bytes(&bundle.manifest).map_err(|_| StorageError::InvalidBundle)?;
            let mut manifest_file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(temporary_directory.join(".xlemma-manifest.json"))?;
            manifest_file.write_all(&manifest_bytes)?;
            manifest_file.sync_all()?;
            File::open(&temporary_directory)?.sync_all()?;
            fs::rename(&temporary_directory, &final_directory)?;
            File::open(&self.root)?.sync_all()?;
            Ok(())
        })();
        if let Err(error) = write_result {
            let _ = fs::remove_dir_all(&temporary_directory);
            return Err(error);
        }

        Ok(receipt)
    }

    fn get_bundle(&self, artifact_id: ArtifactId) -> Result<StoredArtifactBundle, StorageError> {
        artifact_id.validate()?;
        let directory = self.artifact_directory(&artifact_id);
        if !directory.exists() {
            return Err(StorageError::NotFound);
        }
        let metadata = fs::symlink_metadata(&directory)?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err(StorageError::Symlink(directory.display().to_string()));
        }
        let canonical_directory = fs::canonicalize(&directory)?;
        let manifest_path = directory.join(".xlemma-manifest.json");
        let mut manifest_file = open_regular_file_without_following_symlinks(&manifest_path)?;
        let mut manifest_bytes = Vec::new();
        Read::by_ref(&mut manifest_file)
            .take(MAX_BUNDLE_FILE_BYTES + 1)
            .read_to_end(&mut manifest_bytes)?;
        if manifest_bytes.len() as u64 > MAX_BUNDLE_FILE_BYTES {
            return Err(StorageError::BundleTooLarge);
        }
        let manifest: ArtifactManifest = serde_json::from_slice(&manifest_bytes)?;
        // Validate declared limits and identity before reading any payload.
        validate_manifest_limits(&manifest)?;
        if manifest.derive_artifact_id()? != artifact_id {
            return Err(StorageError::InvalidBundle);
        }
        let mut payloads = Vec::with_capacity(manifest.entries.len());
        for entry in &manifest.entries {
            validate_relative_path(Path::new(&entry.path))?;
            let path = directory.join(&entry.path);
            let canonical_file = fs::canonicalize(&path)?;
            if !canonical_file.starts_with(&canonical_directory) {
                return Err(StorageError::OutsideRoot(path.display().to_string()));
            }
            let mut file = open_regular_file_without_following_symlinks(&path)?;
            if file.metadata()?.len() != entry.byte_length {
                return Err(StorageError::InvalidBundle);
            }
            let mut bytes = Vec::new();
            Read::by_ref(&mut file)
                .take(entry.byte_length + 1)
                .read_to_end(&mut bytes)?;
            if bytes.len() as u64 != entry.byte_length {
                return Err(StorageError::InvalidBundle);
            }
            payloads.push(ArtifactPayload {
                path: entry.path.clone(),
                bytes,
            });
        }
        let bundle = StoredArtifactBundle {
            artifact_id,
            manifest,
            payloads,
        };
        validate_stored_bundle(&bundle)?;
        Ok(bundle)
    }
}

#[async_trait::async_trait]
impl<S: AvailabilityReceiptSigner> StorageAdapter for FilesystemStorageAdapter<S> {
    async fn put(&self, bundle: StoredArtifactBundle) -> Result<AvailabilityReceipt, AdapterError> {
        self.put_bundle(bundle).map_err(storage_adapter_error)
    }

    async fn get(&self, artifact_id: ArtifactId) -> Result<StoredArtifactBundle, AdapterError> {
        self.get_bundle(artifact_id).map_err(storage_adapter_error)
    }
}

fn storage_adapter_error(error: StorageError) -> AdapterError {
    AdapterError {
        adapter: "filesystem-content-addressed-storage".into(),
        reason: error.to_string(),
    }
}

pub fn validate_stored_bundle(bundle: &StoredArtifactBundle) -> Result<(), StorageError> {
    bundle.artifact_id.validate()?;
    validate_manifest_limits(&bundle.manifest)?;
    if bundle.manifest.derive_artifact_id()? != bundle.artifact_id
        || bundle.payloads.len() != bundle.manifest.entries.len()
        || bundle.payloads.len() > MAX_BUNDLE_ENTRIES
    {
        return Err(StorageError::InvalidBundle);
    }
    if !bundle
        .payloads
        .windows(2)
        .all(|pair| pair[0].path < pair[1].path)
    {
        return Err(StorageError::InvalidBundle);
    }
    let payloads = bundle
        .payloads
        .iter()
        .map(|payload| (payload.path.as_str(), &payload.bytes))
        .collect::<BTreeMap<_, _>>();
    if payloads.len() != bundle.payloads.len() {
        return Err(StorageError::InvalidBundle);
    }
    if payloads.contains_key(".xlemma-manifest.json") {
        return Err(StorageError::InvalidBundle);
    }
    let mut total = 0_u64;
    for entry in &bundle.manifest.entries {
        validate_relative_path(Path::new(&entry.path))?;
        let bytes = payloads
            .get(entry.path.as_str())
            .ok_or(StorageError::InvalidBundle)?;
        total = total
            .checked_add(bytes.len() as u64)
            .ok_or(StorageError::BundleTooLarge)?;
        if bytes.len() as u64 != entry.byte_length
            || entry.content_hash != format!("blake3:{}", blake3::hash(bytes).to_hex())
            || bytes.len() as u64 > MAX_BUNDLE_FILE_BYTES
            || total > MAX_BUNDLE_BYTES
        {
            return Err(StorageError::InvalidBundle);
        }
    }
    Ok(())
}

fn validate_manifest_limits(manifest: &ArtifactManifest) -> Result<(), StorageError> {
    if manifest.entries.is_empty() {
        return Err(StorageError::EmptyBundle);
    }
    if manifest.entries.len() > MAX_BUNDLE_ENTRIES {
        return Err(StorageError::BundleTooLarge);
    }
    let mut total = 0_u64;
    for entry in &manifest.entries {
        total = total
            .checked_add(entry.byte_length)
            .ok_or(StorageError::BundleTooLarge)?;
        if entry.byte_length > MAX_BUNDLE_FILE_BYTES || total > MAX_BUNDLE_BYTES {
            return Err(StorageError::BundleTooLarge);
        }
    }
    Ok(())
}

pub fn build_bundle_manifest(
    root: &Path,
    inputs: &[BundleInput],
    lean_toolchain: impl Into<String>,
    dependency_lock_hash: impl Into<String>,
    source_commit: Option<String>,
    build_image_digest: Option<String>,
) -> Result<BuiltBundle, StorageError> {
    build_bundle_manifest_at(
        root,
        inputs,
        lean_toolchain,
        dependency_lock_hash,
        source_commit,
        build_image_digest,
        Utc::now(),
    )
}

/// Build a bundle using an explicit timestamp so separate implementations can
/// reproduce the complete JSON manifest byte-for-byte, not merely its
/// content-derived ArtifactID.
pub fn build_bundle_manifest_at(
    root: &Path,
    inputs: &[BundleInput],
    lean_toolchain: impl Into<String>,
    dependency_lock_hash: impl Into<String>,
    source_commit: Option<String>,
    build_image_digest: Option<String>,
    created_at: DateTime<Utc>,
) -> Result<BuiltBundle, StorageError> {
    if inputs.is_empty() {
        return Err(StorageError::EmptyBundle);
    }
    if inputs.len() > MAX_BUNDLE_ENTRIES {
        return Err(StorageError::BundleTooLarge);
    }
    if fs::symlink_metadata(root)?.file_type().is_symlink() {
        return Err(StorageError::Symlink(root.display().to_string()));
    }
    let canonical_root = fs::canonicalize(root)?;
    let lean_toolchain = lean_toolchain.into();
    let dependency_lock_hash = dependency_lock_hash.into();
    let mut seen_paths = BTreeSet::new();
    let mut entries = Vec::new();
    let mut bundle_bytes = 0_u64;

    for input in inputs {
        validate_relative_path(&input.relative_path)?;
        let normalized_path = input.relative_path.to_string_lossy().replace('\\', "/");
        if !seen_paths.insert(normalized_path.clone()) {
            return Err(StorageError::DuplicatePath(normalized_path));
        }
        let full_path = root.join(&input.relative_path);
        if input.media_type.trim().is_empty() {
            return Err(StorageError::NotFile(full_path.display().to_string()));
        }
        let metadata = fs::symlink_metadata(&full_path)?;
        if metadata.file_type().is_symlink() {
            return Err(StorageError::Symlink(full_path.display().to_string()));
        }
        if !metadata.is_file() {
            return Err(StorageError::NotFile(full_path.display().to_string()));
        }
        if metadata.len() > MAX_BUNDLE_FILE_BYTES {
            return Err(StorageError::BundleTooLarge);
        }
        bundle_bytes = bundle_bytes
            .checked_add(metadata.len())
            .ok_or(StorageError::BundleTooLarge)?;
        if bundle_bytes > MAX_BUNDLE_BYTES {
            return Err(StorageError::BundleTooLarge);
        }
        let canonical_file = fs::canonicalize(&full_path)?;
        if !canonical_file.starts_with(&canonical_root) {
            return Err(StorageError::OutsideRoot(full_path.display().to_string()));
        }
        let mut file = open_regular_file_without_following_symlinks(&full_path)?;
        let opened_metadata = file.metadata()?;
        if !opened_metadata.is_file() || !same_file(&metadata, &opened_metadata) {
            return Err(StorageError::Symlink(full_path.display().to_string()));
        }
        let mut bytes = Vec::with_capacity(opened_metadata.len() as usize);
        Read::by_ref(&mut file)
            .take(MAX_BUNDLE_FILE_BYTES + 1)
            .read_to_end(&mut bytes)?;
        if bytes.len() as u64 != opened_metadata.len() || bytes.len() as u64 > MAX_BUNDLE_FILE_BYTES
        {
            return Err(StorageError::BundleTooLarge);
        }
        entries.push(ArtifactEntry {
            path: normalized_path,
            media_type: input.media_type.clone(),
            content_hash: format!("blake3:{}", blake3::hash(&bytes).to_hex()),
            byte_length: bytes.len() as u64,
            encrypted: input.encrypted,
        });
    }

    entries.sort_by(|left, right| left.path.cmp(&right.path));
    let mut manifest = ArtifactManifest {
        protocol_version: XLMP_VERSION.to_owned(),
        entries,
        root: String::new(),
        source_commit,
        lean_toolchain,
        dependency_lock_hash,
        build_image_digest,
        created_at,
    };
    manifest.root = manifest.expected_root();
    // Timestamps and source-control labels are provenance metadata, not content
    // identity. Repacking identical bytes under the same verified environment
    // therefore produces the same ArtifactId.
    let artifact_id = manifest.derive_artifact_id()?;
    Ok(BuiltBundle {
        artifact_id,
        manifest,
    })
}

fn open_regular_file_without_following_symlinks(path: &Path) -> Result<File, std::io::Error> {
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    let file = options.open(path)?;
    if !file.metadata()?.is_file() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "bundle input is not a regular file",
        ));
    }
    Ok(file)
}

#[cfg(unix)]
fn same_file(before: &fs::Metadata, after: &fs::Metadata) -> bool {
    use std::os::unix::fs::MetadataExt;
    before.dev() == after.dev() && before.ino() == after.ino()
}

#[cfg(not(unix))]
fn same_file(before: &fs::Metadata, after: &fs::Metadata) -> bool {
    before.len() == after.len()
        && before.modified().ok() == after.modified().ok()
        && before.is_file() == after.is_file()
}

pub fn availability_satisfied(
    artifact_id: &ArtifactId,
    now: DateTime<Utc>,
    policy: &AvailabilityPolicy,
    receipts: &[AvailabilityReceipt],
    verifier: &impl AvailabilityReceiptVerifier,
) -> bool {
    if policy.required_replicas == 0
        || policy.required_operator_clusters == 0
        || policy.required_providers == 0
        || policy.required_regions == 0
    {
        return false;
    }
    let mut receipt_ids = BTreeSet::new();
    let mut node_ids = BTreeSet::new();
    let valid: Vec<_> = receipts
        .iter()
        .filter(|receipt| {
            receipt.artifact_id == *artifact_id
                && receipt.available_until > now
                && receipt.observed_at <= now
                && receipt.receipt_id.validate().is_ok()
                && receipt.storage_node_id.validate().is_ok()
                && receipt.operator_cluster_id.validate().is_ok()
                && !receipt.provider.trim().is_empty()
                && !receipt.region.trim().is_empty()
                && !receipt.custody_challenge_root.trim().is_empty()
                && !receipt.signature.trim().is_empty()
                && verifier.verify(receipt)
                && receipt_ids.insert(receipt.receipt_id.clone())
                && node_ids.insert(receipt.storage_node_id.clone())
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
    if path.to_string_lossy().contains('\\')
        || path.is_absolute()
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

    struct AcceptTestReceipts;

    impl AvailabilityReceiptVerifier for AcceptTestReceipts {
        fn verify(&self, _receipt: &AvailabilityReceipt) -> bool {
            true
        }
    }

    struct TestReceiptSigner;

    impl AvailabilityReceiptSigner for TestReceiptSigner {
        fn sign(&self, _signing_bytes: &[u8]) -> Result<String, StorageError> {
            Ok("ed25519:test-storage-signature".into())
        }
    }

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
        fs::write(
            root.join("proof.lean"),
            "theorem id (p : Prop) (h : p) : p := h\n",
        )
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
    fn published_bundle_vector_is_byte_for_byte_reproducible() {
        let vector_root =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/deterministic-bundle");
        let inputs: Vec<BundleInput> = serde_json::from_slice(
            &fs::read(vector_root.join("inputs.json")).expect("read vector inputs"),
        )
        .expect("parse vector inputs");
        let expected: BuiltBundle = serde_json::from_slice(
            &fs::read(vector_root.join("expected-bundle.json")).expect("read expected bundle"),
        )
        .expect("parse expected bundle");
        let created_at = "2026-09-04T12:00:00Z".parse().unwrap();

        let actual = build_bundle_manifest_at(
            &vector_root,
            &inputs,
            "leanprover/lean4:v4.33.1",
            "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            Some("vector-1".to_owned()),
            Some(
                "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
                    .to_owned(),
            ),
            created_at,
        )
        .expect("build deterministic vector");

        assert_eq!(actual, expected);
        assert_eq!(
            serde_json::to_vec(&actual).unwrap(),
            serde_json::to_vec(&expected).unwrap()
        );
        assert_eq!(
            xlemma_core::canonical_json_bytes(&actual).unwrap(),
            xlemma_core::canonical_json_bytes(&expected).unwrap()
        );

        let mut reversed_inputs = inputs;
        reversed_inputs.reverse();
        let reversed = build_bundle_manifest_at(
            &vector_root,
            &reversed_inputs,
            "leanprover/lean4:v4.33.1",
            "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            Some("vector-1".to_owned()),
            Some(
                "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
                    .to_owned(),
            ),
            created_at,
        )
        .expect("build reordered deterministic vector");
        assert_eq!(reversed, expected);

        let mut mutated = expected.clone();
        mutated.manifest.entries[0].encrypted = true;
        assert!(matches!(
            mutated.manifest.derive_artifact_id(),
            Err(xlemma_core::IdError::InvalidArtifactManifest(_))
        ));

        let mut unsorted = expected;
        unsorted.manifest.entries.reverse();
        assert!(matches!(
            unsorted.manifest.derive_artifact_id(),
            Err(xlemma_core::IdError::InvalidArtifactManifest(_))
        ));
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
            build_bundle_manifest(&root, &[unsafe_input], "lean", "lock", None, None,),
            Err(StorageError::UnsafePath(_))
        ));

        let platform_ambiguous = BundleInput {
            relative_path: PathBuf::from("..\\proof.lean"),
            media_type: "text/x-lean".into(),
            encrypted: false,
        };
        assert!(matches!(
            build_bundle_manifest(&root, &[platform_ambiguous], "lean", "lock", None, None,),
            Err(StorageError::UnsafePath(_))
        ));

        let safe = BundleInput {
            relative_path: PathBuf::from("proof.lean"),
            media_type: "text/x-lean".into(),
            encrypted: false,
        };
        assert!(matches!(
            build_bundle_manifest(&root, &[safe.clone(), safe], "lean", "lock", None, None,),
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
            operator_cluster_id: OperatorClusterId::derive(&format!("operator-{label}")).unwrap(),
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
            &receipts,
            &AcceptTestReceipts
        ));
    }

    #[test]
    fn filesystem_adapter_round_trips_exact_multifile_bundle_and_rejects_overwrite() {
        let source = temp_root("adapter-source");
        fs::create_dir(source.join("nested")).unwrap();
        fs::write(source.join("proof.lean"), "example : True := by trivial\n").unwrap();
        fs::write(source.join("nested/README.md"), "# Reproduce\n").unwrap();
        let inputs = vec![
            BundleInput {
                relative_path: PathBuf::from("proof.lean"),
                media_type: "text/x-lean".into(),
                encrypted: false,
            },
            BundleInput {
                relative_path: PathBuf::from("nested/README.md"),
                media_type: "text/markdown".into(),
                encrypted: false,
            },
        ];
        let built = build_bundle_manifest_at(
            &source,
            &inputs,
            "leanprover/lean4:v4.33.1",
            "blake3:dependency-lock",
            None,
            None,
            "2026-09-04T12:00:00Z".parse().unwrap(),
        )
        .unwrap();
        let payloads = built
            .manifest
            .entries
            .iter()
            .map(|entry| ArtifactPayload {
                path: entry.path.clone(),
                bytes: fs::read(source.join(&entry.path)).unwrap(),
            })
            .collect();
        let bundle = StoredArtifactBundle {
            artifact_id: built.artifact_id,
            manifest: built.manifest,
            payloads,
        };
        let storage_root = temp_root("adapter-store");
        let adapter = FilesystemStorageAdapter::new(
            storage_root.clone(),
            NodeId::derive(&"storage-node").unwrap(),
            OperatorClusterId::derive(&"storage-operator").unwrap(),
            "local-development-provider".into(),
            "local-development-region".into(),
            Duration::days(30),
            TestReceiptSigner,
        )
        .unwrap();

        let receipt = adapter.put_bundle(bundle.clone()).unwrap();
        assert_eq!(receipt.artifact_id, bundle.artifact_id);
        assert!(receipt.validate_integrity().is_ok());
        assert_eq!(
            adapter.get_bundle(bundle.artifact_id.clone()).unwrap(),
            bundle
        );
        assert!(matches!(
            adapter.put_bundle(adapter.get_bundle(bundle.artifact_id).unwrap()),
            Err(StorageError::AlreadyExists)
        ));

        fs::remove_dir_all(source).unwrap();
        fs::remove_dir_all(storage_root).unwrap();
    }
    #[test]
    fn retrieval_checks_declared_budget_before_opening_payloads() {
        let root = temp_root("oversized-declaration");
        let adapter = FilesystemStorageAdapter::new(
            root.clone(),
            NodeId::derive(&"storage").unwrap(),
            OperatorClusterId::derive(&"operator").unwrap(),
            "provider".into(),
            "region".into(),
            Duration::days(1),
            TestReceiptSigner,
        )
        .unwrap();
        let mut manifest: ArtifactManifest =
            serde_json::from_str(include_str!("../../../examples/no-arbitrage/artifact.json"))
                .unwrap();
        manifest.entries[0].byte_length = MAX_BUNDLE_FILE_BYTES + 1;
        // No payload files exist: the declared budget must fail before any open.
        let id = ArtifactId::derive(&"oversized-store").unwrap();
        let directory = adapter.artifact_directory(&id);
        fs::create_dir(&directory).unwrap();
        fs::write(
            directory.join(".xlemma-manifest.json"),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
        assert!(matches!(
            adapter.get_bundle(id),
            Err(StorageError::BundleTooLarge)
        ));
        fs::remove_dir_all(root).unwrap();
    }
}
