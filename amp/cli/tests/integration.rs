use anyhow::Result;
use amp_cli::{config::Config, client::AmpClient, session::Session};

#[tokio::test]
async fn test_session_lifecycle() -> Result<()> {
    // This test requires a running AMP server
    let config = Config::from_env()?;
    let client = AmpClient::new(&config.server_url);
    
    // Skip test if server is not available
    if !client.health_check().await.unwrap_or(false) {
        println!("Skipping integration test - AMP server not available");
        return Ok(());
    }
    
    // Create a test session
    let session = Session::new("echo 'test'".to_string(), "test-project".to_string());
    
    // Save session
    session.save_to_file(&config.session_dir).await?;
    
    // Load session back
    let loaded_session = Session::load_from_file(&config.session_dir, session.id).await?;
    assert_eq!(session.id, loaded_session.id);
    
    Ok(())
}
