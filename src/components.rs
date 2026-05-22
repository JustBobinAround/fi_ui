mod char_canvas;
mod split_window;
mod text_editor;

use crate::point::Bounds;
use crate::terminal::Terminal;

pub use char_canvas::{CharCanvas, CharEntry, Props};
pub use split_window::{SplitDir, SplitWindow};
pub use text_editor::TextEditorBuilder;

pub trait TuiComponent<T> {
    fn render(&mut self, terminal: &mut Terminal, bounds: &Bounds, app_state: &T);
}

impl<T> TuiComponent<T> for () {
    fn render(&mut self, _terminal: &mut Terminal, _bounds: &Bounds, _app_state: &T) {}
}
