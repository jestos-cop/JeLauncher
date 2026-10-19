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
