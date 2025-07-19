use std::fs::OpenOptions;
use std::os::unix::io::AsRawFd;
use std::path::Path;
use chrono::{Datelike, Timelike, Local};
use libc::{ioctl, c_ulong};

#[repr(C)]
#[derive(Debug)]
struct RtcTime {
    tm_sec: i32,
    tm_min: i32,
    tm_hour: i32,
    tm_mday: i32,
    tm_mon: i32,   // 0-11
    tm_year: i32,  // sejak 1900
    tm_wday: i32,
    tm_yday: i32,
    tm_isdst: i32,
}

// Magic ioctl code untuk RTC_SET_TIME
const RTC_SET_TIME: c_ulong = 0x4024700a;

/// Sync masa sistem sekarang ke RTC hardware secara native (ioctl)
pub fn sync_system_time_to_rtc() -> Result<(), String> {
    let now = Local::now();

    let rtc_time = RtcTime {
        tm_sec: now.second() as i32,
        tm_min: now.minute() as i32,
        tm_hour: now.hour() as i32,
        tm_mday: now.day() as i32,
        tm_mon: (now.month() - 1) as i32,
        tm_year: (now.year() - 1900) as i32,
        tm_wday: now.weekday().num_days_from_sunday() as i32,
        tm_yday: now.ordinal() as i32 - 1,
        tm_isdst: -1,
    };

    let rtc_path = if Path::new("/dev/rtc").exists() {
        "/dev/rtc"
    } else if Path::new("/dev/rtc0").exists() {
        "/dev/rtc0"
    } else {
        return Err("Tiada RTC device (/dev/rtc atau /dev/rtc0)".to_string());
    };

    let file = OpenOptions::new()
        .write(true)
        .open(rtc_path)
        .map_err(|e| format!("Gagal buka {}: {}", rtc_path, e))?;

    let fd = file.as_raw_fd();

    let request: i32 = RTC_SET_TIME as i32;
    let result = unsafe { ioctl(fd, request, &rtc_time) };

    if result != 0 {
        return Err(format!("Gagal set RTC: errno {}", result));
    }

    Ok(())
}
