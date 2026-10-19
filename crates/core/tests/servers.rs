//! Pings a real public server; run with `cargo test --test servers -- --ignored --nocapture`.

use jelauncher_core::servers::ping_server;

#[tokio::test]
#[ignore = "requires network access"]
async fn pings_a_public_server() -> jelauncher_core::Result<()> {
    let status = ping_server("mc.hypixel.net").await?;
    println!("{status:?}");
    assert!(status.players_max > 0);
    Ok(())
}
