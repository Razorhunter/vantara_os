use std::path::Path;
use std::fs;
use crate::modules::timezone::set_timezone_interactive;
use crate::modules::rootuser::add_root_user;
use vantara::show_boot_banner;

const DEFAULT_FIRSTBOOT_PATH: &str = "/etc/.firstboot";

pub fn setup_firstboot() {
    if Path::new(DEFAULT_FIRSTBOOT_PATH).exists() {
        show_boot_banner();

        set_timezone_interactive();
        println!();

        //TODO: keyboard layout

        //TODO: set hostname

        //TODO: network config

        //TODO: add user root
        add_root_user();

        let _ = fs::remove_file(DEFAULT_FIRSTBOOT_PATH);
    }
}
