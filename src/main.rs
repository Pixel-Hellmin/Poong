use windows::Win32::System::Performance::QueryPerformanceCounter;
mod handle;
mod window;

use windows::core::Result;
use window::Window;

fn win32_get_wallclock() -> i64 {
    let mut result: i64 = 0;
    unsafe {QueryPerformanceCounter(&mut result);}
    return result
}

fn main() -> Result<()> {
    let mut window = Window::new(1000, 700)?;
    let start_time: i64 = win32_get_wallclock();
    let mut play_time: i64;
    while window.window_running {
        window.win32_process_pending_messages();
        play_time = (win32_get_wallclock() - start_time) / 10000000;
        println!("Perf counter: {}", play_time);
    }
    Ok(())
}
