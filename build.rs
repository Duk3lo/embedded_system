fn main() {
    if let Ok(it) = dotenvy::dotenv_iter() {
        for item in it {
            let (key, value) = item.expect("Error leyendo variable del .env");
            println!("cargo:rustc-env={}={}", key, value);
        }
    }
    embuild::espidf::sysenv::output();
}