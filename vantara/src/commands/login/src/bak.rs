// const DEFAULT_GROUP_FILE: &str = "/etc/group";

// #[derive(Debug)]
// struct GroupEntry {
//     groupname: String,
//     gid: u32,
//     members: Vec<String>
// }

// if !Path::new(DEFAULT_PASSWD_FILE).exists() || !Path::new(DEFAULT_SHADOW_FILE).exists() {
//     safe_println(format_args!("It seems this is a new machine and it needs an administrator. Setting one now"));
//     add_root_user();
// }

// fn hash_password_with_salt(salt: &str, password: &str) -> String {
//     let mut hasher = Sha512::new();
//     hasher.update(salt.as_bytes());
//     hasher.update(password.as_bytes());
//     let result = hasher.finalize();
//     hex::encode(result)
// }

// fn parse_passwd_line(line: &str) -> Option<PasswdEntry> {
//     let parts: Vec<&str> = line.split(':').collect();
//     if parts.len() < 7 {
//         return None;
//     }

//     Some(PasswdEntry {
//         username: parts[0].to_string(),
//         uid: parts[2].parse().ok()?,
//         gid: parts[3].parse().ok()?,
//         fullname: parts[4].to_string(),
//         home: parts[5].to_string(),
//         shell: parts[6].to_string()
//     })
// }

// fn load_passwd(path: &str) -> std::io::Result<Vec<PasswdEntry>> {
//     use std::fs::File;
//     use std::io::{BufRead, BufReader};

//     let file = File::open(path)?;
//     let reader = BufReader::new(file);
//     let mut entries = Vec::new();

//     for line in reader.lines() {
//         if let Ok(line) = line {
//             if let Some(entry) = parse_passwd_line(&line) {
//                 entries.push(entry);
//             }
//         }
//     }

//     Ok(entries)
// }

// fn parse_shadow_line(line: &str) -> Option<ShadowEntry> {
//     let parts: Vec<&str> = line.split(':').collect();
//     if parts.len() < 2 {
//         return None;
//     }

//     let username = parts[0];
//     let hash_field = parts[1];

//     if !hash_field.starts_with('$') {
//         return None;
//     }

//     let hash_parts: Vec<&str> = hash_field.split('$').collect();
//     if hash_parts.len() < 4 {
//         return None;
//     }

//     Some(ShadowEntry {
//         username: username.to_string(),
//         algo_id: hash_parts[1].to_string(),
//         salt: hash_parts[2].to_string(),
//         hash: hash_parts[3].to_string(),
//     })
// }

// fn load_shadow(path: &str) -> std::io::Result<Vec<ShadowEntry>> {
//     use std::fs::File;
//     use std::io::{BufRead, BufReader};

//     let file = File::open(path)?;
//     let reader = BufReader::new(file);
//     let mut entries = Vec::new();

//     for line in reader.lines() {
//         if let Ok(line) = line {
//             if let Some(entry) = parse_shadow_line(&line) {
//                 entries.push(entry);
//             }
//         }
//     }

//     Ok(entries)
// }

// fn parse_group_line(line: &str) -> Option<GroupEntry> {
//     let parts: Vec<&str> = line.split(':').collect();
//     if parts.len() < 4 {
//         return None;
//     }

//     let members = if parts[3].trim().is_empty() {
//         vec![]
//     } else {
//         parts[3].trim().split(',').map(|s| s.to_string()).collect()
//     };

//     Some(GroupEntry {
//         groupname: parts[0].to_string(),
//         gid: parts[2].parse().ok()?,
//         members,
//     })
// }

// fn load_group(path: &str) -> std::io::Result<Vec<GroupEntry>> {
//     use std::fs::File;
//     use std::io::{BufRead, BufReader};

//     let file = File::open(path)?;
//     let reader = BufReader::new(file);
//     let mut entries = Vec::new();

//     for line in reader.lines() {
//         if let Ok(line) = line {
//             if let Some(entry) = parse_group_line(&line) {
//                 entries.push(entry);
//             }
//         }
//     }

