#![windows_subsystem = "windows"]

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

// NOTE(Fermin): Do we need a V2 with generic types?
#[derive(Copy, Clone)]
struct V2 {
    x: f32,
    y: f32,
}
impl std::ops::Add<V2> for V2 {
    type Output = V2;

    fn add(self, a: V2) -> V2 {
        V2 {
            x: self.x + a.x,
            y: self.y + a.y,
        }
    }
}
impl std::ops::AddAssign<V2> for V2 {
    fn add_assign(&mut self, a: V2) {
        self.x += a.x;
        self.y += a.y;
    }
}
impl std::ops::Mul<f32> for V2 {
    type Output = V2;

    fn mul(self, factor: f32) -> V2 {
        V2 {
            x: self.x * factor,
            y: self.y * factor,
        }
    }
}
impl std::ops::MulAssign<f32> for V2 {
    fn mul_assign(&mut self, factor: f32) {
        self.x *= factor;
        self.y *= factor;
    }
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
    buttons: InputButtons,
}
impl KeyboardInput {
    fn new() -> Self {
        Self {
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
            cursor_pos: V2 { x: 0.0, y: 0.0 },
            dt_for_frame: 0.0,
            keyboard: KeyboardInput::new(),
            mouse_buttons: [
                GameButtonState { ended_down: false },
                GameButtonState { ended_down: false },
            ],
        }
    }
}

enum GameStates {
    Play,
    DeathScene,
}
pub struct GameState {
    state: GameStates,
}

fn main() -> Result<()> {
    let mut window = Window::new(435, 460)?;
    let mut input = GameInput::new();
    let mut game_memory = GameMemory::new();
    let mut game_state = GameState {
        state: GameStates::Play,
    };
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

        update_and_render(
            &mut game_memory,
            &mut window.buffer,
            &input,
            &mut game_state,
        );
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
        //println!("{} ms/f", frame_start_instant.elapsed().as_millis());
    }

    Ok(())
}
