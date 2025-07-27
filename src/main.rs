use std::process;
use clap::Parser;

mod connection;
mod gui;

#[derive(Parser)]
struct CliArgs {
    /// Display windows only from active workspace
    #[arg(short, long)]
    workspace: bool
}

fn main() {
    let args = CliArgs::parse();

    /* Establish connection with the Niri instance */
    let connection = connection::Connection::new();

    let connection = match connection {
        Some(connection) => connection,
        None => {
            eprintln!("Failed to connect with Niri instance");
            process::exit(1);
        }
    };

    gui::start_gui(args, connection);

    process::exit(0)
}
