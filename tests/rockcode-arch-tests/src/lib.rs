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
    // Keep network and UI transports out of domain and kernel layers.
    let forbidden_kernel_dependencies = [
        "rockcode-server",
        "rockcode-storage",
        "rockcode-providers",
        "reqwest",
        "axum",
        "tauri",
        "wry",
        "sqlx",
        "postgres",
        "tokio-postgres",
        "redis",
        "wasmtime",
    ];

    for p in packages {
        let name = p["name"].as_str().unwrap_or("");
        if name == "rockcode-kernel" {
            let deps = p["dependencies"]
                .as_array()
                .cloned()
                .unwrap_or_else(Vec::new);
            for d in deps {
                let dependency = d["name"].as_str().unwrap_or("");
                assert!(
                    !forbidden_kernel_dependencies.contains(&dependency),
                    "rockcode-kernel must not depend on {dependency}"
                );
            }
        }
    }

    // reqwest is permitted only at the provider transport and public SDK boundaries.
    let mut reqwest_owners: Vec<&str> = packages
        .iter()
        .filter_map(|p| {
            let dependencies = p["dependencies"].as_array()?;
            let uses_reqwest = dependencies
                .iter()
                .any(|dependency| dependency["name"].as_str() == Some("reqwest"));
            if uses_reqwest {
                p["name"].as_str()
            } else {
                None
            }
        })
        .collect();
    reqwest_owners.sort_unstable();
    assert_eq!(
        reqwest_owners,
        vec!["rockcode-providers", "rockcode-sdk"],
        "reqwest must stay at provider and SDK transport boundaries"
    );
}
