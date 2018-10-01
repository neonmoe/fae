pub mod element;
pub mod layout;

use renderer;
use std::collections::hash_map::HashMap;
use std::sync::Mutex;
use text;

use self::element::{UIElement, UIElementKind};
use self::layout::Rect;

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

/// Handled by the `window_bootstrap` feature, if in use.
pub fn update(width: f32, height: f32, dpi: f32, mouse: MouseStatus) -> UIStatus {
    renderer::render(width, height);
    text::update_dpi(dpi);
    layout::reset_layout();

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

fn new_element(identifier: String, kind: UIElementKind) -> UIElement {
    let (rect, alignment) = layout::create_next_element();
    let element = UIElement {
        identifier,
        kind,
        rect,
        alignment,
    };
    element
}

fn draw_element(element: &UIElement, text: &str) {
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
        let tx = kind as i32 as f32 / SHEET_LENGTH as f32; // The UV offset based on the element type
        let ty = 0.0;
        let tw = 1.0 / (3.0 * SHEET_LENGTH as f32); // UV width of a spritesheet tile
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

    text::queue_text(rect, NORMAL_UI_TEXT_DEPTH, 16.0, text, alignment);
}
