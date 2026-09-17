use std::{
    io::Write,
    process::{Command, Output, Stdio},
};

fn run(args: &[&str], input: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_bi0cyph3r"))
        .args(args)
        .env_remove("BIOCYPHER_PASSWORD")
        .env_remove("BIOCYPHER_K1")
        .env_remove("BIOCYPHER_K2")
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
    let result = run(
        &[
            "plasmid",
            "Hi",
            "--name",
            "sample",
            "--structure",
            "fluorescent",
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
