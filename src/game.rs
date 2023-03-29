use crate::window::*;
use crate::*;

pub fn update_and_render(buffer: &mut Win32OffscreenBuffer, input: &GameInput) {
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

