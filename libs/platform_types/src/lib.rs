#[macro_use]
extern crate bitflags;

#[macro_export]
macro_rules! w {
    () => {
        256
    };
    (.0) => {
        256.0
    };
}

#[macro_export]
macro_rules! h {
    () => {
        256
    };
    (.0) => {
        256.0
    };
}

//in pixels
pub const SCREEN_WIDTH: usize = w!();
pub const SCREEN_HEIGHT: usize = h!();
pub const SCREEN_LENGTH: usize = SCREEN_WIDTH * SCREEN_HEIGHT;

#[derive(Clone, Copy, Default, Debug)]
pub struct Input {
    pub gamepad: Button::Ty,
    pub previous_gamepad: Button::Ty,
}

impl Input {
    pub fn new() -> Self {
        Input {
            gamepad: Button::Ty::empty(),
            previous_gamepad: Button::Ty::empty(),
        }
    }

    pub fn pressed_this_frame(&self, buttons: Button::Ty) -> bool {
        !self.previous_gamepad.contains(buttons) && self.gamepad.contains(buttons)
    }

    pub fn released_this_frame(&self, buttons: Button::Ty) -> bool {
        self.previous_gamepad.contains(buttons) && !self.gamepad.contains(buttons)
    }
}

//TODO more meaningful names for these?
//TODO clear out unused sound effects

#[derive(Clone, Copy, Debug)]
pub enum SFX {
    CardPlace,
    CardSlide,
    ChipsCollide,
    DieShuffle,
    DieThrow,
    ButtonPress,
}

impl SFX {
    pub fn to_sound_key(&self) -> &'static str {
        match *self {
            SFX::CardPlace => "cardPlace",
            SFX::CardSlide => "cardSlide",
            SFX::ChipsCollide => "chipsCollide",
            SFX::DieShuffle => "dieShuffle",
            SFX::DieThrow => "dieThrow",
            SFX::ButtonPress => "buttonPress",
        }
    }
}

pub struct Speaker {
    requests: Vec<SFX>,
}

impl Speaker {
    pub fn new() -> Self {
        Speaker {
            requests: Vec::with_capacity(8),
        }
    }

    pub fn request_sfx(&mut self, sfx: SFX) {
        self.requests.push(sfx);
    }

    pub fn drain<'a>(&'a mut self) -> impl Iterator<Item = SFX> + 'a {
        self.requests.drain(..)
    }
}

// These values are deliberately picked to be the same as the ones in NES' input registers.
#[allow(non_snake_case)]
pub mod Button {
    bitflags! {
        #[derive(Default)]
        pub flags Ty: u8 {
            const A          = 1 << 0,
            const B          = 1 << 1,
            const Select     = 1 << 2,
            const Start      = 1 << 3,
            const Up         = 1 << 4,
            const Down       = 1 << 5,
            const Left       = 1 << 6,
            const Right      = 1 << 7
        }
    }
}

pub type Logger = Option<fn(&str) -> ()>;

pub type StateParams = ([u8; 16], Logger, Logger);

pub trait State {
    fn frame(&mut self, handle_sound: fn(SFX));

    fn press(&mut self, button: Button::Ty);

    fn release(&mut self, button: Button::Ty);

    fn get_frame_buffer(&self) -> &[u32];
}
