use features::{invariants_checked, log, GLOBAL_ERROR_LOGGER, GLOBAL_LOGGER};
use platform_types::{Button, Input, Speaker, State, StateParams, SFX};
use rendering::{Framebuffer, BLUE, GREY, PALETTE, PURPLE, RED, WHITE, YELLOW};

const GRID_WIDTH: u8 = 40;
const GRID_HEIGHT: u8 = 60;
const GRID_LENGTH: usize = GRID_WIDTH as usize * GRID_HEIGHT as usize;

#[derive(Copy, Clone, PartialEq, Eq)]
enum GridCell<T> {
    Absent,
    Present(T),
    Animating,
}

#[allow(dead_code)]
impl<T> GridCell<T> {
    pub fn is_present(&self) -> bool {
        match self {
            GridCell::Present(_) => true,
            _ => false,
        }
    }

    pub fn is_absent(&self) -> bool {
        match self {
            GridCell::Absent => true,
            _ => false,
        }
    }

    pub fn is_animating(&self) -> bool {
        match self {
            GridCell::Animating => true,
            _ => false,
        }
    }

    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> GridCell<U> {
        match self {
            GridCell::Present(x) => GridCell::Present(f(x)),
            GridCell::Absent => GridCell::Absent,
            GridCell::Animating => GridCell::Animating,
        }
    }

    pub fn and_then<U, F: FnOnce(T) -> GridCell<U>>(self, f: F) -> GridCell<U> {
        match self {
            GridCell::Present(x) => f(x),
            GridCell::Absent => GridCell::Absent,
            GridCell::Animating => GridCell::Animating,
        }
    }
}

macro_rules! on_left {
    ($x: expr) => {
        $x & 1 == 0
    };
    ($x: expr, bit) => {
        $x & 1
    };
}

type HalfHexSpec = u8;

fn get_colours(mut spec: HalfHexSpec) -> (u32, u32) {
    spec &= 0b0011_0011; //cut out BLACK (0b111)
    (
        PALETTE[(spec & 0b111) as usize],
        PALETTE[(spec >> 4) as usize],
    )
}

#[derive(Clone, Copy)]
enum Cursor {
    Unselected(usize),
    Selected(usize, usize),
}

impl Cursor {
    fn wrapping_add(self, other: usize) -> Cursor {
        use Cursor::*;
        match self {
            Unselected(c) => Unselected(c.wrapping_add(other)),
            Selected(c1, c2) => Selected(c1, c2.wrapping_add(other)),
        }
    }
}

use std::convert::From;

impl From<Cursor> for usize {
    fn from(c: Cursor) -> Self {
        use Cursor::*;
        match c {
            Unselected(c) => c,
            Selected(_, c2) => c2,
        }
    }
}

impl Cursor {
    fn iter(&self) -> impl Iterator<Item = usize> {
        use Cursor::*;
        match *self {
            Unselected(c) => vec![c].into_iter(),
            Selected(c1, c2) => vec![c1, c2].into_iter(),
        }
    }
}

struct Animation {
    x: u8,
    y: u8,
    target_x: u8,
    target_y: u8,
    x_rate: u8,
    y_rate: u8,
    spec: Option<HalfHexSpec>,
}

use std::cmp::{max, min};

const DELAY_FACTOR: u8 = 16;

impl Animation {
    pub fn new<H: Into<Option<HalfHexSpec>>>(i: usize, target_i: usize, spec: H) -> Self {
        let spec = spec.into();
        let (x, y) = i_to_xy(i);
        let (target_x, target_y) = i_to_xy(target_i);

        let (x_diff, y_diff) = (
            if target_x == x {
                0
            } else if x > target_x {
                x - target_x
            } else {
                target_x - x
            },
            if target_y == y {
                0
            } else if y > target_y {
                y - target_y
            } else {
                target_y - y
            },
        );

        Animation {
            x,
            y,
            x_rate: max(x_diff / DELAY_FACTOR, 1),
            y_rate: max(y_diff / DELAY_FACTOR, 1),
            target_x,
            target_y,
            spec,
        }
    }

    pub fn is_complete(&self) -> bool {
        self.x == self.target_x && self.y == self.target_y
    }

