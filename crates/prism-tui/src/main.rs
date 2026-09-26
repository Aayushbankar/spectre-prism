use color_eyre::Result;
use prism_tui::app::App;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let mut app = App::new();
    app.run().await?;
    Ok(())
}
