use std::net::ToSocketAddrs;
use std::time::Duration;
use log::{info, warn};

pub fn wait_for_internet() {
    info!("🔍 Verificando conexión real a Internet (DNS)...");
    loop {
        // Intentamos resolver discord.com. 
        // Hasta que el router no nos dé IP y el DNS no funcione, esto fallará.
        if "discord.com:443".to_socket_addrs().is_ok() {
            info!("✅ ¡Internet listo y verificado!");
            break;
        }
        warn!("⏳ Esperando a que el DNS responda...");
        std::thread::sleep(Duration::from_secs(2));
    }
}