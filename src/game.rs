use crate::window::*;
use crate::*;

const WHITE: Color = Color{r: 255, g: 255, b: 255, a: 255};
const BYTES_PER_PIXEL: i32 = 4;
const ENTITY_Y_PADDING: i32 = 10;
const ENTITY_X_PADDING: i32 = 10;
const TILE_SIZE: i32 = 25;

pub struct GameMemory {
    v_entity: Entity,
    h_entity: Entity,
    is_initialized: bool,
}
impl GameMemory {
    pub fn new () -> Self {
        Self {
            v_entity: Entity::new(TILE_SIZE, TILE_SIZE*2),
            h_entity: Entity::new(TILE_SIZE*2, TILE_SIZE),
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

struct Entity {
    p: V2,
    dp: V2,
    color: Color,
    width: i32,
    height: i32,
}
impl Entity {
    fn new(width: i32, height: i32) -> Self {
        Self {
            p: V2{ x: 0, y: 0 },
            dp: V2{ x: 0, y: 0 },
            color: Color::new(120, 168, 82, 255),
            width,
            height,
        }
    }
}

fn draw_rectangle(pos: &V2, width: i32, height: i32, color: &Color, buffer: &mut Win32OffscreenBuffer) {
    let start_x: i32;
    let start_y: i32;

    if pos.x + width > buffer.width {
        start_x = buffer.width - width;
    } else if pos.x < 0 {
        start_x = 0;
    } else {
        start_x = pos.x;
    }

    if pos.y + height > buffer.height {
        start_y = buffer.height - height;
    } else if pos.y < 0 {
        start_y = 0;
    } else {
        start_y = pos.y;
    }

    let mut row: usize = (start_x * BYTES_PER_PIXEL + start_y * buffer.width * BYTES_PER_PIXEL) as usize;
    for _y in 0..height {
        for x in 0..width {
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
    buffer.bits.clear();
    // NOTE(FErmin): Clear to white
    // This is shit for performance, try to rerender only necessary pixels.
    for _y in 0..buffer.height {
        for _x in 0..buffer.width {
            buffer.bits.put_i32(WHITE.get_i32());
        }
    }

    if !memory.is_initialized {
        memory.v_entity.p.x = ENTITY_X_PADDING;
        memory.v_entity.p.y = ENTITY_Y_PADDING;
        memory.h_entity.p.x = ENTITY_X_PADDING;
        memory.h_entity.p.y = buffer.height - ENTITY_Y_PADDING - memory.h_entity.height;
        memory.is_initialized = true;
    }

    // TODO(Fermin): Use newtons eq of motion for better movement feeling
    if input.keyboard.buttons.move_up.ended_down {
        if memory.v_entity.p.y > ENTITY_Y_PADDING {
            memory.v_entity.p.y -= 1;
        }
    }
    if input.keyboard.buttons.move_down.ended_down {
        if memory.v_entity.p.y < buffer.height - ENTITY_Y_PADDING - memory.v_entity.height {
            memory.v_entity.p.y += 1;
        }
    }
    if input.keyboard.buttons.move_left.ended_down {
        if memory.h_entity.p.x > ENTITY_X_PADDING {
            memory.h_entity.p.x -= 1;
        }
    }
    if input.keyboard.buttons.move_right.ended_down {
        if memory.h_entity.p.x < buffer.width - ENTITY_X_PADDING - memory.h_entity.width {
            memory.h_entity.p.x += 1;
        }
    }

    draw_rectangle(
        &memory.v_entity.p,
        memory.v_entity.width,
        memory.v_entity.height,
        &memory.v_entity.color,
        buffer
    );
    draw_rectangle(
        &memory.h_entity.p,
        memory.h_entity.width,
        memory.h_entity.height,
        &memory.h_entity.color,
        buffer
    );

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
