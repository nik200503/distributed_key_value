use std::fs;
use std::process::{Child, Command};
use std::thread;
use std::time::Duration;

fn start_server() -> Child {
    Command::new("cargo")
        .args(&["build", "--bin", "kvs-server"])
        .output()
        .expect("Failed to build server");

    Command::new("cargo")
        .args(&["run", "--bin", "kvs-server", "--", "--port", "4001"])
        .spawn()
        .expect("Failed to start server")
}

fn run_client(args: &[&str]) -> String {
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--quiet",
            "--bin",
            "kvs-client",
            "--",
            "--addr",
            "127.0.0.1:4001",
        ])
        .args(args)
        .output()
        .expect("Failed to run client");

    String::from_utf8(output.stdout).expect("invalid UTF-8")
}

#[test]
fn test_end_to_end() {
    let _ = fs::remove_file("kv.db");

    let mut server_process = start_server();

    thread::sleep(Duration::from_secs(1));

    let output = run_client(&["set", "key1", "value1"]);
    assert!(output.trim().is_empty());

    let output = run_client(&["get", "key1"]);
    assert_eq!(output.trim(), "value1");

    run_client(&["set", "key1", "value2"]);
    let output = run_client(&["get", "key1"]);
    assert_eq!(output.trim(), "value2");

    run_client(&["rm", "key1"]);
    let output = run_client(&["get", "key1"]);
    assert_eq!(output.trim(), "key not found");

    let _ = server_process.kill();
    let _ = fs::remove_file("kv.db");
}
