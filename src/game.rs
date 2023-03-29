use crate::window::*;
use crate::*;

const WHITE: Color = Color{r: 255, g: 255, b: 255, a: 255};
const BYTES_PER_PIXEL: i32 = 4;


pub struct GameMemory {
    is_initialized: bool,
}
impl GameMemory {
    pub fn new () -> Self {
        Self {
            is_initialized: false,
        }
    }
}

struct Color {
    r: i32, g: i32, b: i32, a: i32,
}
impl Color {
    fn new(r: i32, g: i32, b: i32, a: i32) -> Self {
        Self { r, g, b, a, }
    }
    fn get_i32(&self) -> i32 {
        // NOTE(Fermin): Pixel -> BB GG RR AA
        let result: i32 = (self.b << 24) | (self.g << 16) | (self.r << 8) | self.a;
        result
    }
}

fn draw_square(pos: &V2, side_length: i32, buffer: &mut Win32OffscreenBuffer, color: &Color) {
    let start_x: i32;
    let start_y: i32;

    if pos.x + side_length > buffer.width {
        start_x = buffer.width - side_length;
    } else if pos.x < 0 {
        start_x = 0;
    } else {
        start_x = pos.x;
    }

    if pos.y + side_length > buffer.height {
        start_y = buffer.height - side_length;
    } else if pos.y < 0 {
        start_y = 0;
    } else {
        start_y = pos.y;
    }

    let mut row: usize = (start_x * BYTES_PER_PIXEL + start_y * buffer.width * BYTES_PER_PIXEL) as usize;
    for _y in 0..side_length {
        for x in 0..side_length {
            // NOTE(Fermin): Pixel -> BB GG RR AA
            buffer.bits[row + (x * BYTES_PER_PIXEL) as usize] = color.b as u8;
            buffer.bits[row + (x * BYTES_PER_PIXEL + 1) as usize] = color.g as u8;
            buffer.bits[row + (x * BYTES_PER_PIXEL + 2) as usize] = color.r as u8;
            buffer.bits[row + (x * BYTES_PER_PIXEL + 3) as usize] = color.a as u8;
        }
        row += (buffer.width * BYTES_PER_PIXEL) as usize;
    }
}

pub fn update_and_render(memory: &mut GameMemory, buffer: &mut Win32OffscreenBuffer, input: &GameInput) {
    let color = Color::new(50, 168, 82, 255);
    let square_side_length: i32 = 50;

    buffer.bits.clear();
    // NOTE(FErmin): Clear to white
    for _y in 0..buffer.height {
        for _x in 0..buffer.width {
            buffer.bits.put_i32(WHITE.get_i32());
        }
    }

    if !memory.is_initialized {
        draw_square(&V2{x:150, y:150}, square_side_length, buffer, &color);
        memory.is_initialized = true;
    }

    draw_square(&input.cursor_pos, square_side_length, buffer, &color);
    /*
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
    */
}

