/* niri-switch  Copyright (C) 2025  Kiki/Bouba Team */
use async_channel::Sender;

const DBUS_DAEMON_ID: &str = "org.kikibouba.NiriSwitchDaemon";
const DBUS_DAEMON_PATH: &str = "/org/kikibouba/NiriSwitchDaemon";

pub enum DbusEvent {
    Activate,
}

struct NiriSwitchDaemonInterface {
    /// Channel used for communication with GTK
    gtk_channel: Sender<DbusEvent>,
}

#[zbus::interface(name = "org.kikibouba.NiriSwitchDaemon")]
impl NiriSwitchDaemonInterface {
    /// Method called when niri-switch client is run
    async fn activate(&self) {
        self.gtk_channel
            .send(DbusEvent::Activate)
            .await
            .expect("Sending message should succeed");
    }
}

/// Start D-Bus service that handles connection with client
pub async fn server_loop(gtk_channel: Sender<DbusEvent>) -> Result<(), zbus::Error> {
    let interface = NiriSwitchDaemonInterface { gtk_channel };
    let _connection = zbus::connection::Builder::session()?
        .name(DBUS_DAEMON_ID)?
        .serve_at(DBUS_DAEMON_PATH, interface)?
        .build()
        .await?;

    loop {
        /* Don't have to do anything here, dbus handles the requests
         * in the background */
        std::future::pending::<()>().await;
    }
}
