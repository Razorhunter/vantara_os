use std::fs::File;
use std::io::{Read};
use std::mem::size_of;

#[repr(C)]
#[derive(Debug)]
struct UtmpEntry {
    ut_type: i16,                // 2 byte
    ut_pid: i32,                 // 4 byte
    ut_line: [u8; 32],           // 32 byte
    ut_id: [u8; 4],              // 4 byte
    ut_user: [u8; 32],           // 32 byte
    ut_host: [u8; 256],          // 256 byte
    ut_exit: [u8; 4],            // exit_status struct
    ut_session: i32,             // 4 byte
    ut_tv_sec: i32,              // 4 byte
    ut_tv_usec: i32,             // 4 byte
    ut_addr_v6: [i32; 4],        // 16 byte
    unused: [u8; 20],            // reserved
}

impl UtmpEntry {
    fn username(&self) -> String {
        String::from_utf8_lossy(&self.ut_user)
            .trim_matches(char::from(0))
            .to_string()
    }

    fn tty(&self) -> String {
        String::from_utf8_lossy(&self.ut_line)
            .trim_matches(char::from(0))
            .to_string()
    }

    fn host(&self) -> String {
        String::from_utf8_lossy(&self.ut_host)
            .trim_matches(char::from(0))
            .to_string()
    }
}

fn main() {
    let path = "/var/run/utmp"; // atau "/run/utmp"
    let mut file = File::open(path).expect("Gagal buka utmp");

    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).expect("Gagal baca fail");

    let entry_size = size_of::<UtmpEntry>();
    let entries = buffer.len() / entry_size;

    for i in 0..entries {
        let offset = i * entry_size;
        let entry_slice = &buffer[offset..offset + entry_size];

        let entry: UtmpEntry = unsafe {
            std::ptr::read_unaligned(entry_slice.as_ptr() as *const UtmpEntry)
        };

        // Hanya USER_PROCESS = 7
        if entry.ut_type == 7 {
            println!(
                "{}\t{}\t{}",
                entry.username(),
                entry.tty(),
                entry.host()
            );
        }
    }
}
