use std::env;

fn main() {
    match env::var("USER").or_else(|_| env::var("USERNAME")) {
        Ok(user) => println!("{}", user),
        Err(_) => eprintln!("Tak dapat kenal pasti nama pengguna."),
    }
}