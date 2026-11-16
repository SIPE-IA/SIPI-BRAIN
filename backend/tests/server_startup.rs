use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::PathBuf,
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

struct Server(Child);
impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn fixture() -> PathBuf {
    let root = tempfile::Builder::new()
        .prefix("sipibrain-startup-")
        .tempdir()
        .unwrap()
        .keep();
    eprintln!("scratch: {}", root.display());
    std::fs::create_dir(root.join("assets")).unwrap();
    std::fs::write(root.join("index.html"), "<div id=app>startup fixture</div>").unwrap();
    std::fs::write(root.join("assets/app.js"), "console.log('fixture')").unwrap();
    root
}

fn http(address: &str, path: &str) -> String {
    let mut stream = TcpStream::connect(address).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(3)))
        .unwrap();
    write!(
        stream,
        "GET {path} HTTP/1.1\r\nHost: {address}\r\nConnection: close\r\n\r\n"
    )
    .unwrap();
    let mut output = String::new();
    stream.read_to_string(&mut output).unwrap();
    output
}

#[test]
fn serve_starts_listener_serves_bundle_and_keeps_unknown_api_json() {
    let root = fixture();
    let reserved = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = reserved.local_addr().unwrap().to_string();
    drop(reserved);
    let child = Command::new(env!("CARGO_BIN_EXE_sipibrain"))
        .args(["serve", "--bind", &address, "--static-dir"])
        .arg(&root)
        .env("SIPIBRAIN_DATA_DIR", root.join("data"))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let mut server = Server(child);
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if TcpStream::connect(&address).is_ok() {
            break;
        }
        assert!(
            server.0.try_wait().unwrap().is_none(),
            "serve exited before opening its loopback listener"
        );
        assert!(
            Instant::now() < deadline,
            "serve did not open a loopback listener"
        );
        std::thread::sleep(Duration::from_millis(30));
    }
    let index = http(&address, "/");
    assert!(index.starts_with("HTTP/1.1 200"));
    assert!(index.contains("startup fixture"));
    let asset = http(&address, "/assets/app.js");
    assert!(asset.starts_with("HTTP/1.1 200"));
    assert!(asset.contains("console.log('fixture')"));
    let unknown = http(&address, "/api/unknown");
    assert!(unknown.starts_with("HTTP/1.1 404"));
    assert!(unknown.contains("application/json"));
    assert!(!unknown.contains("startup fixture"));
    let traversal = http(&address, "/../Cargo.toml");
    assert!(!traversal.contains("[package]"));
}

#[test]
fn serve_invalid_bundle_directory_fails_without_creating_data() {
    let root = fixture();
    let result = Command::new(env!("CARGO_BIN_EXE_sipibrain"))
        .args(["serve", "--bind", "127.0.0.1:0", "--static-dir"])
        .arg(root.join("absent"))
        .env("SIPIBRAIN_DATA_DIR", root.join("data"))
        .output()
        .unwrap();
    assert!(
        !result.status.success(),
        "invalid static-dir must exit unsuccessfully"
    );
    assert!(!root.join("data").exists());
}

#[test]
fn cli_initializes_once_and_refuses_password_flags_and_non_loopback() {
    let root = fixture();
    for _ in 0..2 {
        let output = Command::new(env!("CARGO_BIN_EXE_sipibrain"))
            .arg("init-db")
            .env("SIPIBRAIN_DATA_DIR", root.join("data"))
            .output()
            .unwrap();
        assert!(output.status.success());
        assert!(String::from_utf8_lossy(&output.stdout).contains("Database initialized."));
    }
    assert!(root.join("data/sipibrain.db").is_file());
    let output = Command::new(env!("CARGO_BIN_EXE_sipibrain"))
        .args([
            "create-user",
            "--username",
            "example",
            "--password",
            "fake-test-value",
        ])
        .env("SIPIBRAIN_DATA_DIR", root.join("data"))
        .output()
        .unwrap();
    assert!(!output.status.success());
    let output = Command::new(env!("CARGO_BIN_EXE_sipibrain"))
        .args(["serve", "--bind", "0.0.0.0:8080", "--static-dir"])
        .arg(&root)
        .env("SIPIBRAIN_DATA_DIR", root.join("refused-data"))
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(!root.join("refused-data").exists());
}
