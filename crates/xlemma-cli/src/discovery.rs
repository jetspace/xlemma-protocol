use anyhow::{bail, Context, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::{Duration, Utc};
use ed25519_dalek::{Signer, SigningKey};
use std::{fs::OpenOptions, io::Read, path::PathBuf};
use xlemma_core::{PolicyId, ReceiptId};
use xlemma_economics::{
    DiscoveryCommand, DiscoveryEnvelope, DiscoveryTrust, ResearchSubmission, ServiceRoundPolicy,
};

pub fn prepare(
    trust: PathBuf,
    policy: PathBuf,
    submission: Option<PathBuf>,
    salt: Option<String>,
) -> Result<()> {
    let trust: DiscoveryTrust = crate::read_json(trust)?;
    trust.validate()?;
    let policy: ServiceRoundPolicy = crate::read_json(policy)?;
    let round = policy.id()?;
    let mut output = serde_json::json!({"round_id":round,"policy_id":PolicyId::derive(&policy)?,"trust_root":trust.root()?});
    if let Some(path) = submission {
        let submission: ResearchSubmission = crate::read_json(path)?;
        output["submission_id"] = serde_json::to_value(submission.id(&round)?)?;
        output["commitment"] = serde_json::to_value(submission.commitment(
            &round,
            &salt.context("--salt is required with --submission")?,
        )?)?;
    }
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

pub fn sign(trust: PathBuf, command: PathBuf, key_file: PathBuf, nonce: String) -> Result<()> {
    let trust: DiscoveryTrust = crate::read_json(trust)?;
    trust.validate()?;
    let mut command: DiscoveryCommand = crate::read_json(command)?;
    if let DiscoveryCommand::ObserveFunding { receipt, .. } = &mut command {
        receipt.funding_receipt_id = receipt.derive_funding_receipt_id()?;
        receipt.validate_integrity()?;
    }
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW);
    }
    let file = options.open(key_file).context("opening signing seed")?;
    let metadata = file.metadata()?;
    if !metadata.is_file() || metadata.len() > 128 {
        bail!("seed must be a small regular file");
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o077 != 0 {
            bail!("seed file must not be accessible by group or others");
        }
    }
    let mut encoded = String::new();
    file.take(129).read_to_string(&mut encoded)?;
    let seed: [u8; 32] = hex::decode(encoded.trim())
        .context("seed must be hexadecimal")?
        .try_into()
        .map_err(|_| anyhow::anyhow!("seed must contain exactly 32 bytes"))?;
    let key = SigningKey::from_bytes(&seed);
    let now = Utc::now();
    let mut envelope = DiscoveryEnvelope {
        command_id: ReceiptId::derive(&"pending")?,
        trust_root: trust.root()?,
        nonce,
        issued_at: now,
        expires_at: now + Duration::minutes(5),
        signer: format!(
            "ed25519:{}",
            URL_SAFE_NO_PAD.encode(key.verifying_key().to_bytes())
        ),
        command,
        signature: String::new(),
    };
    envelope.command_id = envelope.expected_id()?;
    envelope.signature = format!(
        "ed25519:{}",
        URL_SAFE_NO_PAD.encode(key.sign(&envelope.signing_bytes()?).to_bytes())
    );
    envelope.authenticate(&trust, now)?;
    println!("{}", serde_json::to_string_pretty(&envelope)?);
    Ok(())
}

pub fn evidence_inputs(path: PathBuf) -> Result<()> {
    let envelope: xlemma_xlmp::XlmpEnvelope = crate::read_json(path)?;
    envelope.validate_integrity()?;
    let (claim, artifact, policy, roots) = match &envelope.message {
        xlemma_xlmp::XlmpMessage::Certificate(value) => {
            let c = &value.certificate;
            (
                c.claim_id.clone(),
                c.artifact_id.clone(),
                c.verification_policy_id.clone(),
                xlemma_economics::formal_discovery_roots(
                    &c.claim_id,
                    &c.proof_id,
                    &c.artifact_root,
                    &c.environment_root,
                    &c.dependency_root,
                    &c.axiom_set_root,
                )?,
            )
        }
        xlemma_xlmp::XlmpMessage::ResearchCertificate(value) => {
            if value.profile.class == xlemma_core::VerificationProfileClass::Formal {
                bail!("formal discovery requires exact PoIR evidence");
            }
            let roots = value
                .job
                .evidence_roots
                .iter()
                .map(|(kind, root)| {
                    Ok((
                        *kind,
                        root.parse()
                            .or_else(|_| xlemma_core::ArtifactId::derive(root))?,
                    ))
                })
                .collect::<Result<std::collections::BTreeMap<_, _>>>()?;
            (
                value.job.claim_id.clone(),
                value.job.artifact_id.clone(),
                value.profile.policy_id.clone(),
                roots,
            )
        }
        _ => bail!("expected a certificate envelope"),
    };
    println!(
        "{}",
        serde_json::to_string_pretty(
            &serde_json::json!({"claim_id":claim,"artifact_id":artifact,"verification_policy_id":policy,
        "evidence_roots":roots,"certificate_message_id":envelope.message_id,"requires_authenticated_xlmp_ingress":true})
        )?
    );
    Ok(())
}
