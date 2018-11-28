pub mod element;
pub mod keyboard;

pub use glutin::{ModifiersState, VirtualKeyCode};
use std::error::Error;

use clip;
use renderer::{self, Renderer};
use std::collections::HashMap;
use text::{TextCursor, TextRenderer};

use self::element::{UIElement, UIElementKind};
use self::keyboard::KeyStatus;
use self::keyboard::Keyboard;

// TODO: Move these two consts to styles when those exist
const NINEPATCH_TILE_SIZES: ((f32, f32, f32), (f32, f32, f32)) = ((4.0, 4.0, 4.0), (4.0, 4.0, 4.0));
const PADDING: f32 = 6.0;

const NORMAL_UI_ELEMENT_DEPTH: f32 = 0.0;
const NORMAL_UI_TEXT_DEPTH: f32 = NORMAL_UI_ELEMENT_DEPTH - 0.1;

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

/// The state of the UI.
pub struct UIState {
    elements: HashMap<u64, UIElement>,
    last_element: Option<UIElement>,
    mouse: MouseStatus,
    pressed_element: Option<u64>,
    focused_element: Option<u64>,
    hovering: bool,
    window_dimensions: (f32, f32),
    keyboard: Keyboard,
    text_renderer: TextRenderer,
    /// The renderer that renders the UI on the screen.
    pub renderer: Renderer,
}

impl UIState {
    /// Creates a new instance of the UI.
    pub fn create(
        font_data: Vec<u8>,
        ui_spritesheet_data: &[u8],
        opengl21: bool,
    ) -> Result<UIState, Box<Error>> {
        Ok(UIState {
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
            keyboard: Keyboard::new(),
            window_dimensions: (0.0, 0.0),
            text_renderer: TextRenderer::create(font_data)?,
            renderer: Renderer::create(opengl21, ui_spritesheet_data)?,
        })
    }

    fn insert_element(&mut self, element: UIElement) {
        self.last_element = Some(element.clone());
        self.elements.insert(element.id(), element);
    }

    /// Handled by the `window_bootstrap` feature, if in use. Renders
    /// the UI.
    pub fn update_post_application(&mut self, width: f32, height: f32) {
        let renderer = &mut self.renderer;
        let text_renderer = &mut self.text_renderer;
        text_renderer.draw_text(renderer);
        renderer.render(width, height);
    }

    /// Handled by the `window_bootstrap` feature, if in use. Updates
    /// the UI state and reads current system status.
    // TODO: Keyboard navigation of the UI
    pub fn update(
        &mut self,
        width: f32,
        height: f32,
        dpi: f32,
        mouse: MouseStatus,
        key_inputs: Vec<KeyStatus>,
        characters: Vec<char>,
    ) -> UIStatus {
        self.window_dimensions = (width as f32, height as f32);
        self.text_renderer.update_dpi(dpi);

        let hovering_button;
        let focused_element;
        if !self.mouse.pressed {
            self.pressed_element = None;
        }
        self.elements.clear();
        self.last_element = None;
        self.hovering = false;

        self.mouse = mouse;
        hovering_button = self.hovering;
        self.keyboard.update(key_inputs);
        focused_element = self.focused_element;

        if let Some(id) = focused_element {
            for character in characters {
                element::insert_input(id, character);
            }

            if self.keyboard.typed(
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

            let mut delta = 0;
            if self.keyboard.typed(VirtualKeyCode::Right, None) {
                delta += 1;
            }
            if self.keyboard.typed(VirtualKeyCode::Left, None) {
                delta -= 1;
            }
            element::move_cursor(self, delta);
        }

        UIStatus { hovering_button }
    }

    fn draw_element(
        &mut self,
        element: &UIElement,
        text: &str,
        multiline: bool,
        cursor: Option<&mut TextCursor>,
    ) {
        let &UIElement {
            kind,
            rect,
            alignment,
            ..
        } = element;
        let (x0, y0, x1, y1) = rect.coords();

        if kind != UIElementKind::NoBackground {
            let sheet_length = UIElementKind::KindCount as i32;
            let tx0 = kind as i32 as f32 / sheet_length as f32;
            let ty0 = 0.0;
            let tx1 = tx0 + 1.0 / (sheet_length as f32);
            let ty1 = ty0 + 1.0;
            let coords = (x0 - PADDING, y0 - PADDING, x1 + PADDING, y1 + PADDING);

            self.renderer.draw_quad_ninepatch(
                NINEPATCH_TILE_SIZES,
                coords,
                (tx0, ty0, tx1, ty1),
                (0xFF, 0xFF, 0xFF, 0xFF),
                NORMAL_UI_ELEMENT_DEPTH,
                renderer::DRAW_CALL_INDEX_UI,
            );
        }

        let coords = (x0, y0, NORMAL_UI_TEXT_DEPTH);
        let dims = rect.dimensions();
        self.text_renderer
            .queue_text(text, coords, dims, alignment, multiline, cursor);
    }
}
