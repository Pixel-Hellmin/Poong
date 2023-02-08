mod handle;
mod window;

use windows::core::Result;
use window::Window;

fn main() -> Result<()> {
    // NOTE(Fermin): Havent figured out how to exit the program by clicking
    // the X on the window
    let mut window = Window::new(1000, 700)?;
    loop {
        window.win32_process_pending_messages();
        unsafe {
            // TODO(Fermin): See if we can handle exitting in a "safe" way
            if !window::WINDOW_RUNNING {
                break;
            }
        }
    }
    Ok(())
}
