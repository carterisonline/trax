use std::io;

use lsp_server::Notification;
use trax_lsp_server::TraxLspServer;

fn main() {
    let mut server = TraxLspServer::new(|s| eprintln!("{s}"));

    let stdin = io::stdin();

    loop {
        let mut buf = String::new();
        stdin.read_line(&mut buf).unwrap();

        let notification: Notification = serde_json::from_str(&buf).unwrap();

        server.on_notification(&notification.method, notification.params);
    }
}
