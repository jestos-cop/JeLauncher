//! Runs against the Java installs of the current machine; run with `cargo test -- --ignored --nocapture`.

use jelauncher_core::java::detect_java_installs;

#[tokio::test]
#[ignore = "depends on locally installed Java"]
async fn finds_local_java() {
    let installs = detect_java_installs(std::path::Path::new("/nonexistent")).await;
    for install in &installs {
        println!(
            "{} {} {} {}",
            install.major, install.arch, install.vendor, install.path
        );
    }
    assert!(!installs.is_empty());
}

#[tokio::test]
#[ignore = "downloads a Java runtime"]
async fn installs_mojang_java_runtime() -> jelauncher_core::Result<()> {
    use jelauncher_core::java::{check_java, install_runtime};
    use jelauncher_core::net::HttpClient;
    use jelauncher_core::settings::NetworkSettings;

    let dir = tempfile::tempdir().map_err(|err| jelauncher_core::Error::Io {
        path: "tempdir".into(),
        source: err,
    })?;
    let http = HttpClient::new(&NetworkSettings::default())?;
    let java = install_runtime(&http, "java-runtime-delta", dir.path(), 16, |_| {}).await?;
    let install = check_java(&java).await;
    println!("{install:?}");
    assert_eq!(install.map(|i| i.major), Some(21));

    let detected = detect_java_installs(dir.path()).await;
    assert!(
        detected.iter().any(|i| i.path == java.to_string_lossy()),
        "managed runtime not detected"
    );
    Ok(())
}
