use crate::point::{Bounds, Vec2};
use crate::prelude::{CharEntry, Terminal, TuiComponent};

pub enum BarDirection {
    North,
    South,
    East,
    West,
}

pub struct LoadingBar<AppState> {
    calc_progress: fn(&AppState) -> f32,
    bar_direction: fn(&AppState) -> BarDirection,
}

impl<AppState> LoadingBar<AppState> {
    pub fn new() -> Self {
        LoadingBar {
            calc_progress: |_| 0.0,
            bar_direction: |_| BarDirection::East,
        }
    }

    pub fn with_progress_calculator(mut self, calc_progress: fn(&AppState) -> f32) -> Self {
        self.calc_progress = calc_progress;
        self
    }

    pub fn with_bar_direction(mut self, bar_direction: fn(&AppState) -> BarDirection) -> Self {
        self.bar_direction = bar_direction;
        self
    }
}

impl<AppState> TuiComponent<AppState> for LoadingBar<AppState> {
    fn render(&mut self, terminal: &mut Terminal, bounds: &Bounds, app_state: &AppState) {
        let progress = (self.calc_progress)(app_state) / 100.0;
        let progress_str = format!("{:.2}/100.00", progress * 100.0);
        let bar_length = *bounds.width() as f32 * progress.min(100.0);
        let progress_str_offset = (bounds.width() / 2) - (progress_str.len() / 2);
        for y in 0..*bounds.height() {
            terminal.push_cursor_stack();
            if y == bounds.height() / 2 {
                terminal.push_cursor_stack();
                for _ in 0..bar_length as usize {
                    terminal.set_entry(CharEntry::new('█'));
                    terminal.move_relative(Vec2::new(1, 0));
                }
                terminal.pop_cursor_stack();
                terminal.push_cursor_stack();
                terminal.move_relative(Vec2::new(progress_str_offset as isize, 0));
                for c in progress_str.chars() {
                    terminal.set_entry(CharEntry::new(c));
                    terminal.move_relative(Vec2::new(1, 0));
                }
                terminal.pop_cursor_stack();
            } else {
                for _ in 0..bar_length as usize {
                    terminal.set_entry(CharEntry::new('█'));
                    terminal.move_relative(Vec2::new(1, 0));
                }
            }
            terminal.pop_cursor_stack();
            terminal.move_relative(Vec2::new(0, 1));
        }
    }
}
