//! Example: Screenshot capture using the screenshot API
//!
//! Captures a full screenshot and a region screenshot, saving to disk.

fn main() {
    println!("Screenshot Example");
    println!("==================");

    // Full screen screenshot
    println!("Capturing full screen...");
    match usehid::screenshot() {
        Ok(png_bytes) => {
            let path = "screenshot_full.png";
            std::fs::write(path, &png_bytes).expect("Failed to write file");
            println!("Saved full screenshot to {} ({} bytes)", path, png_bytes.len());
        }
        Err(e) => {
            eprintln!("Screenshot failed: {}", e);
        }
    }

    // Region screenshot
    println!("\nCapturing region (100, 100, 400x300)...");
    match usehid::screenshot_region(100, 100, 400, 300) {
        Ok(png_bytes) => {
            let path = "screenshot_region.png";
            std::fs::write(path, &png_bytes).expect("Failed to write file");
            println!("Saved region screenshot to {} ({} bytes)", path, png_bytes.len());
        }
        Err(e) => {
            eprintln!("Region screenshot failed: {}", e);
        }
    }

    // Via Agent API
    println!("\nCapturing via Agent API...");
    let mut agent = usehid::AgentHID::new();
    let result = agent.execute_json(r#"{"action": "screenshot"}"#);
    if result.success {
        if let Some(data) = &result.data {
            println!("Agent screenshot returned base64 data ({} chars)", data.len());
        }
    } else {
        eprintln!("Agent screenshot failed: {:?}", result.error);
    }

    println!("\nDone!");
}
