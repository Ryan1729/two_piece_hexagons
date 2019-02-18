use features::{GLOBAL_ERROR_LOGGER, GLOBAL_LOGGER};
use platform_types::{Button, Input, Speaker, State, StateParams, SFX};
use rendering::{Framebuffer, BLUE, GREEN, PALETTE, RED, WHITE};
use std::cmp::{max, min};

const GRID_WIDTH: u8 = 40;
const GRID_HEIGHT: u8 = 60;
const GRID_LENGTH: usize = GRID_WIDTH as usize * GRID_HEIGHT as usize;

type Grid = Vec<u8>;

#[derive(Default)]
pub struct GameState {
    w: u8,
    h: u8,
    grid: Grid,
}

fn new_grid(w: u8, h: u8) -> Grid {
    let l = w as usize * h as usize;
    let mut grid: Grid = Vec::with_capacity(l);
    let mut c: u8 = 0;
    for _ in 0..l {
        grid.push(c);
        c = c.wrapping_add(1);
    }
    grid
}

impl GameState {
    pub fn new(_seed: [u8; 16]) -> GameState {
        let grid: Grid = new_grid(GRID_WIDTH, GRID_HEIGHT);

        GameState {
            grid,
            w: GRID_WIDTH,
            h: GRID_HEIGHT,
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

#[inline]
pub fn update_and_render(
    framebuffer: &mut Framebuffer,
    state: &mut GameState,
    input: Input,
    _speaker: &mut Speaker,
) {
    framebuffer.clear_to(framebuffer.buffer[0]);

    let edge_offset = 6;
    let offset = 4;
    for y in 0..state.h {
        for x in 0..state.w {
            let (inside, outline) =
                get_colours(state.grid[y as usize * state.w as usize + x as usize]);

            let x_offset = (y % 3) * offset;
            if x & 1 == 0 {
                framebuffer.hexagon_left(
                    x * 6 + x_offset + edge_offset,
                    y * 4 + edge_offset,
                    inside,
                    outline,
                );
            } else {
                framebuffer.hexagon_right(
                    x * 6 + x_offset - 2 + edge_offset,
                    y * 4 + edge_offset,
                    inside,
                    outline,
                );
            }
        }
    }

    match input.gamepad {
        Button::A => framebuffer.clear_to(GREEN),
        Button::B => framebuffer.clear_to(BLUE),
        Button::Select => framebuffer.clear_to(WHITE),
        Button::Start => framebuffer.clear_to(RED),
        Button::Up => {
            state.h = state.h.saturating_sub(1);
            state.grid = new_grid(state.w, state.h);
        }
        Button::Down => {
            if state.h < GRID_HEIGHT {
                state.h += 1;
                state.grid = new_grid(state.w, state.h);
            }
        }
        Button::Left => {
            state.w = state.w.saturating_sub(1);
            state.grid = new_grid(state.w, state.h);
        }
        Button::Right => {
            if state.w < GRID_WIDTH {
                state.w += 1;
                state.grid = new_grid(state.w, state.h);
            }
        }
        _ => {}
    }
}
