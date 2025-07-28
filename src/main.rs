/* niri-switch  Copyright (C) 2025  Kiki/Bouba Team */
use clap::Parser;
use nix::fcntl::{Flock, FlockArg};
use std::{fs::File, process};

mod connection;
mod gui;

#[derive(Parser)]
struct CliArgs {
    /// Display windows only from active workspace
    #[arg(short, long)]
    workspace: bool,
}

fn main() {
    let args = CliArgs::parse();

    /* Prevent multiple instances from running with file lock */
    let lock = match acquire_lock_file() {
        Some(lock) => lock,
        /* TODO: log this failure in verbose mode */
        None => process::exit(0)
    };

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

    /* Locks are released on drop, but just in case check for errors */
    match lock.unlock() {
        Ok(_) => (),
        Err(error) => {
            eprintln!("Failed to release the lock file: {error:?}");
            process::exit(1);
        }
    }

    process::exit(0)
}

/// Acquire system-wide application lock
fn acquire_lock_file() -> Option<Flock<File>> {
    let mut lock_path = std::env::temp_dir();
    lock_path.push("niri_switch.lock");

    let result = File::create(lock_path);

    let file = match result {
        Ok(file) => file,
        Err(error) => {
            eprintln!("Failed to create lock file: {error:?}");
            return None;
        }
    };

    /* niri-switch is unix-only, so we can use flock directly */
    Flock::lock(file, FlockArg::LockExclusiveNonblock).ok()
}
