slint::include_modules!();

use gcodekit4::{init_logging, VERSION};
use tracing::info;

fn main() -> anyhow::Result<()> {
    // Initialize logging
    init_logging()?;

    info!("═══════════════════════════════════════════════════════════");
    info!("GCodeKit4 v{}", VERSION);
    info!("Universal G-Code Sender for CNC Machines");
    info!("═══════════════════════════════════════════════════════════");

    let main_window = MainWindow::new().map_err(|e| anyhow::anyhow!("UI Error: {}", e))?;
    main_window.run().map_err(|e| anyhow::anyhow!("UI Runtime Error: {}", e))?;

    Ok(())
}
