#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(target_os = "windows")]
extern "system" {
    fn MessageBoxW(
        hWnd: isize,
        lpText: *const u16,
        lpCaption: *const u16,
        uType: u32,
    ) -> i32;
}

fn main() {
    // On release builds, capture panics to disk and show an error dialog
    #[cfg(not(debug_assertions))]
    {
        let prev = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            let payload = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
                s.clone()
            } else {
                format!("{:?}", panic_info.payload())
            };
            let location = panic_info
                .location()
                .map(|l| l.to_string())
                .unwrap_or_else(|| "?".to_string());
            let ts = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);

            let log_msg = format!(
                "[{}] PANIC at {}:\n{}",
                ts, location, payload
            );

            let _ = std::fs::write(
                std::env::temp_dir().join("tomato-panic.log"),
                &log_msg,
            );

            #[cfg(target_os = "windows")]
            unsafe {
                let log_path = std::env::temp_dir()
                    .join("tomato-panic.log")
                    .to_string_lossy()
                    .to_string();
                let wide_msg: Vec<u16> = format!(
                    "Tomato 遇到错误并需要关闭。\n\n错误信息:\n{}\n\n位置: {}\n\n日志已保存到:\n{}",
                    payload, location, log_path
                )
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();
                let wide_title: Vec<u16> = "Tomato 错误"
                    .encode_utf16()
                    .chain(std::iter::once(0))
                    .collect();
                MessageBoxW(0, wide_msg.as_ptr(), wide_title.as_ptr(), 0x00000010 | 0x00000000);
            }

            prev(panic_info);
        }));
    }

    tomato_lib::run()
}
