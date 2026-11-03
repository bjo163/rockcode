#[test]
fn arch_boundary_allowlist() {
    use std::process::Command;
    let output = Command::new("cargo")
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .output()
        .expect("cargo metadata failed");
    assert!(output.status.success());
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json");
    let packages = v["packages"].as_array().expect("packages");
    // domain must have zero workspace dependencies (deps on other rockcode-* crates)
    for p in packages {
        let name = p["name"].as_str().unwrap_or("");
        if name == "rockcode-domain" {
            let deps = p["dependencies"]
                .as_array()
                .cloned()
                .unwrap_or_else(Vec::new);
            for d in deps {
                let dn = d["name"].as_str().unwrap_or("");
                if dn.starts_with("rockcode-") {
                    panic!("rockcode-domain must have zero workspace dependencies");
                }
            }
        }
    }
    // kernel must not depend on server/storage/provider/UI/tauri
    for p in packages {
        let name = p["name"].as_str().unwrap_or("");
        if name == "rockcode-kernel" {
            let deps = p["dependencies"]
                .as_array()
                .cloned()
                .unwrap_or_else(Vec::new);
            for d in deps {
                let dn = d["name"].as_str().unwrap_or("");
                assert!(dn != "rockcode-server", "kernel must not depend on server");
                assert!(
                    dn != "rockcode-storage",
                    "kernel must not depend on storage"
                );
                assert!(
                    dn != "rockcode-providers",
                    "kernel must not depend on providers"
                );
                assert!(dn != "tauri", "kernel must not depend on tauri");
            }
        }
    }
}