    pub fn approach_target(&mut self) {
        let (d_x, d_y) = self.get_delta();

        self.x = match d_x {
            x if x > 0 => self.x.saturating_add(x as u8),
            x if x < 0 => self.x.saturating_sub(x.abs() as u8),
            _ => self.x,
        };
        self.y = match d_y {
            y if y > 0 => self.y.saturating_add(y as u8),
            y if y < 0 => self.y.saturating_sub(y.abs() as u8),
            _ => self.y,
        };
    }

    fn get_delta(&self) -> (i8, i8) {
        (
            if self.target_x == self.x {
                0
            } else if self.x > self.target_x {
                let x_diff = self.x - self.target_x;
                -(min(x_diff, self.x_rate) as i8)
            } else {
                let x_diff = self.target_x - self.x;
                min(x_diff, self.x_rate) as i8
            },
            if self.target_y == self.y {
                0
            } else if self.y > self.target_y {
                let y_diff = self.y - self.target_y;
                -(min(y_diff, self.y_rate) as i8)
            } else {
                let y_diff = self.target_y - self.y;
                min(y_diff, self.y_rate) as i8
            },
        )
    }
}

fn advance_animations(state: &mut GameState, speaker: &mut Speaker) {
    for animation_index in (0..state.animations.len()).rev() {
        let animation = &mut state.animations[animation_index];
        animation.approach_target();

        if animation.is_complete() {
            let index = xy_to_i(animation.x, animation.y);

            if state.grid[index].is_present() {
                //hope it is moved soon?
                continue;
            }

            state.grid[index] = if let Some(spec) = animation.spec {
                GridCell::Present(spec)
            } else {
                GridCell::Absent
            };

            let other_index = if on_left!(animation.x) {
                index + 1
            } else {
                index - 1
            };
            if state.grid[other_index].map(get_colours) == state.grid[index].map(get_colours) {
                state.grid[other_index] = GridCell::Absent;
                state.grid[index] = GridCell::Absent;
            }

            state.animations.swap_remove(animation_index);
            apply_gravity_once(&mut state.grid, speaker);
            speaker.request_sfx(SFX::MovePiece);
        }
    }
}

pub struct GameState {
    grid: Grid,
    cursor: Cursor,
    frame_counter: usize,
    animations: Vec<Animation>,
}

type Grid = [GridCell<HalfHexSpec>; GRID_LENGTH];

fn new_grid() -> Grid {
    let mut grid: Grid = [GridCell::Absent; GRID_LENGTH];
    let mut c: HalfHexSpec = 0;
    const W: usize = GRID_WIDTH as usize * 0b1_1000;
    for i in 0..GRID_LENGTH {
        if i < W || i > GRID_LENGTH - W || i % W <= 1 || i % W >= W - 2 {
            continue;
        }

        grid[i] = GridCell::Present(c);
        c = c.wrapping_add(1);
    }
    grid
}

