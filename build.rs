fn main() {
    // Lee el archivo .env si existe
    if let Ok(it) = dotenvy::dotenv_iter() {
        for item in it {
            let (key, value) = item.expect("Error leyendo variable del .env");
            // Esto le dice a Cargo que cree una variable de entorno para el compilador
            println!("cargo:rustc-env={}={}", key, value);
        }
    }
    
    // Configuración necesaria para ESP-IDF
    embuild::espidf::sysenv::output();
}