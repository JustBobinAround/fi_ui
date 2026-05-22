use super::TuiComponent;
use crate::point::{Bounds, Rect, Vec2};
use crate::terminal::Terminal;

#[derive(Debug)]
pub enum SplitDir {
    Vertical,
    Horizontal,
}
impl Default for SplitDir {
    fn default() -> Self {
        SplitDir::Vertical
    }
}

pub struct SplitWindow<AppState> {
    split_dir: SplitDir,
    ratio_sum: usize,
    entries: Vec<(usize, usize, Box<dyn TuiComponent<AppState>>)>,
}

impl<AppState> SplitWindow<AppState> {
    pub fn new() -> Self {
        SplitWindow {
            split_dir: SplitDir::Vertical,
            ratio_sum: 0,
            entries: Vec::new(),
        }
    }
    pub fn with_component(
        mut self,
        window_ratio: usize,
        component: impl TuiComponent<AppState> + 'static,
    ) -> Self {
        self.entries
            .push((self.ratio_sum, window_ratio, Box::new(component)));
        self.ratio_sum += window_ratio;
        self
    }

    pub fn with_direction(mut self, split_dir: SplitDir) -> Self {
        self.split_dir = split_dir;
        self
    }
}

impl<AppState> TuiComponent<AppState> for SplitWindow<AppState> {
    fn render(&mut self, terminal: &mut Terminal, bounds: &Bounds, app_state: &AppState) {
        match self.split_dir {
            SplitDir::Vertical => {
                let bound = bounds.width();
                let mut delta = Bounds::new(0, 0);
                for (ratio_sum, window_ratio, entry) in self.entries.iter_mut() {
                    let x = (*ratio_sum * *bound) / self.ratio_sum;
                    delta = Bounds::new(x, 0) - delta;
                    terminal.move_relative(delta);
                    terminal.push_cursor_stack();
                    let window_size_x = (*window_ratio * *bound) / self.ratio_sum;
                    entry.render(
                        terminal,
                        &Bounds::new(window_size_x, *bounds.height()),
                        app_state,
                    );
                    terminal.pop_cursor_stack();
                }
            }
            SplitDir::Horizontal => {
                let bound = bounds.height();
                let mut delta = Bounds::new(0, 0);
                for (ratio_sum, window_ratio, entry) in self.entries.iter_mut() {
                    let y = (*ratio_sum * *bound) / self.ratio_sum;
                    delta = Bounds::new(0, y) - delta;
                    terminal.move_relative(delta);
                    terminal.push_cursor_stack();
                    let window_size_y = (*window_ratio * *bound) / self.ratio_sum;
                    entry.render(
                        terminal,
                        &Bounds::new(*bounds.width(), window_size_y),
                        app_state,
                    );
                    terminal.pop_cursor_stack();
                }
            }
        }
    }
}
