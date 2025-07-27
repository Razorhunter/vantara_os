use vantara::{safe_print, safe_println, read_password};
use std::io::{self, Write};

pub fn add_root_user() {
    safe_print(format_args!("Set Username (default: root): "));
    io::stdout().flush().unwrap(); // Ensure the prompt is printed immediately
    let mut username = String::new();
    io::stdin().read_line(&mut username).unwrap(); 
    let username = if username.trim().is_empty() { "root".into() } else { username.trim().to_string() };

    safe_print(format_args!("Set {} Password: ", &username));
    io::stdout().flush().unwrap(); // Ensure the prompt is printed immediately
    let password = read_password();

    let uid = 0;
    let gid = 0;

    let _ = vantarauth::modules::group::add_user_to_group_file(&username, gid);
    let _ = vantarauth::modules::passwd::add_user_to_passwd_file(&username, "Administrator", uid, gid);
    let _ = vantarauth::modules::shadow::add_user_to_shadow_file(&username, &password);

    safe_println(format_args!("{} account had been successfuly created. Proceed to login now", username));
    println!();
}