use windows::Win32::System::Performance::QueryPerformanceCounter;
use bytes::BufMut;

use crate::window::Win32OffscreenBuffer;
mod handle;
mod window;

use windows::core::Result;
use window::Window;

fn win32_get_wallclock() -> i64 {
    let mut result: i64 = 0;
    unsafe {QueryPerformanceCounter(&mut result);}
    return result
}

fn render_gradient(buffer: &mut Win32OffscreenBuffer) {
    let r: i32 = 120;
    let g: i32 = 200;
    let b: i32 = 150;
    let pixels_in_buffer: i32 = buffer.width * buffer.height;

    buffer.bits.clear();
    for _ in 0..pixels_in_buffer {
        // NOTE(Fermin): Pixel -> BB GG RR AA
        let color: i32 = (b << 24) | (g << 16) | (r << 8) | 255;
        buffer.bits.put_i32(color);
    }
}

fn main() -> Result<()> {
    let mut window = Window::new(1000, 700)?;
    let mut play_time: i64;

    let start_time: i64 = win32_get_wallclock();
    while window.window_running {
        render_gradient(&mut window.buffer);
        window.win32_process_pending_messages();
        play_time = (win32_get_wallclock() - start_time) / 10000000;
        println!("Perf counter: {}", play_time);
    }
    Ok(())
}
