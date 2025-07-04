use boxdrop_sync_daemon::{Result, ConfigManager, DropboxClient, SyncEngine};
use tracing::{info, error};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    info!("Starting Dropbox Sync Daemon");
    
    // Load configuration
    let config = ConfigManager::load()?;
    info!("Configuration loaded successfully");
    
    // Initialize Dropbox client
    let client = DropboxClient::new(&config.dropbox_token)?;
    info!("Dropbox client initialized");
    
    // Initialize sync engine
    let sync_engine = SyncEngine::new(client, config)?;
    info!("Sync engine initialized");
    
    // Start the sync daemon
    if let Err(e) = sync_engine.run().await {
        error!("Sync engine failed: {}", e);
        return Err(e.into());
    }
    
    Ok(())
} 