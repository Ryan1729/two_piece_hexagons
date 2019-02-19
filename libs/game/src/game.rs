use features::{GLOBAL_ERROR_LOGGER, GLOBAL_LOGGER};
use platform_types::{Button, Input, Speaker, State, StateParams, SFX};
use rendering::{Framebuffer, BLUE, GREEN, PALETTE, RED, WHITE, YELLOW};

const GRID_WIDTH: u8 = 40;
const GRID_HEIGHT: u8 = 60;
const GRID_LENGTH: usize = GRID_WIDTH as usize * GRID_HEIGHT as usize;

type Grid = [u8; GRID_LENGTH];

pub struct GameState {
    grid: Grid,
    cursor: usize,
}

fn new_grid() -> Grid {
    let mut grid: Grid = [0; GRID_LENGTH];
    let mut c: u8 = 0;
    for i in 0..GRID_LENGTH {
        grid[i] = c;
        c = c.wrapping_add(1);
    }
    grid
}

impl GameState {
    pub fn new(_seed: [u8; 16]) -> GameState {
        let grid: Grid = new_grid();

        GameState {
            grid,
            cursor: GRID_WIDTH as usize + 1,
        }
    }
}

pub struct EntireState {
    pub game_state: GameState,
    pub framebuffer: Framebuffer,
    pub input: Input,
    pub speaker: Speaker,
}

impl EntireState {
    pub fn new((seed, logger, error_logger): StateParams) -> Self {
        let framebuffer = Framebuffer::new();

        unsafe {
            GLOBAL_LOGGER = logger;
            GLOBAL_ERROR_LOGGER = error_logger;
        }

        EntireState {
            game_state: GameState::new(seed),
            framebuffer,
            input: Input::new(),
            speaker: Speaker::new(),
        }
    }
}

impl State for EntireState {
    fn frame(&mut self, handle_sound: fn(SFX)) {
        update_and_render(
            &mut self.framebuffer,
            &mut self.game_state,
            self.input,
            &mut self.speaker,
        );

        self.input.previous_gamepad = self.input.gamepad;

        for request in self.speaker.drain() {
            handle_sound(request);
        }
    }

    fn press(&mut self, button: Button::Ty) {
        if self.input.previous_gamepad.contains(button) {
            //This is meant to pass along the key repeat, if any.
            //Not sure if rewriting history is the best way to do this.
            self.input.previous_gamepad.remove(button);
        }

        self.input.gamepad.insert(button);
    }

    fn release(&mut self, button: Button::Ty) {
        self.input.gamepad.remove(button);
    }

    fn get_frame_buffer(&self) -> &[u32] {
        &self.framebuffer.buffer
    }
}

fn get_colours(mut spec: u8) -> (u32, u32) {
    spec &= 0b0111_0111;
    (
        PALETTE[(spec & 0b111) as usize],
        PALETTE[(spec >> 4) as usize],
    )
}

const HEX_WIDTH: u8 = 4;
const HEX_HEIGHT: u8 = 8;
const HALF_HEX_HEIGHT: u8 = HEX_HEIGHT / 2;
const EDGE_OFFSET: u8 = 6;

fn p_xy(x: u8, y: u8) -> (u8, u8) {
    let x_offset = (y % 3) * HEX_WIDTH;
    if x & 1 == 0 {
        (
            x * 6 + x_offset + EDGE_OFFSET,
            y * HALF_HEX_HEIGHT + EDGE_OFFSET,
        )
    } else {
        (
            x * 6 + x_offset - 2 + EDGE_OFFSET,
            y * HALF_HEX_HEIGHT + EDGE_OFFSET,
        )
    }
}

#[inline]
pub fn update_and_render(
    framebuffer: &mut Framebuffer,
    state: &mut GameState,
    input: Input,
    _speaker: &mut Speaker,
) {
    framebuffer.clear_to(framebuffer.buffer[0]);

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let (inside, outline) =
                get_colours(state.grid[y as usize * GRID_WIDTH as usize + x as usize]);

            let (p_x, p_y) = p_xy(x, y);
            if x & 1 == 0 {
                framebuffer.hexagon_left(p_x, p_y, inside, outline);
            } else {
                framebuffer.hexagon_right(p_x, p_y, inside, outline);
            }
        }
    }

    let (x, y) = (
        (state.cursor % GRID_WIDTH as usize) as u8,
        (state.cursor / GRID_WIDTH as usize) as u8,
    );

    let (p_x, p_y) = p_xy(x, y);
    framebuffer.draw_rect(p_x as usize - 1, p_y as usize - 1, 6, 10, YELLOW);

    match input.gamepad {
        Button::A => framebuffer.clear_to(GREEN),
        Button::B => framebuffer.clear_to(BLUE),
        Button::Select => framebuffer.clear_to(WHITE),
        Button::Start => framebuffer.clear_to(RED),
        _ => {}
    }

    if input.pressed_this_frame(Button::Up) {
        if y > 0 {
            state.cursor = state.cursor.saturating_sub(GRID_WIDTH as usize);
        }
    }
    if input.pressed_this_frame(Button::Down) {
        if state.cursor + (GRID_WIDTH as usize) < GRID_LENGTH as usize {
            state.cursor += GRID_WIDTH as usize;
        }
    }
    if input.pressed_this_frame(Button::Left) {
        if x > 0 {
            state.cursor = state.cursor.saturating_sub(1);
        }
    }
    if input.pressed_this_frame(Button::Right) {
        if x < GRID_WIDTH - 1 {
            state.cursor += 1;
        }
    }
}
