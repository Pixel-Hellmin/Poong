use crate::window::*;
use crate::*;

const WHITE: Color = Color{r: 255, g: 255, b: 255, a: 255};
const BYTES_PER_PIXEL: i32 = 4;
const ENTITY_Y_PADDING: i32 = 10;
const ENTITY_X_PADDING: i32 = 10;
const TILE_SIZE: i32 = 25;

pub struct GameMemory {
    l_entity: Entity,
    r_entity: Entity,
    b_entity: Entity,
    t_entity: Entity,
    ball: Entity,
    is_initialized: bool,
}
impl GameMemory {
    pub fn new () -> Self {
        Self {
            l_entity: Entity::new(TILE_SIZE, TILE_SIZE*2, Color::new(120, 130, 170, 255)),
            r_entity: Entity::new(TILE_SIZE, TILE_SIZE*2, Color::new(120, 130, 170, 255)),
            b_entity: Entity::new(TILE_SIZE*2, TILE_SIZE, Color::new(120, 130, 170, 255)),
            t_entity: Entity::new(TILE_SIZE*2, TILE_SIZE, Color::new(120, 130, 170, 255)),
            ball: Entity::new(TILE_SIZE, TILE_SIZE, Color::new(220, 30, 70, 255)),
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
    ddp: V2,
    color: Color,
    width: i32,
    height: i32,
}
impl Entity {
    fn new(width: i32, height: i32, color: Color) -> Self {
        Self {
            p: V2{ x: 0, y: 0 },
            dp: V2{ x: 0, y: 0 },
            ddp: V2{ x: 0, y: 0 },
            color,
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
        memory.l_entity.p.x = ENTITY_X_PADDING;
        memory.l_entity.p.y = ENTITY_Y_PADDING;

        memory.r_entity.p.x = buffer.width - ENTITY_X_PADDING - memory.r_entity.width;
        memory.r_entity.p.y = ENTITY_Y_PADDING;

        memory.b_entity.p.x = ENTITY_X_PADDING;
        memory.b_entity.p.y = buffer.height - ENTITY_Y_PADDING - memory.b_entity.height;

        memory.t_entity.p.x = ENTITY_X_PADDING;
        memory.t_entity.p.y = ENTITY_Y_PADDING;

        memory.ball.p.x = (buffer.width as f32 *0.5) as i32;
        memory.ball.p.y = (buffer.height as f32 *0.5) as i32;

        memory.is_initialized = true;
    }

    // TODO(Fermin): Fix entities getting out of bounce
    // TODO(Fermin): Use floats
    // TODO(Fermin): Use same struct for each pair??
    let player_speed = 3000.0; // pixels/s
    let drag = -8;
    let mut ddp = V2 { x: 0, y: 0};

    if input.keyboard.buttons.move_up.ended_down {
        if memory.l_entity.p.y > ENTITY_Y_PADDING {
            ddp.y = -1;
        }
    }
    if input.keyboard.buttons.move_down.ended_down {
        if memory.l_entity.p.y < buffer.height - ENTITY_Y_PADDING - memory.l_entity.height {
            ddp.y = 1;
        }
    }
    if input.keyboard.buttons.move_left.ended_down {
        if memory.b_entity.p.x > ENTITY_X_PADDING {
            ddp.x = -1;
        }
    }
    if input.keyboard.buttons.move_right.ended_down {
        if memory.b_entity.p.x < buffer.width - ENTITY_X_PADDING - memory.b_entity.width {
            ddp.x = 1;
        }
    }

    ddp.x *= player_speed as i32;
    ddp.y *= player_speed as i32;
    ddp.y += drag*memory.l_entity.dp.y;
    ddp.x += drag*memory.t_entity.dp.x;

    let player_delta = V2 {
        x: (0.5 * ddp.x as f32 * input.dt_for_frame.powi(2) + memory.t_entity.dp.x as f32 * input.dt_for_frame).round() as i32,
        y: (0.5 * ddp.y as f32 * input.dt_for_frame.powi(2) + memory.r_entity.dp.y as f32 * input.dt_for_frame).round() as i32,
    };

    memory.r_entity.p.y += player_delta.y;
    memory.l_entity.p.y = memory.r_entity.p.y;

    memory.t_entity.p.x += player_delta.x;
    memory.b_entity.p.x = memory.t_entity.p.x;

    memory.r_entity.dp.y = (ddp.y as f32 * input.dt_for_frame + memory.r_entity.dp.y as f32).round() as i32;
    memory.l_entity.dp.y = memory.r_entity.dp.y;

    memory.t_entity.dp.x = (ddp.x as f32 * input.dt_for_frame + memory.t_entity.dp.x as f32).round() as i32;
    memory.b_entity.dp.x = memory.t_entity.dp.x;

    draw_rectangle(
        &memory.l_entity.p,
        memory.l_entity.width,
        memory.l_entity.height,
        &memory.l_entity.color,
        buffer
    );
    draw_rectangle(
        &memory.r_entity.p,
        memory.r_entity.width,
        memory.r_entity.height,
        &memory.r_entity.color,
        buffer
    );
    draw_rectangle(
        &memory.t_entity.p,
        memory.t_entity.width,
        memory.t_entity.height,
        &memory.t_entity.color,
        buffer
    );
    draw_rectangle(
        &memory.b_entity.p,
        memory.b_entity.width,
        memory.b_entity.height,
        &memory.b_entity.color,
        buffer
    );
    draw_rectangle(
        &memory.ball.p,
        memory.ball.width,
        memory.ball.height,
        &memory.ball.color,
        buffer
    );
}
