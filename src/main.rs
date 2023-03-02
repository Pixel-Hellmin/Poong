use windows::Win32::System::Performance::QueryPerformanceCounter;
use windows::core::Result;
use bytes::BufMut;
use crate::window::*;

mod handle;
mod window;


struct V2 {
    x: i32,
    y: i32,
}

struct GameButtonState {
    // TODO(Fermin): Half transitions
    ended_down: bool,
}
struct InputButtons {
    move_up: GameButtonState,
    move_down: GameButtonState,
    move_left: GameButtonState,
    move_right: GameButtonState,
    back: GameButtonState,
    start: GameButtonState,
    jump: GameButtonState,
}
struct KeyboardInput {
    is_connected: bool,
    buttons: InputButtons,
}
impl KeyboardInput {
    fn new () -> Self {
        Self {
            is_connected: false,
            buttons: InputButtons {
                move_up: GameButtonState { ended_down: false },
                move_down: GameButtonState { ended_down: false },
                move_left: GameButtonState { ended_down: false },
                move_right: GameButtonState { ended_down: false },
                back: GameButtonState { ended_down: false },
                start: GameButtonState { ended_down: false },
                jump: GameButtonState { ended_down: false },
            }
        }
    }
}

pub struct GameInput {
    // TODO(Fermin): Controller\Keyboard support
    cursor_pos: V2,
    dt_for_frame: f32,
    keyboard: KeyboardInput,
    mouse_buttons: [GameButtonState; 2],
}
impl GameInput {
    fn new () -> Self {
        Self {
            cursor_pos: V2 { x: 0, y: 0 },
            dt_for_frame: 0.0,
            keyboard: KeyboardInput::new(),
            mouse_buttons: [
                GameButtonState { ended_down: false },
                GameButtonState { ended_down: false }
            ],
        }
    }
}

fn win32_get_wallclock() -> i64 {
    let mut result: i64 = 0;
    unsafe {QueryPerformanceCounter(&mut result);}
    return result
}

fn render_gradient(buffer: &mut Win32OffscreenBuffer, input: &GameInput) {
    static mut WAVE: f32 = 0.001;
    static mut WAVE_DELTA: f32 = 0.001;

    let mut r: i32;
    let mut g: i32;
    let mut b: i32;

    buffer.bits.clear();
    let pixels_in_buffer: i32 = buffer.width * buffer.height;
    for pixel in 0..pixels_in_buffer {

        let gradient_in_x: f32 = if input.mouse_buttons[0].ended_down {
            0.0
        } else if input.keyboard.buttons.move_up.ended_down {
            50.0
        } else if input.keyboard.buttons.move_down.ended_down {
            100.0
        } else if input.keyboard.buttons.move_left.ended_down {
            150.0
        } else if input.keyboard.buttons.move_right.ended_down {
            200.0
        } else {
            ((pixel % buffer.width) as f32 / buffer.width as f32) * 255.0
        };

        let gradient_in_y: f32 = if input.mouse_buttons[1].ended_down {
            0.0
        } else if input.keyboard.buttons.start.ended_down {
            50.0
        } else if input.keyboard.buttons.back.ended_down {
            100.0
        } else if input.keyboard.buttons.jump.ended_down {
            150.0
        } else {
            ((pixel / buffer.height) as f32 / buffer.height as f32) * 255.0
        };
        
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
    let mut input = GameInput::new();
    let mut _play_time: i64;

    let start_time: i64 = win32_get_wallclock();
    while window.window_running {
        render_gradient(&mut window.buffer, &input);
        window.win32_process_pending_messages(&mut input);
        _play_time = (win32_get_wallclock() - start_time) / 10000000;
        //println!("Perf counter: {}", play_time);
    }
    Ok(())
}
