pub mod element;
pub mod keyboard;
pub mod layout;

pub use glutin::{ModifiersState, VirtualKeyCode};

use renderer;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use text::{self, TextCursor};

use self::element::{UIElement, UIElementKind};
pub use self::keyboard::KeyStatus;
use self::layout::Rect;
use clip;

const TILE_SIZE: f32 = 16.0;
const OUTER_TILE_WIDTH: f32 = 4.0;
const PADDING: f32 = 2.0;

const NORMAL_UI_ELEMENT_DEPTH: f32 = 0.0;
const NORMAL_UI_TEXT_DEPTH: f32 = NORMAL_UI_ELEMENT_DEPTH - 0.1;

lazy_static! {
    static ref UI_STATE: Mutex<UIState> = Mutex::new(UIState {
        elements: HashMap::new(),
        last_element: None,
        mouse: MouseStatus {
            x: 0.0,
            y: 0.0,
            last_pressed: false,
            pressed: false,
        },
        pressed_element: None,
        focused_element: None,
        hovering: false,
        keys: HashMap::new(),
        last_key_input_time: None,
    });
    static ref WINDOW_DIMENSIONS: Mutex<(f32, f32)> = Mutex::new((0.0, 0.0));
}

struct UIState {
    elements: HashMap<u64, UIElement>,
    last_element: Option<UIElement>,
    mouse: MouseStatus,
    pressed_element: Option<u64>,
    focused_element: Option<u64>,
    hovering: bool,
    keys: HashMap<VirtualKeyCode, KeyStatus>,
    last_key_input_time: Option<Instant>,
}

impl UIState {
    fn insert_element(&mut self, element: UIElement) {
        self.last_element = Some(element.clone());
        self.elements.insert(element.id(), element);
    }
}

pub struct UIStatus {
    pub hovering_button: bool,
}

/// Describes the current status of the mouse.
#[derive(Clone, Copy)]
pub struct MouseStatus {
    /// The x-coordinate of the mouse in logical pixels.
    pub x: f32,
    /// The y-coordinate of the mouse in logical pixels.
    pub y: f32,
    /// Was the mouse pressed during the previous frame?
    pub last_pressed: bool,
    /// Is the mouse pressed currently?
    pub pressed: bool,
}

impl MouseStatus {
    /// Returns true if the mouse was clicked, ie. was just
    /// released. True for one frame per click.
    #[inline]
    pub fn clicked(&self) -> bool {
        !self.pressed && self.last_pressed
    }
}

/// Handled by the `window_bootstrap` feature, if in use.
// TODO: Keyboard navigation of the UI
pub fn update(
    width: f32,
    height: f32,
    dpi: f32,
    mouse: MouseStatus,
    key_inputs: Vec<KeyStatus>,
    characters: Vec<char>,
) -> UIStatus {
    renderer::render(width, height);
    text::update_dpi(dpi);
    layout::reset_layout();

    {
        let mut dimensions = WINDOW_DIMENSIONS.lock().unwrap();
        *dimensions = (width as f32, height as f32);
    }

    let hovering_button;
    let focused_element;
    {
        let mut state = UI_STATE.lock().unwrap();
        if !state.mouse.pressed {
            state.pressed_element = None;
        }
        state.elements.clear();
        state.last_element = None;
        state.hovering = false;

        state.mouse = mouse;
        hovering_button = state.hovering;

        for key in state.keys.iter_mut().map(|(_, key_status)| key_status) {
            key.last_pressed = key.pressed;
        }
        for mut key_input in key_inputs {
            let keycode = key_input.keycode;
            key_input.last_pressed = {
                if !state.keys.contains_key(&keycode) {
                    false
                } else {
                    state.keys[&keycode].pressed
                }
            };
            state.keys.insert(keycode, key_input);
        }
        focused_element = state.focused_element;
    }

    if let Some(id) = focused_element {
        for character in characters {
            element::insert_input(id, character);
        }

        if keyboard::key_typed(
            VirtualKeyCode::V,
            Some(ModifiersState {
                ctrl: true,
                shift: false,
                alt: false,
                logo: false,
            }),
        ) {
            if let Some(paste) = clip::get() {
                element::insert_input_str(id, &paste);
            }
        }

        let mut last_key_input_time = {
            let mut state = UI_STATE.lock().unwrap();
            state.last_key_input_time
        };
        let right_held = keyboard::key_held(VirtualKeyCode::Right, None);
        let left_held = keyboard::key_held(VirtualKeyCode::Left, None);
        let now = Instant::now();
        if !right_held && !left_held {
            last_key_input_time = None;
        } else {
            let amount = if right_held && left_held {
                0
            } else if right_held {
                1
            } else {
                -1
            };
            if let Some(time) = last_key_input_time {
                if now > time && now - time > Duration::from_millis(500) {
                    element::move_cursor(amount);
                    last_key_input_time = Some(now - Duration::from_millis(470));
                }
            } else {
                last_key_input_time = Some(now);
                element::move_cursor(amount);
            }
        }
        let mut state = UI_STATE.lock().unwrap();
        state.last_key_input_time = last_key_input_time;
    }

    UIStatus { hovering_button }
}

fn new_element(identifier: String, kind: UIElementKind) -> UIElement {
    let (rect, alignment) = layout::create_next_element();
    UIElement {
        identifier,
        kind,
        rect,
        alignment,
    }
}

fn draw_element(element: &UIElement, text: &str, multiline: bool, cursor: Option<TextCursor>) {
    let &UIElement {
        kind,
        rect,
        alignment,
        ..
    } = element;
    let Rect {
        left,
        top,
        right,
        bottom,
    } = rect;

    if kind != UIElementKind::NoBackground {
        let sheet_length = UIElementKind::KindCount as i32;
        let tx = kind as i32 as f32 / sheet_length as f32; // The UV offset based on the element type
        let ty = 0.0;
        let tw = 1.0 / (3.0 * sheet_length as f32); // UV width of a spritesheet tile
        let th = 1.0 / 3.0; // UV height of a spritesheet tile
        let tx = [tx, tx + tw, tx + tw * 2.0];
        let ty = [ty, ty + th, ty + th * 2.0];

        let left_ = [left - TILE_SIZE - PADDING, left - PADDING, right + PADDING];
        let top_ = [top - TILE_SIZE - PADDING, top - PADDING, bottom + PADDING];
        let right_ = [left_[1], left_[2], left_[2] + TILE_SIZE];
        let bottom_ = [top_[1], top_[2], top_[2] + TILE_SIZE];
        let z = NORMAL_UI_ELEMENT_DEPTH;

        for i in 0..9 {
            let xi = i % 3;
            let yi = i / 3;
            let coords = (left_[xi], top_[yi], right_[xi], bottom_[yi]);
            let texcoords = (tx[xi], ty[yi], tx[xi] + tw, ty[yi] + th);
            let color = (0xFF, 0xFF, 0xFF, 0xFF);
            renderer::draw_quad(coords, texcoords, color, z, renderer::DRAW_CALL_INDEX_UI);
        }
    }

    text::queue_text(
        text,
        (rect.left, rect.top, NORMAL_UI_TEXT_DEPTH),
        rect.width(),
        rect.height(),
        alignment,
        multiline,
        cursor,
    );
}
