use crate::memory::Memory;
use crate::ui::EventPump;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

pub type InputRef = Rc<RefCell<Input>>;

// The "strobe state": the order in which the NES reads the buttons.
const STROBE_A: u8 = 0;
const STROBE_B: u8 = 1;
const STROBE_SELECT: u8 = 2;
const STROBE_START: u8 = 3;
const STROBE_UP: u8 = 4;
const STROBE_DOWN: u8 = 5;
const STROBE_LEFT: u8 = 6;
const STROBE_RIGHT: u8 = 7;

#[derive(Default, Debug)]
struct Gamepad {
    left: bool,
    right: bool,
    up: bool,
    down: bool,
    a: bool,
    b: bool,
    select: bool,
    start: bool,
    strobe_state: u8,
}

impl Gamepad {
    fn next_state(&mut self) -> u8 {
        let state = match self.strobe_state {
            STROBE_A => self.a,
            STROBE_B => self.b,
            STROBE_SELECT => self.select,
            STROBE_START => self.start,
            STROBE_UP => self.up,
            STROBE_DOWN => self.down,
            STROBE_LEFT => self.left,
            STROBE_RIGHT => self.right,
            _ => panic!("invalid state {}", self.strobe_state),
        };
        self.strobe_state = (self.strobe_state + 1) & 7;
        state as u8
    }
    fn reset(&mut self) {
        self.strobe_state = STROBE_A;
    }
}

#[derive(Default)]
pub struct Input {
    gamepad1: Gamepad,
    gamepad2: Gamepad,
    event_pump: Option<EventPump>,
    turboa: bool,
    turbob: bool,
    lctrl: bool,
    lshift: bool,
}

pub enum InputResult {
    Continue,
    Quit,
    Open,
    Menu,
    Reset,
    PowerCycle,
    IncSpeed,
    DecSpeed,
    FastForward(bool),
    SetState(u8),
    Save,
    Load,
    ToggleSound,
    ToggleDebug,
    Screenshot,
    ToggleRecord,
    ToggleFullscreen,
    CycleLogLevel,
}

use InputResult::*;

impl Input {
    pub fn new() -> Self {
        Self {
            gamepad1: Gamepad::default(),
            gamepad2: Gamepad::default(),
            event_pump: None,
            turboa: false,
            turbob: false,
            lctrl: false,
            lshift: false,
        }
    }

    pub fn init(event_pump: EventPump) -> Self {
        Self {
            gamepad1: Gamepad::default(),
            gamepad2: Gamepad::default(),
            event_pump: Some(event_pump),
            turboa: false,
            turbob: false,
            lctrl: false,
            lshift: false,
        }
    }

    pub fn poll_events(&mut self, turbo: bool) -> InputResult {
        if self.turboa {
            self.gamepad1.a = self.turboa && turbo;
        }
        if self.turbob {
            self.gamepad1.b = self.turbob && turbo;
        }
        if let Some(event_pump) = &mut self.event_pump {
            if let Some(event) = event_pump.poll_event() {
                let result = match event {
                    Event::Quit { .. } => Quit,
                    Event::KeyDown {
                        keycode: Some(key), ..
                    } => match key {
                        Keycode::Escape => Menu,
                        Keycode::LCtrl => {
                            self.lctrl = true;
                            Continue
                        }
                        Keycode::LShift => {
                            self.lshift = true;
                            Continue
                        }
                        Keycode::O if self.lctrl => Open,
                        Keycode::Q if self.lctrl => Quit,
                        Keycode::R if self.lctrl => Reset,
                        Keycode::P if self.lctrl => PowerCycle,
                        Keycode::Equals if self.lctrl => IncSpeed,
                        Keycode::Minus if self.lctrl => DecSpeed,
                        Keycode::Space => FastForward(true),
                        Keycode::Num1 if self.lctrl => SetState(1),
                        Keycode::Num2 if self.lctrl => SetState(2),
                        Keycode::Num3 if self.lctrl => SetState(3),
                        Keycode::Num4 if self.lctrl => SetState(4),
                        Keycode::S if self.lctrl => Save,
                        Keycode::L if self.lctrl => Load,
                        Keycode::M if self.lctrl => ToggleSound,
                        Keycode::V if self.lctrl => ToggleRecord,
                        Keycode::D if self.lctrl => ToggleDebug,
                        Keycode::Return if self.lctrl => ToggleFullscreen,
                        Keycode::F10 => Screenshot,
                        Keycode::F9 => CycleLogLevel,
                        _ => self.handle_gamepad_event(key, true),
                    },
                    Event::KeyUp {
                        keycode: Some(key), ..
                    } => match key {
                        Keycode::LCtrl => {
                            self.lctrl = false;
                            Continue
                        }
                        Keycode::LShift => {
                            self.lshift = false;
                            Continue
                        }
                        Keycode::Space => FastForward(false),
                        _ => self.handle_gamepad_event(key, false),
                    },
                    _ => Continue,
                };
                return result;
            }
        }
        InputResult::Continue
    }

    fn handle_gamepad_event(&mut self, key: Keycode, down: bool) -> InputResult {
        match key {
            Keycode::Z => self.gamepad1.a = down,
            Keycode::X => self.gamepad1.b = down,
            Keycode::A => self.turboa = down,
            Keycode::S => self.turbob = down,
            Keycode::RShift => self.gamepad1.select = down,
            Keycode::Return => self.gamepad1.start = down,
            Keycode::Up => self.gamepad1.up = down,
            Keycode::Down => self.gamepad1.down = down,
            Keycode::Left => self.gamepad1.left = down,
            Keycode::Right => self.gamepad1.right = down,
            _ => {}
        }
        Continue
    }
}

impl Memory for Input {
    fn readb(&mut self, addr: u16) -> u8 {
        match addr {
            0x4016 => self.gamepad1.next_state(),
            0x4017 => self.gamepad2.next_state(),
            _ => 0,
        }
    }

    fn writeb(&mut self, addr: u16, _val: u8) {
        if addr == 0x4016 {
            self.gamepad1.reset();
            self.gamepad2.reset();
        }
    }
}

impl fmt::Debug for Input {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::result::Result<(), fmt::Error> {
        write!(f, "Input {{ }} ")
    }
}
