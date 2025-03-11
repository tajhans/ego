mod session;
mod ui;

use anyhow::Result;
use chrono::Local;
use clap::{Parser, Subcommand};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use session::Session;
use std::thread;
use std::time::Duration;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Start {
        #[arg(value_name = "PROJECT_DIRECTORY")]
        project_directory: String,

        #[arg(short, long)]
        track_activity: bool,
    },
    End,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Start {
            project_directory,
            track_activity,
        } => {
            let session = Session::new(project_directory)?;
            session.save()?;
            println!("Session started in directory: {}", project_directory);
            println!("Initial line count: {}", session.initial_line_count);
            println!("Initial character count: {}", session.initial_char_count);

            if *track_activity {
                println!("Activity tracking enabled. Press Ctrl+C to end session.");

                thread::spawn(move || -> Result<()> {
                    crossterm::terminal::enable_raw_mode()?;

                    let mut last_save = std::time::Instant::now();

                    loop {
                        if event::poll(Duration::from_millis(100))? {
                            if let Event::Key(key) = event::read()? {
                                if key.kind == KeyEventKind::Press {
                                    if let Some(mut session) = Session::load()? {
                                        session.record_activity();

                                        if last_save.elapsed().as_secs() > 60 {
                                            session.save()?;
                                            last_save = std::time::Instant::now();
                                        }
                                    }

                                    if key.code == KeyCode::Char('c')
                                        && key
                                            .modifiers
                                            .contains(crossterm::event::KeyModifiers::CONTROL)
                                    {
                                        break;
                                    }
                                }
                            }
                        }
                    }

                    crossterm::terminal::disable_raw_mode()?;
                    Ok(())
                });
            }
        }
        Commands::End => {
            if let Some(mut session) = Session::load()? {
                let end_time = Local::now();

                session.end()?;

                ui::draw_stats(&session, end_time)?;
            } else {
                println!("No active session found.");
            }
        }
    }

    Ok(())
}
