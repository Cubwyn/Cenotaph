// src/main.rs
// Entry point for Cenotaph: The Great Omission.
//
// Runtime path:
//   main.rs -> app.rs -> core::engine::EngineState
//
// Validation path:
//   cargo run -- validate -> core::engine::validation

mod app;
mod core;
mod data;
mod game;
mod systems;

use winit::event_loop::{ControlFlow, EventLoop};

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

    // First non-validation argument is the level name. Use `continue` to resume
    // the autosave instead of opening the movement sandbox.
    let level_name = first_arg.unwrap_or_else(|| "movement_test".to_string());

    // Start the window/event loop and hand runtime work to App/EngineState.
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut application = app::App::new(level_name);

    event_loop
        .run_app(&mut application)
        .map_err(|e| format!("Event loop crashed: {}", e))?;

    Ok(())
}
