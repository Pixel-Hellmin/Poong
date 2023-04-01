use bytes::BytesMut;
use windows::Win32::System::Performance::QueryPerformanceCounter;
use windows::core::Result;
use bytes::BufMut;
use crate::window::*;
use crate::game::*;

mod handle;
mod window;
mod game;


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

fn main() -> Result<()> {
    let mut window = Window::new(1000, 700)?;
    let mut input = GameInput::new();
    let mut _play_time: i64;
    let mut game_memory = GameMemory::new();

    let start_time: i64 = win32_get_wallclock();
    while window.window_running {
        update_and_render(&mut game_memory, &mut window.buffer, &input);
        window.win32_process_pending_messages(&mut input);
        _play_time = (win32_get_wallclock() - start_time) / 10000000;
        //println!("Perf counter: {}", play_time);
    }
    Ok(())
}
