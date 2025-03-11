mod session;
mod ui;

use anyhow::Result;
use chrono::Local;
use clap::{Parser, Subcommand};
use session::Session;

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
    },
    End,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Start { project_directory } => {
            let session = Session::new(project_directory)?;
            session.save()?;
            println!("Session started in directory: {}", project_directory);
            println!("Initial line count: {}", session.initial_line_count);
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