impl GameState {
    pub fn new(_seed: [u8; 16]) -> GameState {
        let grid: Grid = new_grid();

        GameState {
            grid,
            cursor: Cursor::Unselected(GRID_WIDTH as usize + 1),
            frame_counter: 0,
            animations: Vec::with_capacity(GRID_WIDTH as usize),
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

const HEX_WIDTH: u8 = 4;
const HEX_HEIGHT: u8 = 8;
const HALF_HEX_HEIGHT: u8 = HEX_HEIGHT / 2;
const EDGE_OFFSET: u8 = 6;

const ROW_TYPES: u8 = 3;

fn p_xy(x: u8, y: u8) -> (u8, u8) {
    let x_offset = (y % ROW_TYPES) * HEX_WIDTH;
    if on_left!(x) {
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

//This way we don't need to allocate a closure every frame.
fn marching_ants(frame_counter: usize) -> fn(usize, usize, usize, usize) -> u32 {
    macro_rules! marching_ants {
        ($offset: expr) => {{
            fn _marching_ants(x: usize, y: usize, _: usize, _: usize) -> u32 {
                if (x + y + $offset) & 2 == 0 {
                    YELLOW
                } else {
                    PURPLE
                }
            }

            _marching_ants
        }};
    }

    match frame_counter & 0b1_1000 {
        0 => marching_ants!(0),
        0b0_1000 => marching_ants!(1),
        0b1_0000 => marching_ants!(2),
        _ => marching_ants!(3),
    }
}

fn is_index_next_to(x: u8, _y: u8, index: usize) -> bool {
    if index < GRID_LENGTH {
        let width = GRID_WIDTH as usize;
        let new_x = index % width;
        let looped = (x == 0 && new_x == width - 1) || (x as usize == width - 1 && new_x == 0);
        if !looped {
            return true;
        }
    }
    false
}

//see `design/gridMovement.md` for the derivation of this table.
static MOVEMENT: [i8; 24] = {
    const W: i8 = GRID_WIDTH as i8;

    [
        -(W + 1),
        2 * W - 1,
        W - 1,
        1,
        -(2 * W + 1),
        W - 1,
        -1,
        -(W + 1),
        -(W - 1),
        2 * W + 1,
        W - 1,
        1,
        -(2 * W + 1),
        W - 1,
        -1,
        -(W - 1),
        -(W - 1),
        2 * W + 1,
        W + 1,
        1,
        -(2 * W - 1),
        W + 1,
        -1,
        -(W - 1),
    ]
};

#[derive(Clone, Copy)]
enum Dir {
    Up,
    Down,
    Left,
    Right,
}

fn get_movement_offset(x: u8, y: u8, dir: Dir) -> i8 {
    let index = ((y % ROW_TYPES) << 3) | (on_left!(x, bit) << 2) | dir as u8;

    MOVEMENT[index as usize]
}

fn get_hex_index(index: usize, dir: Dir) -> Option<usize> {
    let (x, y) = i_to_xy(index);
    let new_index = index.wrapping_add(get_movement_offset(x, y, dir) as usize);

    if is_index_next_to(x, y, new_index) {
        Some(new_index)
    } else {
        None
    }
}

fn i_to_xy(i: usize) -> (u8, u8) {
    (
        (i % GRID_WIDTH as usize) as u8,
        (i / GRID_WIDTH as usize) as u8,
    )
}

fn xy_to_i(x: u8, y: u8) -> usize {
    y as usize * GRID_WIDTH as usize + x as usize
}

fn draw_hexagon(framebuffer: &mut Framebuffer, x: u8, y: u8, spec: HalfHexSpec) {
    let (inside, outline) = get_colours(spec);

    let (p_x, p_y) = p_xy(x, y);
    if on_left!(x) {
        framebuffer.hexagon_left(p_x, p_y, inside, outline);
    } else {
        framebuffer.hexagon_right(p_x, p_y, inside, outline);
    }
}

fn apply_gravity_once(grid: &mut Grid, speaker: &mut Speaker) {
    for index in 0..grid.len() {
        if let GridCell::Present(half_hex) = grid[index] {
            let (x, y) = i_to_xy(index);

            // these 8 hal-hexes are the ones in the very middle that would cause animation loops
            // otherwise. It appears this restriction would be unnecessary if there was an odd
            // number of rows/columns.
            if ((x == GRID_WIDTH / 2 - 1 || x == GRID_WIDTH / 2 - 2) && y == GRID_HEIGHT / 2 - 1)
                || ((x == GRID_WIDTH / 2 || x == GRID_WIDTH / 2 + 1) && y == GRID_HEIGHT / 2 - 2)
                || ((x == GRID_WIDTH / 2 - 1 || x == GRID_WIDTH / 2 - 2)
                    && y == GRID_HEIGHT / 2 + 1)
                || ((x == GRID_WIDTH / 2 || x == GRID_WIDTH / 2 + 1) && y == GRID_HEIGHT / 2)
            {
                continue;
            }

            let should_fall_right = x < GRID_WIDTH / 2;
            let should_fall_down = y < GRID_HEIGHT / 2;

            let (forward_x, backward_x) = if should_fall_right {
                (Dir::Right, Dir::Left)
            } else {
                (Dir::Left, Dir::Right)
            };

            let (forward_y, backward_y) = if should_fall_down {
                (Dir::Down, Dir::Up)
            } else {
                (Dir::Up, Dir::Down)
            };

            macro_rules! move_if_possible {
                //pretend these names are more generic
                (target $forward_xy_index:expr, $forward_x_index:expr, $forward_y_index: expr) => {
                    if let (Some(forward_x_index), Some(forward_xy_index), Some(forward_y_index)) =
                        ($forward_x_index, $forward_xy_index, $forward_y_index)
                    {
                        if [forward_x_index, forward_xy_index, forward_y_index]
                            .into_iter()
                            .map(|i| &grid[*i])
                            .all(|h| h.is_absent())
                        {
                            grid[forward_xy_index] = GridCell::Present(half_hex);
                            grid[index] = GridCell::Absent;
                            speaker.request_sfx(SFX::Wud);
                            continue;
                        }
                    }
                };
            }

            let forward_x_index = get_hex_index(index, forward_x);
            let forward_xy_index = forward_x_index.and_then(|i| get_hex_index(i, forward_y));
            let forward_y_index = get_hex_index(index, forward_y);

            //  →
            // ↓↘

            move_if_possible!(target forward_xy_index, forward_x_index, forward_y_index);

            let backward_x_forward_y_index =
                forward_y_index.and_then(|i| get_hex_index(i, backward_x));

            // ↙↓↘

            move_if_possible!(target forward_xy_index, backward_x_forward_y_index, forward_y_index);

            let forward_x_backward_y_index =
                forward_x_index.and_then(|i| get_hex_index(i, backward_y));

            // ↗
            // →
            // ↘

            move_if_possible!(target forward_xy_index, forward_x_backward_y_index, forward_x_index);
        }
    }
}

#[inline]
pub fn update_and_render(
    framebuffer: &mut Framebuffer,
    state: &mut GameState,
    input: Input,
    speaker: &mut Speaker,
) {
    //
    //UPDATE
    //
    advance_animations(state, speaker);
    apply_gravity_once(&mut state.grid, speaker);

    match input.gamepad {
        Button::B => framebuffer.clear_to(BLUE),
        Button::Select => {
            if invariants_checked!() {
                log!(state.animations.len() == 0);
            }
            framebuffer.clear_to(WHITE)
        }
        Button::Start => framebuffer.clear_to(RED),
        _ => {}
    }

    if input.pressed_this_frame(Button::A) {
        match state.cursor {
            Cursor::Unselected(c) => {
                if !state.grid[c].is_animating() {
                    state.cursor = Cursor::Selected(c, c);
                }
            }
            Cursor::Selected(c1, c2) => {
                macro_rules! animate {
                    ($h1: expr, $h2: expr) => {
                        speaker.request_sfx(SFX::MovePiece);
                        state.grid[c1] = GridCell::Animating;
                        state.grid[c2] = GridCell::Animating;
                        state.animations.push(Animation::new(c1, c2, $h1));
                        state.animations.push(Animation::new(c2, c1, $h2));
                        state.cursor = Cursor::Unselected(c2);
                    };
                }

                match (state.grid[c1], state.grid[c2]) {
                    (GridCell::Present(h1), GridCell::Present(h2)) => {
                        animate!(h1, h2);
                    }
                    (GridCell::Present(h1), GridCell::Absent) => {
                        animate!(h1, None);
                    }
                    (GridCell::Absent, GridCell::Present(h2)) => {
                        animate!(None, h2);
                    }
                    (GridCell::Absent, GridCell::Absent) => {
                        animate!(None, None);
                    }
                    _ => {}
                }
            }
        };
    }

    macro_rules! move_cursor {
        ($dir: expr) => {
            let cursor_num: usize = state.cursor.into();

            let (x, y) = i_to_xy(cursor_num);

            let offset: i8 = get_movement_offset(x, y, $dir);

            let new_cursor = state.cursor.wrapping_add(offset as usize);
            let new_cursor_num: usize = new_cursor.into();

            if is_index_next_to(x, y, new_cursor_num) {
                state.cursor = new_cursor;
            }
        };
    }

    if input.pressed_this_frame(Button::Up) {
        move_cursor!(Dir::Up);
    }
    if input.pressed_this_frame(Button::Down) {
        move_cursor!(Dir::Down);
    }
    if input.pressed_this_frame(Button::Left) {
        move_cursor!(Dir::Left);
    }
    if input.pressed_this_frame(Button::Right) {
        move_cursor!(Dir::Right);
    }

    //
    // RENDER
    //

    framebuffer.clear_to(GREY);

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            if let GridCell::Present(spec) = state.grid[xy_to_i(x, y)] {
                draw_hexagon(framebuffer, x, y, spec);
            }
        }
    }

    for index in state.cursor.iter() {
        let (x, y) = i_to_xy(index);
        let (p_x, p_y) = p_xy(x, y);
        framebuffer.draw_rect_with_shader(
            p_x as usize - 1,
            p_y as usize - 1,
            6,
            10,
            marching_ants(state.frame_counter),
        );
    }

    for anim in state.animations.iter() {
        if let &Animation {
            x,
            y,
            spec: Some(spec),
            ..
        } = anim
        {
            draw_hexagon(framebuffer, x, y, spec);
        }
    }

    state.frame_counter += 1;
}
