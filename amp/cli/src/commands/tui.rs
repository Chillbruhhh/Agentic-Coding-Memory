use crate::app::App;
use anyhow::Result;

pub async fn run_tui() -> Result<()> {
    println!("Launching AMP Bridge TUI...");
    
    let mut app = App::new().await?;
    app.run().await?;
    
    println!("TUI session ended");
    Ok(())
}
