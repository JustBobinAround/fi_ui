mod components;
mod escapes;
mod gap_buffer;
mod point;
mod terminal;

pub mod prelude {
    pub use crate::{
        components::*,
        terminal::{
            InputEvent, Terminal, TerminalApp, TerminalAppBuilder, TerminalErr, TerminalRes,
        },
    };
}
