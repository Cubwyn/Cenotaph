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
mod developer;
mod game;
mod systems;

use developer::commands::{available_levels, parse_args, resolve_level_path, ProjectCommand};
use winit::event_loop::{ControlFlow, EventLoop};

fn main() {
    if let Err(error) = run() {
        eprintln!("[CENOTAPH] {}", error);
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let command = parse_args(std::env::args_os().skip(1)).map_err(invalid_input)?;
    match command {
        ProjectCommand::Help => {
            println!("{}", developer::commands::help_text());
            Ok(())
        }
        ProjectCommand::ListLevels => {
            let levels = available_levels(".").map_err(invalid_input)?;
            println!("Playable levels ({}):", levels.len());
            for level in levels {
                println!("- {}", level);
            }
            Ok(())
        }
        ProjectCommand::Validate => run_validation(),
        ProjectCommand::Doctor => run_doctor(),
        ProjectCommand::Editor => developer::editor_server::run(),
        ProjectCommand::Continue => run_game("continue".to_string()),
        ProjectCommand::Play { level_id } => {
            resolve_level_path(".", &level_id).map_err(invalid_input)?;
            run_game(level_id)
        }
    }
}

fn run_validation() -> Result<(), Box<dyn std::error::Error>> {
    let report = core::engine::validation::validate_project_content();
    if report.is_ok() {
        println!("{}", report);
        Ok(())
    } else {
        eprintln!("{}", report);
        Err(std::io::Error::new(std::io::ErrorKind::InvalidData, report.summary()).into())
    }
}

fn run_doctor() -> Result<(), Box<dyn std::error::Error>> {
    let report = developer::doctor::run_project_doctor();
    if report.is_ok() {
        println!("{}", report);
        Ok(())
    } else {
        eprintln!("{}", report);
        Err(std::io::Error::new(std::io::ErrorKind::InvalidData, report.summary()).into())
    }
}

fn run_game(level_name: String) -> Result<(), Box<dyn std::error::Error>> {
    // Start the window/event loop and hand runtime work to App/EngineState.
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut application = app::App::new(level_name);

    event_loop
        .run_app(&mut application)
        .map_err(|e| format!("Event loop crashed: {}", e))?;

    if let Some(error) = application.take_fatal_error() {
        return Err(std::io::Error::other(error).into());
    }

    Ok(())
}

fn invalid_input(message: String) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidInput, message)
}
