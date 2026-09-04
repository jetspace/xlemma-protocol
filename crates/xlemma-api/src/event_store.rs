//! Durable, tamper-evident API state journal.
//!
//! Every acknowledged state mutation is appended as one canonical JSON record,
//! flushed to the operating system, and linked to the preceding record. The
//! journal intentionally stores protocol records rather than application logs:
//! authentication tokens, private keys, and request headers never enter it.

use crate::VerificationJobRecord;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Read, Write},
    path::{Path, PathBuf},
    sync::Mutex,
};
use thiserror::Error;
use xlemma_core::{canonical_json_bytes, canonical_json_hash};
use xlemma_xlmp::{ObservationCommitMessage, XlmpEnvelope, XlmpMessage};

const JOURNAL_VERSION: &str = "xlemma-api-event-journal-v1";
const JOURNAL_HASH_DOMAIN: &str = "api-event-journal-entry-v1";
const GENESIS_HASH: &str =
    "blake3:0000000000000000000000000000000000000000000000000000000000000000";
const MAX_SAFE_SEQUENCE: u64 = 9_007_199_254_740_991;
const MAX_ENTRY_BYTES: u64 = 64 * 1024 * 1024;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "event_type", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum ApiJournalEvent {
    MessageAccepted {
        envelope: XlmpEnvelope,
    },
    VerificationJobCreated {
        job: VerificationJobRecord,
    },
    VerificationJobUpdated {
        job: VerificationJobRecord,
        expected_previous_updated_at: DateTime<Utc>,
    },
}

#[derive(Debug, Serialize)]
struct JournalIdentity<'a> {
    journal_version: &'static str,
    sequence: u64,
    previous_hash: &'a str,
    event: &'a ApiJournalEvent,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct JournalEntry {
    journal_version: String,
    sequence: u64,
    previous_hash: String,
    event_hash: String,
    event: ApiJournalEvent,
}

impl JournalEntry {
    fn new(
        sequence: u64,
        previous_hash: String,
        event: ApiJournalEvent,
    ) -> Result<Self, EventStoreError> {
        if sequence > MAX_SAFE_SEQUENCE {
            return Err(EventStoreError::SequenceExhausted);
        }
        let event_hash = entry_hash(sequence, &previous_hash, &event)?;
        Ok(Self {
            journal_version: JOURNAL_VERSION.to_owned(),
            sequence,
            previous_hash,
            event_hash,
            event,
        })
    }

    fn verify(
        &self,
        expected_sequence: u64,
        expected_previous: &str,
    ) -> Result<(), EventStoreError> {
        if self.journal_version != JOURNAL_VERSION
            || self.sequence != expected_sequence
            || self.previous_hash != expected_previous
            || self.event_hash != entry_hash(self.sequence, &self.previous_hash, &self.event)?
        {
            return Err(EventStoreError::IntegrityViolation(self.sequence));
        }
        Ok(())
    }
}

fn entry_hash(
    sequence: u64,
    previous_hash: &str,
    event: &ApiJournalEvent,
) -> Result<String, EventStoreError> {
    let digest = canonical_json_hash(
        JOURNAL_HASH_DOMAIN,
        &JournalIdentity {
            journal_version: JOURNAL_VERSION,
            sequence,
            previous_hash,
            event,
        },
    )?;
    Ok(format!("blake3:{}", hex::encode(digest)))
}

#[derive(Clone, Default)]
pub(crate) struct RecoveredState {
    pub projection: xlemma_xlmp::ProtocolProjection,
    pub jobs: BTreeMap<String, VerificationJobRecord>,
    pub messages: BTreeMap<String, XlmpEnvelope>,
    pub observation_commits: BTreeMap<String, ObservationCommitMessage>,
}

