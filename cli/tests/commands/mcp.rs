// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::io::Write;
use std::process::{Command, Stdio};
use tempfile::tempdir;

fn gitehr() -> Command {
    Command::new(env!("CARGO_BIN_EXE_gitehr"))
}

#[test]
fn mcp_resources_read_works_with_explicit_repo_path() {
    let dir = tempdir().unwrap();
    std::fs::create_dir(dir.path().join(".gitehr")).unwrap();
    std::fs::create_dir(dir.path().join("journal")).unwrap();
    std::fs::create_dir(dir.path().join("state")).unwrap();
    std::fs::write(dir.path().join(".gitehr/GITEHR_VERSION"), "0.2.1\n").unwrap();
    std::fs::write(
        dir.path().join("journal/20260627T000000.000Z-test.md"),
        "# Test\n",
    )
    .unwrap();
    std::fs::write(dir.path().join("state/problems.json"), "[]\n").unwrap();

    let mut child = gitehr()
        .args([
            "mcp",
            "serve",
            "--stdio",
            "--repo-path",
            dir.path().to_str().unwrap(),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    {
        let stdin = child.stdin.as_mut().unwrap();
        writeln!(
            stdin,
            r#"{{"jsonrpc":"2.0","id":1,"method":"initialize","params":{{"protocolVersion":"2024-11-05","capabilities":{{}},"clientInfo":{{"name":"test-client","version":"1.0.0"}}}}}}"#
        )
        .unwrap();
        writeln!(
            stdin,
            r#"{{"jsonrpc":"2.0","method":"notifications/initialized"}}"#
        )
        .unwrap();
        writeln!(
            stdin,
            r#"{{"jsonrpc":"2.0","id":2,"method":"resources/list","params":{{}}}}"#
        )
        .unwrap();
        writeln!(
            stdin,
            r#"{{"jsonrpc":"2.0","id":3,"method":"resources/read","params":{{"uri":"gitehr://repo/journal"}}}}"#
        )
        .unwrap();
        writeln!(
            stdin,
            r#"{{"jsonrpc":"2.0","id":4,"method":"resources/read","params":{{"uri":"gitehr://repo/status"}}}}"#
        )
        .unwrap();
    }

    let out = child.wait_with_output().unwrap();
    assert!(out.status.success());

    let responses: Vec<serde_json::Value> = String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();

    assert_eq!(responses.len(), 4);
    assert_eq!(
        responses[1]["result"]["resources"][0]["uri"],
        "gitehr://repo/journal"
    );
    assert_eq!(
        responses[1]["result"]["resources"][0]["mimeType"],
        "application/json"
    );
    assert_eq!(
        responses[2]["result"]["contents"][0]["uri"],
        "gitehr://repo/journal"
    );
    assert_eq!(
        responses[2]["result"]["contents"][0]["mimeType"],
        "application/json"
    );
    assert!(
        responses[2]["result"]["contents"][0]["text"]
            .as_str()
            .unwrap()
            .contains("20260627T000000.000Z-test.md")
    );
    assert_eq!(
        responses[3]["result"]["contents"][0]["uri"],
        "gitehr://repo/status"
    );
    assert!(
        responses
            .iter()
            .all(|response| response.get("error").is_none())
    );
}

#[test]
fn mcp_prompts_follow_lifecycle_validate_arguments_and_redact_logs() {
    let dir = tempdir().unwrap();
    std::fs::create_dir(dir.path().join(".gitehr")).unwrap();
    let sensitive_value = "private-clinical-sentinel";

    let mut child = gitehr()
        .args([
            "mcp",
            "serve",
            "--stdio",
            "--repo-path",
            dir.path().to_str().unwrap(),
        ])
        .env("RUST_LOG", "debug")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    {
        let stdin = child.stdin.as_mut().unwrap();
        writeln!(
            stdin,
            r#"{{"jsonrpc":"2.0","id":1,"method":"initialize","params":{{"protocolVersion":"2024-11-05","capabilities":{{}},"clientInfo":{{"name":"test-client","version":"1.0.0"}}}}}}"#
        )
        .unwrap();
        writeln!(
            stdin,
            r#"{{"jsonrpc":"2.0","method":"notifications/initialized"}}"#
        )
        .unwrap();
        writeln!(
            stdin,
            r#"{{"jsonrpc":"2.0","id":2,"method":"prompts/list","params":{{}}}}"#
        )
        .unwrap();
        writeln!(
            stdin,
            r#"{{"jsonrpc":"2.0","id":3,"method":"prompts/get","params":{{"name":"soap_note","arguments":{{"chief_complaint":"{sensitive_value}"}}}}}}"#
        )
        .unwrap();
        writeln!(
            stdin,
            r#"{{"jsonrpc":"2.0","id":4,"method":"prompts/get","params":{{"name":"medication_review","arguments":{{"focus":42}}}}}}"#
        )
        .unwrap();
        writeln!(
            stdin,
            r#"{{"jsonrpc":"2.0","id":5,"method":"prompts/get","params":{{"name":"{sensitive_value}"}}}}"#
        )
        .unwrap();
        writeln!(
            stdin,
            r#"{{"jsonrpc":"2.0","id":6,"method":"{sensitive_value}"}}"#
        )
        .unwrap();
        writeln!(
            stdin,
            r#"{{"jsonrpc":"2.0","id":null,"method":"prompts/list","params":{{}}}}"#
        )
        .unwrap();
        writeln!(stdin, r#""{sensitive_value}""#).unwrap();
    }

    let out = child.wait_with_output().unwrap();
    assert!(out.status.success());

    let responses: Vec<serde_json::Value> = String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(
        responses.len(),
        8,
        "notifications must not receive responses"
    );
    assert_eq!(
        responses[1]["result"]["prompts"].as_array().unwrap().len(),
        5
    );
    assert_eq!(responses[2]["result"]["messages"][0]["role"], "user");
    assert!(
        responses[2]["result"]["messages"][0]["content"]["text"]
            .as_str()
            .unwrap()
            .contains(sensitive_value)
    );
    assert_eq!(responses[3]["error"]["code"], -32602);
    assert_eq!(responses[4]["error"]["code"], -32602);
    assert_eq!(responses[5]["error"]["code"], -32601);
    assert!(responses[6]["id"].is_null());
    assert_eq!(responses[6]["error"]["code"], -32600);
    assert!(responses[7]["id"].is_null());
    assert_eq!(responses[7]["error"]["code"], -32600);
    assert!(
        !String::from_utf8_lossy(&out.stderr).contains(sensitive_value),
        "clinical prompt values must not be copied into stderr logs"
    );
}

#[test]
fn mcp_serve_initialize_responds_on_stdout_only() {
    let dir = tempdir().unwrap();
    std::fs::create_dir(dir.path().join(".gitehr")).unwrap();

    let mut child = gitehr()
        .args([
            "mcp",
            "serve",
            "--stdio",
            "--repo-path",
            dir.path().to_str().unwrap(),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    {
        let stdin = child.stdin.as_mut().unwrap();
        writeln!(
            stdin,
            r#"{{"jsonrpc":"2.0","id":1,"method":"initialize","params":{{"protocolVersion":"2024-11-05","capabilities":{{}},"clientInfo":{{"name":"test-client","version":"1.0.0"}}}}}}"#
        )
        .unwrap();
    }

    let out = child.wait_with_output().unwrap();
    assert!(out.status.success());

    let stdout = String::from_utf8_lossy(&out.stdout);
    let response: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert_eq!(response["result"]["serverInfo"]["name"], "gitehr");
    assert_eq!(response["result"]["protocolVersion"], "2024-11-05");
    assert!(
        !stdout.contains("INFO"),
        "MCP stdout must contain protocol JSON only; got {stdout:?}"
    );

    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("Starting MCP server on stdio"),
        "tracing should go to stderr; got {stderr:?}"
    );
}

#[test]
fn mcp_serve_refuses_non_repository_path() {
    let dir = tempdir().unwrap();

    let out = gitehr()
        .args([
            "mcp",
            "serve",
            "--stdio",
            "--repo-path",
            dir.path().to_str().unwrap(),
        ])
        .output()
        .unwrap();

    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("not a GitEHR repository"),
        "expected a clear refusal; got {stderr:?}"
    );
}

#[test]
fn mcp_serve_refuses_encrypted_repository() {
    let dir = tempdir().unwrap();
    std::fs::create_dir(dir.path().join(".gitehr")).unwrap();
    std::fs::write(dir.path().join(".gitehr/ENCRYPTED"), "").unwrap();

    let out = gitehr()
        .args([
            "mcp",
            "serve",
            "--stdio",
            "--repo-path",
            dir.path().to_str().unwrap(),
        ])
        .output()
        .unwrap();

    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("marked as encrypted"),
        "expected a clear refusal; got {stderr:?}"
    );
}
