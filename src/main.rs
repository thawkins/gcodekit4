use gcodekit4::core::Controller;
use gcodekit4::data::Position;
use gcodekit4::firmware::FirmwareCapabilities;
use gcodekit4::gcode::GcodeParser;
use gcodekit4::{init_logging, VERSION};
use tracing::{debug, info};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    init_logging()?;

    info!("═══════════════════════════════════════════════════════════");
    info!("GCodeKit4 v{}", VERSION);
    info!("Universal G-Code Sender for CNC Machines");
    info!("═══════════════════════════════════════════════════════════");

    // Initialize core components
    debug!("Initializing core components...");

    let controller = Controller::new();
    debug!("✓ Controller initialized: {:?}", controller.get_state());

    let mut parser = GcodeParser::new();
    debug!("✓ G-Code parser initialized");

    // Display firmware capabilities
    debug!("Loading firmware capabilities...");
    let grbl_caps = FirmwareCapabilities::grbl();
    info!("GRBL Capabilities:");
    info!("  • Max Axes: {}", grbl_caps.max_axes);
    info!("  • Max Feed Rate: {} mm/min", grbl_caps.max_feed_rate);
    info!("  • Max Rapid Rate: {} mm/min", grbl_caps.max_rapid_rate);
    info!("  • Supports Probing: {}", grbl_caps.supports_probing);
    info!("  • Buffer Size: {}", grbl_caps.buffer_size);

    // Test G-Code parsing
    debug!("Testing G-Code parser...");
    match parser.parse("G00 X10.5 Y20.3 Z5.0 ; Rapid move") {
        Ok(cmd) => info!("✓ Parsed G-Code: {}", cmd.command),
        Err(e) => info!("✗ Parse error: {}", e),
    }

    // Test position calculations
    let pos1 = Position::new(0.0, 0.0, 0.0);
    let pos2 = Position::new(3.0, 4.0, 0.0);
    let distance = pos1.distance_to(&pos2);
    info!("Distance calculation test:");
    info!("  From: {}", pos1);
    info!("  To: {}", pos2);
    info!("  Distance: {:.3}", distance);

    info!("═══════════════════════════════════════════════════════════");
    info!("Initialization complete. Ready for development.");
    info!("═══════════════════════════════════════════════════════════");

    Ok(())
}
