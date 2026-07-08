// src/main.rs
// Entry point for Cenotaph: The Great Omission.
//
// Cenotaph is a first-person exploration game built with Rust, wgpu, and rapier3d.
// This module serves as the application entry point and orchestrates the main event loop.
//
// Module Architecture:
// ┌─────────────────────────────────────────────────────────────────┐
// │                        Core Systems                             │
// ├─────────────────────────────────────────────────────────────────┤
// │ config  │ input   │ world   │ gameplay │ physics │ render       │
// │ Settings│ Events  │ Levels  │ Systems  │ Physics │ Graphics     │
// ├─────────────────────────────────────────────────────────────────┤
// │ engine  │ audio   │ save    │ editor*  │ app     │ main         │
// │ Core    │ Sound   │ Save    │ Editor   │ Loop    │ Entry        │
// └─────────────────────────────────────────────────────────────────┘
// * Editor module only compiled with --features editor
//
// Key Dependencies:
// - main.rs → app.rs → engine/state.rs (core engine)
// - engine/state.rs → all subsystem modules
// - render/renderer.rs → rendering pipeline
// - gameplay/* → player systems and mechanics

mod app;
mod core;
mod data;
mod game;
mod systems;

use winit::event_loop::{ControlFlow, EventLoop};

/// Application entry point
///
/// Initializes the event loop and starts the main application.
/// The event loop handles window events, input, and frame updates.
///
/// # Returns
/// - `Ok(())` on successful application exit
/// - `Err(Box<dyn Error>)` if the event loop fails to start or crashes
///
/// # Error Handling
/// This function uses proper error handling instead of panicking, allowing
/// the application to gracefully handle startup failures and provide
/// meaningful error messages to users and developers.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let first_arg = args.next();

    if matches!(
        first_arg.as_deref(),
        Some("validate" | "validate-content" | "--validate")
    ) {
        let report = core::engine::validation::validate_project_content();
        if report.is_ok() {
            println!("{}", report);
            return Ok(());
        }

        eprintln!("{}", report);
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, report.summary()).into());
    }

    // Parse optional level name from CLI arguments
    let level_name = first_arg.unwrap_or_else(|| "ashwalk_01".to_string());

    // Create the main event loop that will handle all window and input events
    // This is the core of our application - without it, nothing happens
    let event_loop = EventLoop::new()?;

    // Set control flow to poll for events continuously
    // This ensures smooth, responsive gameplay by processing events as they occur
    event_loop.set_control_flow(ControlFlow::Poll);

    // Initialize the application with all necessary systems and state
    // This sets up the engine, renderer, input handlers, and game state
    let mut application = app::App::new(level_name);

    // Run the application event loop
    // This will handle all window events, input, and rendering until the application exits
    // The event loop is the heartbeat of our game - it processes user input, updates game state,
    // and renders frames at the target frame rate
    event_loop
        .run_app(&mut application)
        .map_err(|e| format!("Event loop crashed: {}", e))?;

    // Application completed successfully
    Ok(())
}
