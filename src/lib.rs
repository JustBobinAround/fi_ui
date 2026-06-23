mod components;
mod escapes;
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
