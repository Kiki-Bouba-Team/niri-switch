/* niri-switch  Copyright (C) 2025  Kiki/Bouba Team */
use clap::Parser;

#[derive(Parser)]
#[command(version)]
struct CliArgs;

#[zbus::proxy(
    default_service = "org.kikibouba.NiriSwitchDaemon",
    default_path = "/org/kikibouba/NiriSwitchDaemon",
    interface = "org.kikibouba.NiriSwitchDaemon"
)]
trait NiriSwitchDaemon {
    fn activate(&self) -> zbus::Result<()>;
}

fn main() {
    let _args = CliArgs::parse();

    /* Connect to D-Bus session */
    let result = zbus::blocking::Connection::session();
    let connection = match result {
        Ok(connection) => connection,
        Err(error) => {
            eprintln!("Failed to establish connection with D-Bus: {error:?}");
            std::process::exit(1)
        }
    };

    /* Create proxy for daemon D-Bus interface */
    let result = NiriSwitchDaemonProxyBlocking::new(&connection);
    let proxy = match result {
        Ok(proxy) => proxy,
        Err(error) => {
            eprintln!("Failed to create interface proxy: {error:?}");
            std::process::exit(1)
        }
    };

    /* Call activate method on the daemon interface */
    let result = proxy.activate();
    match result {
        Ok(_) => (),
        Err(error) => {
            eprintln!("Failed to call 'Activate' method: {error:?}");
            std::process::exit(1)
        }
    }
}