impl RecoveredState {
    fn apply(&mut self, event: &ApiJournalEvent) -> Result<(), EventStoreError> {
        match event {
            ApiJournalEvent::MessageAccepted { envelope } => {
                envelope
                    .validate_integrity()
                    .map_err(|error| EventStoreError::InvalidEvent(error.to_string()))?;
                xlemma_xlmp::verify_ed25519_signature(envelope)
                    .map_err(|error| EventStoreError::InvalidEvent(error.to_string()))?;
                if let XlmpMessage::Certificate(message) = &envelope.message {
                    let job = self
                        .jobs
                        .get(message.certificate.job_id.as_str())
                        .ok_or_else(|| {
                            EventStoreError::MissingRecord(message.certificate.job_id.to_string())
                        })?;
                    crate::validate_certificate_evidence(
                        job,
                        &message.certificate,
                        &self.messages,
                        Utc::now(),
                    )
                    .map_err(|_| {
                        EventStoreError::InvalidEvent("invalid certificate evidence".into())
                    })?;
                }
                self.projection
                    .apply(envelope)
                    .map_err(|error| EventStoreError::InvalidEvent(error.to_string()))?;
                let message_id = envelope.message_id.to_string();
                if self.messages.contains_key(&message_id) {
                    return Err(EventStoreError::DuplicateRecord(message_id));
                }
                if let XlmpMessage::ObservationCommit(commit) = &envelope.message {
                    let receipt_id = commit.receipt_id.to_string();
                    if self.observation_commits.contains_key(&receipt_id) {
                        return Err(EventStoreError::DuplicateRecord(receipt_id));
                    }
                    self.observation_commits.insert(receipt_id, commit.clone());
                }
                self.messages.insert(message_id, envelope.clone());
            }
            ApiJournalEvent::VerificationJobCreated { job } => {
                let job_id = job.job_id.to_string();
                if self.jobs.insert(job_id.clone(), job.clone()).is_some() {
                    return Err(EventStoreError::DuplicateRecord(job_id));
                }
            }
            ApiJournalEvent::VerificationJobUpdated {
                job,
                expected_previous_updated_at,
            } => {
                let job_id = job.job_id.to_string();
                let prior = self
                    .jobs
                    .get(&job_id)
                    .ok_or_else(|| EventStoreError::MissingRecord(job_id.clone()))?;
                let immutable_fields_match = prior.job_id == job.job_id
                    && prior.researcher_id == job.researcher_id
                    && prior.claim_id == job.claim_id
                    && prior.theory_id == job.theory_id
                    && prior.artifact_id == job.artifact_id
                    && prior.artifact_root == job.artifact_root
                    && prior.policy_id == job.policy_id
                    && canonical_json_bytes(&prior.policy)? == canonical_json_bytes(&job.policy)?
                    && prior.committee_members == job.committee_members
                    && prior.maximum_budget_minor_units == job.maximum_budget_minor_units
                    && prior.settlement_asset == job.settlement_asset
                    && prior.created_at == job.created_at;
                let observations_are_valid = job.observations.iter().all(|observation| {
                    observation.job_id == job.job_id && observation.validate_integrity().is_ok()
                });
                let valid_observation_update = observations_are_valid
                    && match (prior.state, job.state) {
                        (
                            xlemma_core::VerificationState::ClaimCommitted,
                            xlemma_core::VerificationState::CheckersRevealed,
                        ) => job.observations.len() == 1,
                        (
                            xlemma_core::VerificationState::CheckersRevealed,
                            xlemma_core::VerificationState::CheckersRevealed,
                        ) => {
                            job.observations.len() == prior.observations.len()
                                || job.observations.len() == prior.observations.len() + 1
                        }
                        (
                            xlemma_core::VerificationState::CheckersRevealed,
                            xlemma_core::VerificationState::Passed
                            | xlemma_core::VerificationState::Failed
                            | xlemma_core::VerificationState::Divergent
                            | xlemma_core::VerificationState::Quarantined,
                        ) => job.observations.len() == prior.observations.len(),
                        _ => false,
                    }
                    && job.observations.starts_with(&prior.observations);
                if prior.updated_at != *expected_previous_updated_at
                    || job.updated_at < *expected_previous_updated_at
                    || !immutable_fields_match
                    || !valid_observation_update
                {
                    return Err(EventStoreError::InvalidTransition(job_id));
                }
                self.jobs.insert(job_id, job.clone());
            }
        }
        Ok(())
    }
}

struct JournalWriter {
    file: File,
    next_sequence: u64,
    previous_hash: String,
    state: RecoveredState,
    failed: bool,
}

pub(crate) struct EventJournal {
    path: PathBuf,
    writer: Mutex<JournalWriter>,
}

