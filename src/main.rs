use crate::game::*;
use crate::window::*;
use bytes::BufMut;
use std::thread::sleep;
use std::time::{Duration, Instant};
use windows::core::Result;
use windows::Win32::Media::timeBeginPeriod;

mod game;
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
    fn new() -> Self {
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
            },
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
    fn new() -> Self {
        Self {
            cursor_pos: V2 { x: 0, y: 0 },
            dt_for_frame: 0.0,
            keyboard: KeyboardInput::new(),
            mouse_buttons: [
                GameButtonState { ended_down: false },
                GameButtonState { ended_down: false },
            ],
        }
    }
}

fn main() -> Result<()> {
    let mut window = Window::new(1000, 700)?;
    let mut input = GameInput::new();
    let mut game_memory = GameMemory::new();
    let target_seconds_per_frame: f32 = 1.0 / window.refresh_rate as f32;

    // NOTE(Fermin): Set the Windows scheduler granularity to 1ms,
    // should this be in window.rs????
    unsafe {
        timeBeginPeriod(1);
    }

    //let process_start_instant = Instant::now();
    while window.window_running {
        let frame_start_instant = Instant::now();

        input.dt_for_frame = target_seconds_per_frame;

        update_and_render(&mut game_memory, &mut window.buffer, &input);
        window.win32_process_pending_messages(&mut input);

        let target_ms_per_frame = target_seconds_per_frame * 1000.0;
        if frame_start_instant.elapsed().as_millis() < target_ms_per_frame as u128 {
            let ms_until_next_frame: u64 = (target_ms_per_frame as u128
                - frame_start_instant.elapsed().as_millis())
            .try_into()
            .expect("Error calculating ms until next frame");
            sleep(Duration::from_millis(ms_until_next_frame));
        }

        // Debug logs
        //println!("Play time: {} seconds", process_start_instant.elapsed().as_secs());
        //println!("Monitor refresh rate: {}Hz", window.refresh_rate as f32);
        println!("{} ms/f", frame_start_instant.elapsed().as_millis());
    }

    Ok(())
}