//     Ok(entries)
// }

// fn generate_salt(len: usize) -> String {
//     rand::rng()
//     .sample_iter(&Alphanumeric)
//     .take(len)
//     .map(char::from)
//     .collect()
// }

// fn add_user_to_passwd_file(username: &str, fullname: &str, uid: u32, gid: u32, passwd_path: &str) -> std::io::Result<()> {
//     let home_dir = format!("/root");
//     let shell = "/bin/shell";

//     let entry = format!(
//         "{}:x:{}:{}:{}:{}:{}\n",
//         username, uid, gid, fullname, home_dir, shell
//     );

//     let mut file = OpenOptions::new()
//         .append(true)
//         .create(true)
//         .open(passwd_path)?;

//     writeln!(file, "{}", entry)?;

//     std::fs::create_dir_all(home_dir)?;

//     Ok(())
// }

// fn add_user_to_shadow_file(username: &str, password: &str, shadow_path: &str) -> std::io::Result<()> {
//     let salt = generate_salt(16);
//     let hash = hash_password_with_salt(&salt, password);

//     let entry = format!(
//         "{}:$6${}${}:::::::\n",
//         username,
//         salt,
//         hash
//     );

//     let mut file = OpenOptions::new()
//         .append(true)
//         .create(true)
//         .open(shadow_path)?;

//     writeln!(file, "{}", entry)?;

//     Ok(())
// }

// fn add_user_to_group_file(username: &str, gid: u32, group_path: &str) -> std::io::Result<()> {
//     let entry = format!(
//         "{}:x:{}:\n",
//         username, gid
//     );

//     let mut file = OpenOptions::new()
//         .append(true)
//         .create(true)
//         .open(group_path)?;

//     writeln!(file, "{}", entry)?;

//     Ok(())
// }

// fn check_login(username: &str, password: &str, passwd_list: &[PasswdEntry], shadow_list: &[ShadowEntry]) -> bool {
//     let user = passwd_list.iter().find(|u| u.username == username);
//     if user.is_none() {
//         safe_print(format_args!("User not found. "));
//         return false;
//     }

//     let shadow = shadow_list.iter().find(|p| p.username == username);

//     if shadow.is_none() {
//         safe_print(format_args!("User entry not found. "));
//         return false;
//     }

//     let entry = shadow.unwrap();

//     if entry.algo_id != "6" {
//         safe_print(format_args!("Unsupported algorithm. "));
//         return false;
//     }

//     let input_hash = hash_password_with_salt(&entry.salt, password);

//     if input_hash == entry.hash {
//         safe_println(format_args!("Login successfully."));
//         std::env::set_var("HOME", &user.unwrap().home);
//         std::env::set_var("USER", username);
//         std::env::set_current_dir(&user.unwrap().home).unwrap_or_else(|_| {
//             safe_eprintln(format_args!("Failed to set home dir to {}", &user.unwrap().home));
//         });
//         true
//     } else {
//         safe_println(format_args!("Invalid login. "));
//         false
//     }
// }

// fn add_root_user() {
//     safe_print(format_args!("Set Username (default: root): "));
//     io::stdout().flush().unwrap(); // Ensure the prompt is printed immediately
//     let mut username = String::new();
//     io::stdin().read_line(&mut username).unwrap(); 
//     let username = if username.trim().is_empty() { "root".into() } else { username.trim().to_string() };

//     safe_print(format_args!("Set {} Password: ", &username));
//     io::stdout().flush().unwrap(); // Ensure the prompt is printed immediately
//     let password = read_password();

//     let uid = 0;
//     let gid = 0;

//     let _ = add_user_to_group_file(&username, gid, DEFAULT_GROUP_FILE);
//     let _ = add_user_to_passwd_file(&username, "Administrator", uid, gid, DEFAULT_PASSWD_FILE);
//     let _ = add_user_to_shadow_file(&username, &password, DEFAULT_SHADOW_FILE);

//     safe_println(format_args!("{} account had been successfuly created. Proceed to login now", username));
//     println!();
// }