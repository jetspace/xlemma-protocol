use std::{path::Path, process::Command};

#[test]
fn cli_prepares_wallet_vectors_and_rejects_expiry() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for kind in ["fund", "refund"] {
        let input = format!("examples/wallet/{kind}-intent.json");
        let args = [
            "wallet-preflight",
            "config/wallet-base-sepolia.json",
            "examples/wallet/trust.json",
            &input,
        ];
        let run = |time: &str| {
            Command::new(env!("CARGO_BIN_EXE_xlemma-cli"))
                .current_dir(&root)
                .args(args)
                .args(["--at", time])
                .output()
                .unwrap()
        };
        let output = run("1788998460");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let actual: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        let expected: serde_json::Value = serde_json::from_slice(
            &std::fs::read(root.join(format!("examples/wallet/{kind}-expected.json"))).unwrap(),
        )
        .unwrap();
        assert_eq!(actual, expected);
        let expired = run("1788998700");
        assert!(!expired.status.success());
        assert!(expired.stdout.is_empty());
    }
}
