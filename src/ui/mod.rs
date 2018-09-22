mod element;

use renderer;
use std::collections::hash_map::HashMap;
use std::sync::Mutex;

pub use self::element::*;

const TILE_SIZE: f32 = 16.0;
const OUTER_TILE_WIDTH: f32 = 4.0;
const PADDING: f32 = 2.0;
const SHEET_LENGTH: u32 = 4;

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
        hovering: false,
    });
    static ref WINDOW_DIMENSIONS: Mutex<(f32, f32)> = Mutex::new((0.0, 0.0));
}

struct UIState {
    elements: HashMap<u64, UIElement>,
    last_element: Option<UIElement>,
    mouse: MouseStatus,
    pressed_element: Option<u64>,
    hovering: bool,
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

#[derive(Clone, Copy)]
pub struct MouseStatus {
    pub x: f32,
    pub y: f32,
    pub last_pressed: bool,
    pub pressed: bool,
}

/// If using the Window provided by this crate, you don't need to call
/// this function. Window calls it inside refresh().
pub fn update(width: f64, height: f64, mouse: MouseStatus) -> UIStatus {
    {
        let mut dimensions = WINDOW_DIMENSIONS.lock().unwrap();
        *dimensions = (width as f32, height as f32);
    }

    let mut state = UI_STATE.lock().unwrap();
    if !state.mouse.pressed {
        state.pressed_element = None;
    }

    state.elements.clear();
    state.last_element = None;
    state.hovering = false;

    state.mouse = mouse;
    let hovering_button = state.hovering;
    UIStatus { hovering_button }
}

fn new_element(state: &UIState, identifier: String, t: UIElementType) -> UIElement {
    let y = if let Some(ref element) = state.last_element {
        element.y0 + 16.0 + TILE_SIZE + OUTER_TILE_WIDTH * 3.0
    } else {
        30.0
    };
    let e = UIElement {
        identifier,
        t,
        x0: 30.0,
        y0: y,
        x1: 30.0 + 88.0,
        y1: y + 16.0,
        anchor_x: 0.0,
        anchor_y: 0.0,
    };
    e
}

fn apply_anchor(ax: f32, ay: f32, x0: f32, y0: f32, x1: f32, y1: f32) -> (f32, f32, f32, f32) {
    let lock = WINDOW_DIMENSIONS.lock().unwrap();
    let (width, height) = *lock;
    let xo = width * ax;
    let yo = height * ay;
    (x0 + xo, y0 + yo, x1 + xo, y1 + yo)
}

fn draw_element(element: &UIElement, text: &str) {
    let &UIElement {
        t,
        x0,
        y0,
        x1,
        y1,
        anchor_x,
        anchor_y,
        ..
    } = element;
    let (x0, y0, x1, y1) = apply_anchor(anchor_x, anchor_y, x0, y0, x1, y1);

    if t != UIElementType::NoBackground {
        let tx = t as i32 as f32 / SHEET_LENGTH as f32; // The UV offset based on the element type
        let ty = 0.0;
        let tw = 1.0 / (3.0 * SHEET_LENGTH as f32); // UV width of a spritesheet tile
        let th = 1.0 / 3.0; // UV height of a spritesheet tile
        let x0_ = [x0 - TILE_SIZE - PADDING, x0 - PADDING, x1 + PADDING];
        let y0_ = [y0 - TILE_SIZE - PADDING, y0 - PADDING, y1 + PADDING];
        let x1_ = [x0 - PADDING, x1 + PADDING, x1 + PADDING + TILE_SIZE];
        let y1_ = [y0 - PADDING, y1 + PADDING, y1 + PADDING + TILE_SIZE];
        let tx = [tx, tx + tw, tx + tw * 2.0];
        let ty = [ty, ty + th, ty + th * 2.0];
        let z = NORMAL_UI_ELEMENT_DEPTH;

        for i in 0..9 {
            let xi = i % 3;
            let yi = i / 3;
            let (x0, y0, x1, y1) = (x0_[xi], y0_[yi], x1_[xi], y1_[yi]);
            let (tx0, ty0, tx1, ty1) = (tx[xi], ty[yi], tx[xi] + tw, ty[yi] + th);
            renderer::draw_quad(x0, y0, x1, y1, z, tx0, ty0, tx1, ty1, 0);
        }
    }

    renderer::queue_text(x0, y0, NORMAL_UI_TEXT_DEPTH, 16.0, text);
}