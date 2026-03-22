mod app;
mod display;
mod layout;
#[allow(dead_code)]
mod process;
#[allow(dead_code)]
mod scanner;
#[allow(dead_code)]
mod session;
mod tui;
mod watch;
mod widgets;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "sigwatch", about = "Live game state inspector")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Attach {
        #[arg(group = "target")]
        pid: Option<u32>,

        #[arg(long, group = "target")]
        name: Option<String>,

        #[arg(long, short)]
        session: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Attach {
            pid,
            name,
            session: session_file,
        } => {
            let target_pid = match (pid, name) {
                (Some(p), _) => p,
                (_, Some(ref n)) => process::find_pid_by_name(n)?,
                _ => anyhow::bail!("provide a PID or --name"),
            };

            let proc = process::attach(target_pid)?;
            let saved = session_file
                .as_deref()
                .and_then(|path| session::load(path).ok());

            let mut app = app::App::new(proc, saved);
            tui::run(&mut app)
        }
    }
}
