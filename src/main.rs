mod handle;
mod window;

use windows::core::Result;
use window::Window;

fn main() -> Result<()> {
    let mut window = Window::new(1000, 700)?;
    while window.window_running {
        window.win32_process_pending_messages();
        //println!("asdads");
    }
    Ok(())
}
