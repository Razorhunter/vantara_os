use log::info;

fn main() {
    env_logger::init();

    // Biasanya di sini setup backend, seat, dan compositor
    // Tapi kita letak placeholder dulu
    println!("(Placeholder) Wayland compositor running...");
    loop {
        std::thread::sleep(std::time::Duration::from_secs(60));
    }
}
