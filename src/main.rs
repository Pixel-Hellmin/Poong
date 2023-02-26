use windows::Win32::System::Performance::QueryPerformanceCounter;
use bytes::BufMut;

use crate::window::Win32OffscreenBuffer;
mod handle;
mod window;

use windows::core::Result;
use window::Window;

struct V2 {
    x: i32,
    y: i32,
}

struct GameInput {
    cursor_coords: V2,
    dt_for_frame: f32,
}
impl Default for GameInput {
    fn default () -> GameInput {
        GameInput {
            cursor_coords: V2 { x: 0, y: 0 },
            dt_for_frame: 0.0,
        }
    }
}

fn win32_get_wallclock() -> i64 {
    let mut result: i64 = 0;
    unsafe {QueryPerformanceCounter(&mut result);}
    return result
}

fn render_gradient(buffer: &mut Win32OffscreenBuffer) {
    static mut WAVE: f32 = 0.001;
    static mut WAVE_DELTA: f32 = 0.001;

    let mut r: i32;
    let mut g: i32;
    let mut b: i32;

    buffer.bits.clear();
    let pixels_in_buffer: i32 = buffer.width * buffer.height;
    for pixel in 0..pixels_in_buffer {

        let gradient_in_x: f32 = ((pixel % buffer.width) as f32 / buffer.width as f32) * 255.0;
        let gradient_in_y: f32 = ((pixel / buffer.height) as f32 / buffer.height as f32) * 255.0;
        
        unsafe {
            r = (255.0 * WAVE + gradient_in_x + gradient_in_y) as i32;
            g = (255.0 - 255.0 * WAVE + gradient_in_y) as i32;
            b = (75.0 + 75.0 * WAVE + gradient_in_y - gradient_in_x) as i32;
        }

        // NOTE(Fermin): Pixel -> BB GG RR AA
        let color: i32 = (b << 24) | (g << 16) | (r << 8) | 255;
        buffer.bits.put_i32(color);
    }

    unsafe { 
        if WAVE >= 0.99 {
            WAVE_DELTA = -0.001;
        }
        if WAVE <= 0.01 {
            WAVE_DELTA = 0.001;
        }
        WAVE += WAVE_DELTA;
    }
    
}

fn main() -> Result<()> {
    let mut window = Window::new(1000, 700)?;
    let mut input = GameInput::default();
    let mut _play_time: i64;

    let start_time: i64 = win32_get_wallclock();
    while window.window_running {
        render_gradient(&mut window.buffer);
        window.win32_process_pending_messages();
        _play_time = (win32_get_wallclock() - start_time) / 10000000;
        //println!("Perf counter: {}", play_time);
    }
    Ok(())
}
