mod gap_buffer;
mod selection_tree;

use super::{
    TuiComponent,
    char_canvas::{CharEntry, Props},
};
use crate::{
    point::{Bounds, Vec2},
    terminal::Terminal,
};

pub use gap_buffer::GapBuffer;
pub use selection_tree::SelectionTree;

pub struct TextEditorBuilder<AppState> {
    use_buf: Option<fn(&AppState) -> &GapBuffer>,
    buf_offset: Option<fn(&AppState) -> usize>,
    line_counter: Option<fn(&AppState, usize, &mut [char; 6])>,
    cursor: Option<fn(&AppState) -> &SelectionTree>,
}

impl<AppState> TextEditorBuilder<AppState> {
    pub fn new() -> Self {
        Self {
            use_buf: None,
            buf_offset: None,
            line_counter: None,
            cursor: None,
        }
    }

    pub fn use_buffer(mut self, f: fn(&AppState) -> &GapBuffer) -> Self {
        self.use_buf = Some(f);
        self
    }

    pub fn with_buffer_offset(mut self, f: fn(&AppState) -> usize) -> Self {
        self.buf_offset = Some(f);
        self
    }

    pub fn with_line_counter(mut self, f: fn(&AppState, usize, &mut [char; 6])) -> Self {
        self.line_counter = Some(f);
        self
    }

    pub fn with_cursor(mut self, f: fn(&AppState) -> &SelectionTree) -> Self {
        self.cursor = Some(f);
        self
    }

    pub fn finalize(self) -> TextEditor<AppState> {
        let use_buf = self.use_buf.unwrap_or(|_| &gap_buffer::EMPTY_BUF);
        let buf_offset = self.buf_offset.unwrap_or(|_| 0);
        let line_counter = self.line_counter.unwrap_or(|_, _, _| {});
        let cursor = self
            .cursor
            .unwrap_or(|_| &selection_tree::EMPTY_SELECTION_TREE);

        TextEditor {
            use_buf,
            buf_offset,
            line_counter,
            cursor,
            position: Vec2::default(),
            coord: Vec2::default(),
            delta: Vec2::default(),
            line_offset: 0,
        }
    }
}

pub enum TextEditMode {
    Normal,
    Insert,
    Visual,
    VisualLine,
}

pub struct TextEditor<AppState> {
    use_buf: fn(&AppState) -> &GapBuffer,
    buf_offset: fn(&AppState) -> usize,
    line_counter: fn(&AppState, usize, &mut [char; 6]),
    cursor: fn(&AppState) -> &SelectionTree,
    position: Vec2,
    coord: Vec2,
    delta: Vec2,
    line_offset: usize,
}

impl<AppState> TextEditor<AppState> {
    fn reset_coords(&mut self, terminal: &mut Terminal) {
        self.position = Vec2::default();
        self.delta = Vec2::default();
        terminal.move_relative(self.delta);
        self.coord = Vec2::default();
        self.line_offset = 0;
    }

    fn update_term(&mut self, terminal: &mut Terminal, bounds: &Bounds, ce: impl Into<CharEntry>) {
        if self.coord.is_within_bounds(bounds) {
            let delta = self.coord - self.position;
            self.position += delta;
            terminal.move_relative(delta);
            terminal.set_entry(ce.into());
        }
        self.coord += Vec2::new(1, 0);
    }

    fn render_line_counter(
        &mut self,
        terminal: &mut Terminal,
        bounds: &Bounds,
        app_state: &AppState,
        line_count: &mut [char; 6],
    ) {
        self.coord.set_x(0);
        (self.line_counter)(app_state, self.line_offset, line_count);
        for c in line_count.iter() {
            self.update_term(terminal, bounds, *c as char);
        }
        self.coord.set_x(7);
    }

    fn handle_wrapping(&mut self, terminal: &mut Terminal, bounds: &Bounds) {
        let wrap_bound = *bounds.width() as isize;
        if self.coord.x() >= &wrap_bound {
            self.coord += Vec2::new(0, 1);
            self.coord.set_x(0);
            for _ in 0..5 {
                self.update_term(terminal, bounds, ' ');
            }
            self.update_term(terminal, bounds, '↪');
            self.coord.set_x(7);
        }
    }

    const INSERT_CURSOR: CharEntry =
        CharEntry::new(' ').with_props(Props::new().with_reverse_video());

    fn render_insert_mode(
        &mut self,
        terminal: &mut Terminal,
        bounds: &Bounds,
        app_state: &AppState,
    ) {
        let buf = (self.use_buf)(app_state);
        let offset = (self.buf_offset)(app_state);
        self.reset_coords(terminal);

        let mut line_count = [' '; 6];

        self.render_line_counter(terminal, bounds, app_state, &mut line_count);

        let vert_bound = *bounds.height() as isize / 2;
        let cursor = (self.cursor)(app_state);

        for (i, c) in buf.iter_from(offset..).enumerate() {
            let i = i + offset;

            if c == &'\n' {
                self.line_offset += 1;
                self.coord += Vec2::new(0, 1);
                self.render_line_counter(terminal, bounds, app_state, &mut line_count);
            } else {
                self.handle_wrapping(terminal, bounds);
                if cursor.contains(i.saturating_sub(1)) {
                    // eprintln!("hit: {}", i);
                    let cursor: CharEntry =
                        CharEntry::new(*c).with_props(Props::new().with_reverse_video());
                    self.update_term(terminal, bounds, cursor);
                } else {
                    self.update_term(terminal, bounds, *c);
                }
            }

            if self.coord.y() - 1 >= vert_bound {
                break;
            }
        }

        if cursor.contains(buf.len()) {
            self.handle_wrapping(terminal, bounds);
            self.update_term(terminal, bounds, Self::INSERT_CURSOR);
        }
    }
}

impl<AppState> TuiComponent<AppState> for TextEditor<AppState> {
    fn render(&mut self, terminal: &mut Terminal, bounds: &Bounds, app_state: &AppState) {
        self.render_insert_mode(terminal, bounds, app_state);
    }
}
