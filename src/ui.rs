use renderer;
use std::sync::Mutex;

const TILE_SIZE: f32 = 16.0;
const OUTER_TILE_WIDTH: f32 = 4.0;
const PADDING: f32 = 2.0;
const SHEET_LENGTH: u32 = 4;

lazy_static! {
    static ref UI_STATE: Mutex<UIState> = Mutex::new(UIState {
        elements: Vec::new(),
        mouse: MouseStatus {
            x: 0.0,
            y: 0.0,
            last_pressed: false,
            pressed: false,
        },
        pressed_element: None,
        hovering: false,
    });
}

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq)]
enum UIElementType {
    NoBackground = -1,
    ButtonNormal = 0,
    ButtonHovered = 1,
    ButtonPressed = 2,
    Panel = 3,
}

struct UIElement {
    id: usize,
    t: UIElementType,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

impl UIElement {
    fn is_point_inside(&self, x: f32, y: f32) -> bool {
        !(x < self.x - PADDING - OUTER_TILE_WIDTH
            || x >= self.x + self.w + PADDING + OUTER_TILE_WIDTH
            || y < self.y - PADDING - OUTER_TILE_WIDTH
            || y >= self.y + self.h + PADDING + OUTER_TILE_WIDTH)
    }
}

struct UIState {
    elements: Vec<UIElement>,
    mouse: MouseStatus,
    pressed_element: Option<usize>,
    hovering: bool,
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
    renderer::render(width, height);
    let mut state = UI_STATE.lock().unwrap();
    if !state.mouse.pressed {
        state.pressed_element = None;
    }

    state.elements.clear();
    state.mouse = mouse;
    let hovering_button = state.hovering;
    state.hovering = false;
    UIStatus { hovering_button }
}

pub fn label(label: &str) {
    let mut state = UI_STATE.lock().unwrap();

    let element = new_element(&state, UIElementType::Panel);
    draw_element(&element, label);
    state.elements.push(element);
}

pub fn button(label: &str) -> bool {
    let mut state = UI_STATE.lock().unwrap();

    let mut element = new_element(&state, UIElementType::ButtonNormal);
    let hovered = element.is_point_inside(state.mouse.x, state.mouse.y);
    let just_released = !state.mouse.pressed && state.mouse.last_pressed;
    let can_be_pressed =
        state.pressed_element.is_none() || state.pressed_element.unwrap() == element.id;

    if state.mouse.pressed && hovered && can_be_pressed {
        element.t = UIElementType::ButtonPressed;
        state.pressed_element = Some(element.id);
    } else if hovered {
        element.t = UIElementType::ButtonHovered;
    }

    draw_element(&element, label);
    state.elements.push(element);
    state.hovering |= hovered;

    hovered && just_released && can_be_pressed
}

fn new_element(state: &UIState, t: UIElementType) -> UIElement {
    UIElement {
        id: state.elements.len(),
        t,
        x: 30.0,
        y: if let Some(element) = state.elements.last() {
            element.y + 16.0 + TILE_SIZE + OUTER_TILE_WIDTH * 3.0
        } else {
            30.0
        },
        w: 88.0,
        h: 16.0,
    }
}

#[cfg_attr(rustfmt, rustfmt_skip)]
fn draw_element(element: &UIElement, text: &str) {
    let &UIElement { t, x, y, w, h, .. } = element;

    if t != UIElementType::NoBackground {
        let tx = t as i32 as f32 / SHEET_LENGTH as f32; // The UV offset based on the element type
        let ty = 0.0;
        let tw = 1.0 / (3.0 * SHEET_LENGTH as f32); // UV width of a spritesheet tile
        let th = 1.0 / 3.0; // UV height of a spritesheet tile
        let x = [x - TILE_SIZE - PADDING, x - PADDING, x + w + PADDING];
        let y = [y - TILE_SIZE - PADDING, y - PADDING, y + h + PADDING];
        let w = [TILE_SIZE, w + PADDING * 2.0, TILE_SIZE];
        let h = [TILE_SIZE, h + PADDING * 2.0, TILE_SIZE];
        let tx = [tx, tx + tw, tx + tw * 2.0];
        let ty = [ty, ty + th, ty + th * 2.0];
        for i in 0..9 {
            let xi = i % 3;
            let yi = i / 3;
            renderer::draw_quad(x[xi], y[yi], w[xi], h[yi], 0.0, tx[xi], ty[yi], tw, th, 0);
        }
    }

    renderer::queue_text(x, y + 14.0, 0.0, 16.0, text);
}