impl EventJournal {
    pub fn open(path: impl AsRef<Path>) -> Result<(Self, RecoveredState), EventStoreError> {
        let path = path.as_ref().to_path_buf();
        if path.as_os_str().is_empty() {
            return Err(EventStoreError::InvalidPath);
        }
        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            std::fs::create_dir_all(parent)?;
        }

        let mut options = OpenOptions::new();
        options.create(true).read(true).append(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options
                .mode(0o600)
                .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
        }
        let file = options.open(&path)?;
        if !file.metadata()?.is_file() {
            return Err(EventStoreError::InvalidPath);
        }
        #[cfg(unix)]
        {
            use std::os::fd::AsRawFd;
            // SAFETY: file owns this valid descriptor for the journal lifetime.
            // The nonblocking advisory lock is released when that file closes.
            if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
                return Err(EventStoreError::Io(std::io::Error::last_os_error()));
            }
            file.sync_all()?;
            let parent = path
                .parent()
                .filter(|p| !p.as_os_str().is_empty())
                .unwrap_or_else(|| Path::new("."));
            File::open(parent)?.sync_all()?;
        }
        let mut recovered = RecoveredState::default();
        let mut expected_sequence = 0_u64;
        let mut previous_hash = GENESIS_HASH.to_owned();
        let mut reader = BufReader::new(file.try_clone()?);
        loop {
            let mut line = Vec::new();
            let read = Read::by_ref(&mut reader)
                .take(MAX_ENTRY_BYTES + 1)
                .read_until(b'\n', &mut line)?;
            if read == 0 {
                break;
            }
            if read as u64 > MAX_ENTRY_BYTES || line.pop() != Some(b'\n') || line.is_empty() {
                return Err(EventStoreError::IntegrityViolation(expected_sequence));
            }
            let entry: JournalEntry = serde_json::from_slice(&line)?;
            if line != canonical_json_bytes(&entry)? {
                return Err(EventStoreError::IntegrityViolation(expected_sequence));
            }
            entry.verify(expected_sequence, &previous_hash)?;
            recovered.apply(&entry.event)?;
            previous_hash = entry.event_hash;
            expected_sequence = expected_sequence
                .checked_add(1)
                .ok_or(EventStoreError::SequenceExhausted)?;
        }

        Ok((
            Self {
                path,
                writer: Mutex::new(JournalWriter {
                    file,
                    next_sequence: expected_sequence,
                    previous_hash,
                    state: recovered.clone(),
                    failed: false,
                }),
            },
            recovered,
        ))
    }

    /// Persist one complete state mutation before it becomes visible or is
    /// acknowledged by the API.
    pub fn append(&self, event: ApiJournalEvent) -> Result<(), EventStoreError> {
        let mut writer = self.writer.lock().map_err(|_| EventStoreError::Poisoned)?;
        if writer.failed {
            return Err(EventStoreError::Poisoned);
        }
        let mut next_state = writer.state.clone();
        next_state.apply(&event)?;
        let entry = JournalEntry::new(writer.next_sequence, writer.previous_hash.clone(), event)?;
        let mut encoded = canonical_json_bytes(&entry)?;
        encoded.push(b'\n');
        if encoded.len() as u64 > MAX_ENTRY_BYTES {
            return Err(EventStoreError::InvalidEvent(
                "journal entry exceeds byte limit".into(),
            ));
        }
        if let Err(error) = writer
            .file
            .write_all(&encoded)
            .and_then(|()| writer.file.sync_all())
        {
            // No subsequent acknowledgement is safe after a partial write or
            // uncertain fsync. Recovery must inspect the durable file first.
            writer.failed = true;
            return Err(EventStoreError::Io(error));
        }
        writer.next_sequence = writer
            .next_sequence
            .checked_add(1)
            .ok_or(EventStoreError::SequenceExhausted)?;
        writer.previous_hash = entry.event_hash;
        writer.state = next_state;
        Ok(())
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[derive(Debug, Error)]
pub(crate) enum EventStoreError {
    #[error("event journal path must not be empty")]
    InvalidPath,
    #[error("event journal I/O failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("event journal JSON failed: {0}")]
    Json(#[from] serde_json::Error),
    #[error("event journal canonicalization failed: {0}")]
    Canonicalization(#[from] xlemma_core::CanonicalizationError),
    #[error("event journal integrity violation at sequence {0}")]
    IntegrityViolation(u64),
    #[error("event journal contains duplicate record {0}")]
    DuplicateRecord(String),
    #[error("event journal update references missing record {0}")]
    MissingRecord(String),
    #[error("event journal contains an invalid state transition for {0}")]
    InvalidTransition(String),
    #[error("event journal sequence is exhausted")]
    SequenceExhausted,
    #[error("event journal writer lock was poisoned")]
    Poisoned,
    #[error("event journal contains invalid protocol data: {0}")]
    InvalidEvent(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use std::io::{Seek, SeekFrom};
    use xlemma_core::{ClaimId, PolicyId, ResearcherId, TheoryId};

    fn temp_path(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "xlemma-event-journal-{label}-{}-{}.jsonl",
            std::process::id(),
            Utc::now().timestamp_nanos_opt().unwrap_or_default()
        ))
    }

    fn job() -> VerificationJobRecord {
        let at = Utc.with_ymd_and_hms(2026, 9, 4, 20, 0, 0).unwrap();
        let observation: xlemma_core::ObservationReceipt = serde_json::from_str(include_str!(
            "../../../examples/no-arbitrage/observations.json"
        ))
        .map(|observations: Vec<xlemma_core::ObservationReceipt>| observations[0].clone())
        .unwrap();
        VerificationJobRecord {
            job_id: observation.job_id,
            researcher_id: ResearcherId::derive(&"journal-researcher").unwrap(),
            claim_id: ClaimId::from_canonical_elaborated_type(
                &TheoryId::derive(&"journal-theory").unwrap(),
                "forall p : Prop, p -> p",
            )
            .unwrap(),
            theory_id: TheoryId::derive(&"journal-theory").unwrap(),
            artifact_id: xlemma_core::ArtifactId::derive(&"journal-artifact").unwrap(),
            artifact_root: "blake3:artifact".into(),
            policy_id: PolicyId::derive(&"journal-policy").unwrap(),
            policy: xlemma_consensus::FormalConsensusPolicy {
                minimum_verified_users: 1,
                minimum_operators: 1,
                minimum_operator_clusters: 1,
                minimum_infrastructure_providers: 1,
                minimum_regions: 1,
                required_family_counts: BTreeMap::from([(
                    xlemma_core::CheckerFamily::LeanKernel,
                    1,
                )]),
                require_identical_artifact_root: true,
                require_identical_environment_root: true,
                require_identical_dependency_root: true,
                require_identical_axiom_set_root: true,
                challenge_period_seconds: 86_400,
            },
            committee_members: Vec::new(),
            maximum_budget_minor_units: 100,
            settlement_asset: "USD".into(),
            state: xlemma_core::VerificationState::ClaimCommitted,
            observations: Vec::new(),
            created_at: at,
            updated_at: at,
        }
    }

    #[test]
    fn restart_recovers_durable_job_history() {
        let path = temp_path("recovery");
        let (journal, recovered) = EventJournal::open(&path).unwrap();
        assert!(recovered.jobs.is_empty());
        let original = job();
        journal
            .append(ApiJournalEvent::VerificationJobCreated {
                job: original.clone(),
            })
            .unwrap();
        let mut updated = original.clone();
        let observation: xlemma_core::ObservationReceipt = serde_json::from_str(include_str!(
            "../../../examples/no-arbitrage/observations.json"
        ))
        .map(|observations: Vec<xlemma_core::ObservationReceipt>| observations[0].clone())
        .unwrap();
        updated.observations.push(observation);
        updated.updated_at += chrono::Duration::seconds(1);
        updated.state = xlemma_core::VerificationState::CheckersRevealed;
        journal
            .append(ApiJournalEvent::VerificationJobUpdated {
                job: updated.clone(),
                expected_previous_updated_at: original.updated_at,
            })
            .unwrap();
        drop(journal);

        let (_reopened, recovered) = EventJournal::open(&path).unwrap();
        assert_eq!(
            recovered.jobs.get(updated.job_id.as_str()).unwrap().state,
            updated.state
        );
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn modified_history_fails_closed() {
        let path = temp_path("tamper");
        let (journal, _) = EventJournal::open(&path).unwrap();
        journal
            .append(ApiJournalEvent::VerificationJobCreated { job: job() })
            .unwrap();
        drop(journal);

        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&path)
            .unwrap();
        file.seek(SeekFrom::Start(0)).unwrap();
        file.write_all(b"X").unwrap();
        file.sync_data().unwrap();
        assert!(EventJournal::open(&path).is_err());
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn out_of_order_job_update_fails_closed_on_replay() {
        let path = temp_path("transition");
        let (journal, _) = EventJournal::open(&path).unwrap();
        let original = job();
        journal
            .append(ApiJournalEvent::VerificationJobCreated {
                job: original.clone(),
            })
            .unwrap();
        let mut updated = original.clone();
        let observation: xlemma_core::ObservationReceipt = serde_json::from_str(include_str!(
            "../../../examples/no-arbitrage/observations.json"
        ))
        .map(|observations: Vec<xlemma_core::ObservationReceipt>| observations[0].clone())
        .unwrap();
        updated.observations.push(observation);
        updated.updated_at += chrono::Duration::seconds(1);
        let result = journal.append(ApiJournalEvent::VerificationJobUpdated {
            job: updated,
            expected_previous_updated_at: original.updated_at - chrono::Duration::seconds(1),
        });
        assert!(matches!(result, Err(EventStoreError::InvalidTransition(_))));
        drop(journal);

        assert!(EventJournal::open(&path).is_ok());
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn job_history_cannot_rewrite_immutable_terms() {
        let path = temp_path("immutable");
        let (journal, _) = EventJournal::open(&path).unwrap();
        let original = job();
        journal
            .append(ApiJournalEvent::VerificationJobCreated {
                job: original.clone(),
            })
            .unwrap();
        let mut rewritten = original.clone();
        let observation: xlemma_core::ObservationReceipt = serde_json::from_str(include_str!(
            "../../../examples/no-arbitrage/observations.json"
        ))
        .map(|observations: Vec<xlemma_core::ObservationReceipt>| observations[0].clone())
        .unwrap();
        rewritten.observations.push(observation);
        rewritten.settlement_asset = "OTHER".into();
        rewritten.state = xlemma_core::VerificationState::CheckersRevealed;
        rewritten.updated_at += chrono::Duration::seconds(1);
        assert!(matches!(
            journal.append(ApiJournalEvent::VerificationJobUpdated {
                job: rewritten,
                expected_previous_updated_at: original.updated_at,
            }),
            Err(EventStoreError::InvalidTransition(_))
        ));
        drop(journal);
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn incomplete_final_record_fails_closed() {
        let path = temp_path("missing-newline");
        let (journal, _) = EventJournal::open(&path).unwrap();
        journal
            .append(ApiJournalEvent::VerificationJobCreated { job: job() })
            .unwrap();
        drop(journal);
        let file = OpenOptions::new().write(true).open(&path).unwrap();
        file.set_len(file.metadata().unwrap().len() - 1).unwrap();
        assert!(EventJournal::open(&path).is_err());
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn io_failure_prevents_later_acknowledgements() {
        let path = temp_path("failed-write");
        let (journal, _) = EventJournal::open(&path).unwrap();
        // Inject a descriptor on which writes fail.
        journal.writer.lock().unwrap().file = File::open(&path).unwrap();
        assert!(matches!(
            journal.append(ApiJournalEvent::VerificationJobCreated { job: job() }),
            Err(EventStoreError::Io(_))
        ));
        assert!(matches!(
            journal.append(ApiJournalEvent::VerificationJobCreated { job: job() }),
            Err(EventStoreError::Poisoned)
        ));
        assert!(journal.writer.lock().unwrap().state.jobs.is_empty());
        drop(journal);
        std::fs::remove_file(path).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn concurrent_writer_and_symlink_journal_are_rejected() {
        let path = temp_path("locked");
        let (journal, _) = EventJournal::open(&path).unwrap();
        assert!(EventJournal::open(&path).is_err());
        drop(journal);
        let alias = temp_path("symlink");
        std::os::unix::fs::symlink(&path, &alias).unwrap();
        assert!(EventJournal::open(&alias).is_err());
        std::fs::remove_file(alias).unwrap();
        std::fs::remove_file(path).unwrap();
    }
}
