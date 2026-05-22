use super::TuiComponent;
use crate::{
    escapes::{EscapeWriter, TerminalRequest},
    point::{Bounds, Vec2},
    terminal::Terminal,
};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Props {
    flags: u32,
    r: u8,
    g: u8,
    b: u8,
}

impl Props {
    const BOLD_MASK: u32 = 0b0000_00000_0000_0000_0000_0000_0000_0001;
    const UNDERLINE_MASK: u32 = 0b0000_00000_0000_0000_0000_0000_0000_0010;
    const REVERSE_VIDEO_MASK: u32 = 0b0000_00000_0000_0000_0000_0000_0000_0100;

    pub const fn new() -> Self {
        Self {
            flags: 0,
            r: 255,
            g: 255,
            b: 255,
        }
    }

    fn has_flag(&self, mask: u32) -> bool {
        (self.flags & mask) != 0
    }

    pub const fn with_bold(mut self) -> Self {
        self.flags |= Self::BOLD_MASK;
        self
    }

    pub const fn with_underline(mut self) -> Self {
        self.flags = Self::UNDERLINE_MASK;
        self
    }
    pub const fn with_reverse_video(mut self) -> Self {
        self.flags = Self::REVERSE_VIDEO_MASK;
        self
    }

    pub fn apply_props<'b, W: std::io::Write>(
        &self,
        ew: &mut EscapeWriter<W>,
    ) -> Result<(), std::io::Error> {
        if self.has_flag(Self::BOLD_MASK) {
            ew.handle_term_request(&TerminalRequest::Bold)?;
        }
        if self.has_flag(Self::UNDERLINE_MASK) {
            ew.handle_term_request(&TerminalRequest::Underline)?;
        }
        if self.has_flag(Self::REVERSE_VIDEO_MASK) {
            ew.handle_term_request(&TerminalRequest::ReverseVideo)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CharEntry {
    c: char,
    props: Props,
}

impl From<char> for CharEntry {
    fn from(value: char) -> CharEntry {
        CharEntry {
            c: value,
            props: Props::new(),
        }
    }
}

impl CharEntry {
    pub const fn new(c: char) -> Self {
        Self {
            c,
            props: Props::new(),
        }
    }

    pub fn set_char(&mut self, c: char) {
        self.c = c;
    }
    pub fn draw<W: std::io::Write>(&self, ew: &mut EscapeWriter<W>) -> std::io::Result<()> {
        ew.print_char_entry(&self)
    }
    pub fn inner_char(&self) -> char {
        self.c
    }

    pub const fn with_props(mut self, props: Props) -> Self {
        self.props = props;
        self
    }

    pub fn apply_props<'b, W: std::io::Write>(
        &self,
        ew: &mut EscapeWriter<W>,
    ) -> Result<(), std::io::Error> {
        self.props.apply_props(ew)
    }
}

#[derive(Debug)]
pub struct CharCanvas<AppState> {
    offset: Vec2,
    entries: HashMap<Vec2, CharEntry>,
    update_fn: Option<fn(&Bounds, &mut Self, &AppState)>,
}

impl<AppState> CharCanvas<AppState> {
    pub fn new() -> Self {
        Self {
            offset: Vec2::default(),
            entries: HashMap::new(),
            update_fn: None,
        }
    }

    pub fn on_render(
        mut self,
        update_fn: fn(&Bounds, &mut CharCanvas<AppState>, &AppState),
    ) -> Self {
        self.update_fn = Some(update_fn);
        self
    }

    pub fn set_entry(&mut self, x: isize, y: isize, entry: impl Into<CharEntry>) {
        let v = Vec2::new(x, y);
        self.entries.insert(v, entry.into());
    }

    pub fn set_offset(&mut self, x: isize, y: isize) {
        self.offset = Vec2::new(x, y);
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }
}

impl<AppState> TuiComponent<AppState> for CharCanvas<AppState> {
    fn render(&mut self, terminal: &mut Terminal, bounds: &Bounds, app_state: &AppState) {
        self.update_fn.take().map(|update_fn| {
            update_fn(bounds, self, app_state);
            self.update_fn = Some(update_fn);
        });

        let mut position = Vec2::default();
        let delta = self.offset - position;
        terminal.move_relative(delta);
        for (coord, entry) in &self.entries {
            if (*coord + self.offset).is_within_bounds(bounds) {
                let delta = *coord - position;
                position += delta;
                terminal.move_relative(delta);
                terminal.set_entry(entry.clone());
            }
        }
    }
}
