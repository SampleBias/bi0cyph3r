use std::{
    fs,
    io::Write,
    path::PathBuf,
    process::{Command, Output, Stdio},
};

fn run(args: &[&str], input: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_bi0cyph3r"))
        .args(args)
        .env_remove("BIOCYPHER_PASSWORD")
        .env_remove("BIOCYPHER_K1")
        .env_remove("BIOCYPHER_K2")
        .env_remove("BIOCYPHER_THEME")
        // Launch the absolute Cargo binary with no external tools on PATH.
        .env("PATH", "")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

#[test]
fn piped_utf8_roundtrip_keeps_stdout_clean() {
    let message = "Hello 🧬\nCafé\n";
    let encoded = run(&["encode", "-"], message);
    assert!(encoded.status.success());
    assert!(encoded.stdout.iter().all(|b| b"ATCG\n".contains(b)));
    let decoded = run(
        &["decode", "-"],
        std::str::from_utf8(&encoded.stdout).unwrap(),
    );
    assert!(decoded.status.success());
    assert_eq!(decoded.stdout, message.as_bytes());
}

#[test]
fn invalid_arguments_fail_and_non_tty_does_not_hang() {
    for args in [
        vec!["encode", "hello", "--mode", "typo"],
        vec!["decode", "ATNG"],
        vec!["encode", "secret", "--mode", "secure"],
        vec!["encode", "secret", "--mode", "splitkey"],
        vec![],
    ] {
        let output = run(&args, "");
        assert!(!output.status.success(), "{args:?}");
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
    }
    assert!(run(&["--help"], "").status.success());
}

#[test]
fn plasmid_fasta_can_be_decoded_by_cli() {
    for structure in ["payload", "marked", "fluorescent"] {
        let result = run(
            &[
                "plasmid",
                "Hi",
                "--name",
                "sample",
                "--structure",
                structure,
            ],
            "",
        );
        assert!(result.status.success());
        let fasta = std::str::from_utf8(&result.stdout).unwrap();
        assert!(fasta.starts_with(">sample mode=basic\n"));
        let decoded = run(&["decode", "-"], fasta);
        assert!(decoded.status.success());
        assert_eq!(decoded.stdout, b"Hi");
    }
}

struct TestDir(PathBuf);

impl TestDir {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "biocypher-cli-test-{}-{}",
            std::process::id(),
            rand::random::<u64>()
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn all_modes_roundtrip_via_files_without_external_tools() {
    let temp = TestDir::new();
    let input_path = temp.0.join("message.txt");
    let message = "Hello 🧬\nCafé\t世界\n";
    fs::write(&input_path, message).unwrap();

    for mode in ["basic", "nanopore", "secure", "splitkey"] {
        let sequence_path = temp.0.join(format!("{mode}.fasta"));
        let keys_path = temp.0.join(format!("{mode}-keys.json"));
        let mut encode_args = vec![
            "encode",
            "--input",
            input_path.to_str().unwrap(),
            "--mode",
            mode,
            "--password",
            "regression-test-password",
            "--output",
            "fasta",
            "--save",
            sequence_path.to_str().unwrap(),
        ];
        if mode == "splitkey" {
            encode_args.extend(["--keys-output", keys_path.to_str().unwrap()]);
        }
        let encoded = run(&encode_args, "");
        assert_success(&encoded);
        assert!(encoded.stdout.is_empty());
        assert!(fs::read_to_string(&sequence_path).unwrap().starts_with('>'));

        let keys: serde_json::Value = if mode == "splitkey" {
            serde_json::from_slice(&fs::read(&keys_path).unwrap()).unwrap()
        } else {
            serde_json::Value::Null
        };
        let mut decode_args = vec![
            "decode",
            "--input",
            sequence_path.to_str().unwrap(),
            "--mode",
            mode,
            "--password",
            "regression-test-password",
        ];
        if mode == "splitkey" {
            decode_args.extend([
                "--k1",
                keys["k1_base64"].as_str().unwrap(),
                "--k2",
                keys["k2_base64"].as_str().unwrap(),
            ]);
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                assert_eq!(
                    fs::metadata(&keys_path).unwrap().permissions().mode() & 0o777,
                    0o600
                );
            }
        }
        let decoded = run(&decode_args, "");
        assert_success(&decoded);
        assert_eq!(decoded.stdout, message.as_bytes(), "{mode}");
    }
}

#[test]
fn safety_json_works_without_external_tools() {
    let output = run(&["safety", "-", "--output", "json"], ">sample\natcg atcg\n");
    assert_success(&output);
    let result: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["operation"], "Safety");
    assert_eq!(result["sequence"], "ATCGATCG");
    assert_eq!(result["stats"]["length"], 8);
    assert_eq!(result["stats"]["gc_content"], 50.0);
    assert!(result["report"].is_object());
}

#[test]
fn cli_exports_never_overwrite_existing_files() {
    let temp = TestDir::new();
    let output_path = temp.0.join("existing.dna");
    fs::write(&output_path, "original").unwrap();
    let output = run(
        &["encode", "Hello", "--save", output_path.to_str().unwrap()],
        "",
    );
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert_eq!(fs::read_to_string(&output_path).unwrap(), "original");
}

#[test]
fn theme_flag_is_validated_without_changing_scriptable_output() {
    for theme in ["original", "crimson", "paper", "monochrome", "amber"] {
        let result = run(&["--theme", theme, "encode", "Hi"], "");
        assert!(result.status.success());
        assert_eq!(result.stdout, b"TACATCCT\n");
    }
    assert!(!run(&["--theme", "invalid", "encode", "Hi"], "")
        .status
        .success());
}
