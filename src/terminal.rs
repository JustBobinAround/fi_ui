mod events;
mod lib_c;
mod raw_mode;
mod terminal_dims;

pub use events::InputEvent;

use crate::{
    components::{CharEntry, TuiComponent},
    escapes::{EscapeWriter, TerminalRequest},
    point::{Bounds, Vec2},
};
use std::{
    collections::{BTreeMap, VecDeque},
    io::{BufReader, BufWriter, Read},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, PoisonError, RwLock,
    },
    thread::JoinHandle,
    time::{Duration, Instant},
};
use terminal_dims::{TermSizeErr, TerminalDims, TERM_SIZE_CHANGED};

const FRAME_RATE: Duration = Duration::from_nanos(16_666_667);
const SLEEP_TIME: Duration = Duration::from_nanos(16_666_667 / 2);

impl From<TermSizeErr> for TerminalErr {
    fn from(value: TermSizeErr) -> Self {
        Self::TermSizeErr(value)
    }
}

impl From<std::io::Error> for TerminalErr {
    fn from(value: std::io::Error) -> Self {
        Self::IoErr(value)
    }
}

impl<T> From<PoisonError<T>> for TerminalErr {
    fn from(value: PoisonError<T>) -> Self {
        Self::PoisonErr(format!("{:#?}", value))
    }
}

#[derive(Debug)]
pub enum TerminalErr {
    TermSizeErr(TermSizeErr),
    IoErr(std::io::Error),
    FailedToLockWriter,
    PoisonErr(String),
}

// &mut self,
// writer: &mut W,
// component: &mut impl TuiComponent<T>,
// app_state: &T,
pub struct TerminalAppBuilder<AppState, MainComponent: TuiComponent<AppState>> {
    app_state: AppState,
    main_component: MainComponent,
    on_input_event: Option<fn(&mut AppState, &InputEvent) -> TerminalRes<()>>,
    on_update: Option<fn(&mut AppState) -> TerminalRes<()>>,
    should_render: Option<fn(&AppState, &InputEvent) -> bool>,
    run_while: Option<fn(&AppState) -> bool>,
}

pub struct TerminalApp<AppState, MainComponent: TuiComponent<AppState>> {
    app_state: AppState,
    main_component: MainComponent,
    on_input_event: fn(&mut AppState, &InputEvent) -> TerminalRes<()>,
    on_update: fn(&mut AppState) -> TerminalRes<()>,
    should_render: fn(&AppState, &InputEvent) -> bool,
    run_while: fn(&AppState) -> bool,
}
impl<AppState, MainComponent: TuiComponent<AppState>> TerminalApp<AppState, MainComponent> {
    pub fn run(mut self) -> TerminalRes<()> {
        let mut writer = std::io::stdout().lock();
        let mut terminal = Terminal::new()?;
        let mut last_time = Instant::now();

        terminal.enter_alt_screen(&mut writer)?;
        terminal.enable_raw_mode()?;
        terminal.hide_cursor(&mut writer)?;
        let run_while = self.run_while;
        let on_update = self.on_update;
        let on_input_event = self.on_input_event;

        on_update(&mut self.app_state)?;
        terminal.draw(&mut writer, &mut self.main_component, &self.app_state)?;

        while run_while(&self.app_state) {
            let mut should_render = false;

            while Instant::now().saturating_duration_since(last_time) >= FRAME_RATE {
                let input_event = terminal.next_input_event()?;
                if input_event.is_some() {
                    on_input_event(&mut self.app_state, &input_event)?;
                }
                on_update(&mut self.app_state)?;
                should_render |= (self.should_render)(&self.app_state, &input_event);
                last_time += FRAME_RATE;
            }

            if should_render {
                terminal.draw(&mut writer, &mut self.main_component, &self.app_state)?;
            }

            std::thread::sleep(SLEEP_TIME);
        }

        terminal.show_cursor(&mut writer)?;
        terminal.disable_raw_mode()?;
        terminal.exit_alt_screen(&mut writer)?;

        Ok(())
    }
}

impl<AppState, MainComponent: TuiComponent<AppState>> TerminalAppBuilder<AppState, MainComponent> {
    pub fn new(app_state: AppState, main_component: MainComponent) -> Self {
        Self {
            app_state,
            main_component,
            on_input_event: None,
            on_update: None,
            should_render: None,
            run_while: None,
        }
    }

    pub fn on_input_event(mut self, f: fn(&mut AppState, &InputEvent) -> TerminalRes<()>) -> Self {
        self.on_input_event = Some(f);
        self
    }

    pub fn only_render_on_input_event(mut self) -> Self {
        self.should_render = Some(|_, input_event| input_event.is_some());
        self
    }

    pub fn only_render_when(mut self, f: fn(&AppState, &InputEvent) -> bool) -> Self {
        self.should_render = Some(f);
        self
    }

    pub fn on_update(mut self, f: fn(&mut AppState) -> TerminalRes<()>) -> Self {
        self.on_update = Some(f);
        self
    }

    pub fn run_while(mut self, f: fn(&AppState) -> bool) -> Self {
        self.run_while = Some(f);
        self
    }

    pub fn finalize(mut self) -> TerminalApp<AppState, MainComponent> {
        let app_state = self.app_state;
        let main_component = self.main_component;
        let on_input_event: fn(&mut AppState, &InputEvent) -> TerminalRes<()> =
            self.on_input_event.take().unwrap_or(|_, _| Ok(()));
        let on_update: fn(&mut AppState) -> TerminalRes<()> =
            self.on_update.take().unwrap_or(|_| Ok(()));
        let should_render: fn(&AppState, &InputEvent) -> bool =
            self.should_render.take().unwrap_or(|_, _| true);
        let run_while: fn(&AppState) -> bool = self.run_while.take().unwrap_or(|_| true);
        TerminalApp {
            app_state,
            main_component,
            on_input_event,
            on_update,
            should_render,
            run_while,
        }
    }
}

pub type TerminalRes<T> = Result<T, TerminalErr>;

#[derive(Debug)]
pub struct Terminal {
    dimensions: TerminalDims,
    virt_cursor: Vec2,
    cursor_stack: Vec<Vec2>,
    buffer_a: BTreeMap<Vec2, CharEntry>,
    buffer_b: BTreeMap<Vec2, CharEntry>,
    read_buf: Arc<RwLock<VecDeque<u8>>>,
    is_running: Arc<AtomicBool>,
    stdin_handle: Option<JoinHandle<()>>,
}

// const FRAME_TIME: Duration = Duration::from_nanos(16_666_667);
// let mut last_time = Instant::now();
// while !rl.window_should_close() {
//     while Instant::now().saturating_duration_since(last_time) >= FRAME_TIME {
//         game.update(&mut player, &mut world);
//         last_time += FRAME_TIME;
//     }

impl Drop for Terminal {
    fn drop(&mut self) {
        self.is_running.swap(false, Ordering::Relaxed);
        match self.stdin_handle.take() {
            Some(handle) => {
                for _ in 0..5 {
                    if handle.is_finished() {
                        let _ = handle.join();
                        break;
                    } else {
                        std::thread::sleep(FRAME_RATE);
                    }
                }
            }
            None => {}
        }
    }
}
impl Terminal {
    pub fn new() -> TerminalRes<Self> {
        let dimensions = TerminalDims::new()?;
        let virt_cursor = Vec2::default();
        let cursor_stack = Vec::new();
        let buffer_a = BTreeMap::new();
        let buffer_b = BTreeMap::new();
        let read_buf = Arc::new(RwLock::new(VecDeque::with_capacity(100)));
        let read_buf_2 = read_buf.clone();
        let is_running = Arc::new(AtomicBool::new(true));
        let is_running_2 = is_running.clone();
        let stdin_handle = Some(std::thread::spawn(move || {
            let stdin = std::io::stdin().lock();
            let mut buf_reader = BufReader::new(stdin);
            let mut buf = [0; 1];
            while is_running_2.load(Ordering::Relaxed) {
                match buf_reader.read(&mut buf) {
                    Ok(bytes_read) if bytes_read > 0 => match read_buf_2.write() {
                        Ok(mut a) => {
                            a.push_back(buf[0]);
                        }
                        Err(_) => {
                            break;
                        }
                    },
                    Ok(_) => {}
                    Err(_) => {
                        break;
                    }
                }
            }
        }));

        Ok(Terminal {
            dimensions,
            virt_cursor,
            cursor_stack,
            buffer_a,
            buffer_b,
            read_buf,
            is_running,
            stdin_handle,
        })
    }

    pub fn width(&self) -> usize {
        self.dimensions.cols as usize
    }

    pub fn height(&self) -> usize {
        self.dimensions.rows as usize
    }

    pub fn bounds(&self) -> Bounds {
        Bounds::new(self.width(), self.height())
    }

    pub fn set_entry(&mut self, entry: CharEntry) {
        self.buffer_a.insert(self.virt_cursor, entry);
    }

    pub fn move_relative(&mut self, delta: impl Into<Vec2>) {
        self.virt_cursor += delta.into();
    }

    pub fn push_cursor_stack(&mut self) {
        self.cursor_stack.push(self.virt_cursor);
    }

    pub fn pop_cursor_stack(&mut self) {
        match self.cursor_stack.pop() {
            Some(c) => {
                self.virt_cursor = c;
            }
            None => {}
        }
    }

    pub fn cursor_is_within_term_bounds(&self) -> bool {
        self.virt_cursor.x() >= &0
            && self.virt_cursor.y() >= &0
            && self.virt_cursor.x() < &(self.width() as isize)
            && self.virt_cursor.y() < &(self.height() as isize)
    }

    fn sync(&mut self) {
        self.virt_cursor = Vec2::default();
        self.buffer_a.clear();
    }

    pub fn hide_cursor<W: std::io::Write>(&mut self, writer: &mut W) -> TerminalRes<()> {
        let mut ew = EscapeWriter::new(writer);
        ew.handle_term_request(&TerminalRequest::DisableShowCursor)?;
        writer.flush()?;
        Ok(())
    }

    pub fn show_cursor<W: std::io::Write>(&mut self, writer: &mut W) -> TerminalRes<()> {
        let mut ew = EscapeWriter::new(writer);
        ew.handle_term_request(&TerminalRequest::EnableShowCursor)?;
        writer.flush()?;
        Ok(())
    }

    pub fn enable_raw_mode(&self) -> TerminalRes<()> {
        crate::terminal::raw_mode::enable_raw_mode()?;
        Ok(())
    }

    pub fn disable_raw_mode(&self) -> TerminalRes<()> {
        crate::terminal::raw_mode::disable_raw_mode()?;
        Ok(())
    }

    pub fn enter_alt_screen<W: std::io::Write>(&mut self, writer: &mut W) -> TerminalRes<()> {
        let mut ew = EscapeWriter::new(writer);
        ew.handle_term_request(&TerminalRequest::EnableAlternateScreenBufferCursorSaveOrRestore)?;
        writer.flush()?;
        Ok(())
    }

    pub fn exit_alt_screen<W: std::io::Write>(&mut self, writer: &mut W) -> TerminalRes<()> {
        let mut ew = EscapeWriter::new(writer);
        ew.handle_term_request(&TerminalRequest::DisableAlternateScreenBufferCursorSaveOrRestore)?;
        writer.flush()?;
        Ok(())
    }

    pub fn next_input_event(&mut self) -> TerminalRes<&'static InputEvent> {
        if self.read_buf.read().is_ok_and(|bytes| bytes.len() > 0) {
            let mut read_buf = self.read_buf.write()?;

            if let Some(b) = read_buf.pop_front() {
                Ok(InputEvent::from_raw(b))
            } else {
                Ok(&InputEvent::None)
            }
        } else {
            Ok(&InputEvent::None)
        }
    }

    pub fn draw<W: std::io::Write, T>(
        &mut self,
        writer: &mut W,
        component: &mut impl TuiComponent<T>,
        app_state: &T,
    ) -> TerminalRes<()> {
        self.sync();
        if TERM_SIZE_CHANGED.swap(false, Ordering::Relaxed) {
            self.dimensions.update_screen_size()?;
        }

        component.render(self, &self.bounds(), &app_state);

        let mut buf = BufWriter::new(writer);
        let mut ew = EscapeWriter::new(&mut buf);
        let first_coord = Vec2::new(0, 0);

        let mut last_coord = &first_coord;
        ew.goto_coord(&last_coord)?;
        ew.flush()?;
        for (coord, entry) in &self.buffer_a {
            self.buffer_b.remove(coord);
            if !coord.is_to_the_right_of(last_coord) {
                ew.goto_coord(coord)?;
            }
            entry.draw(&mut ew)?;
            last_coord = coord;
        }

        let clear_entry = CharEntry::new(' ');
        for (coord, _entry) in &self.buffer_b {
            if !coord.is_to_the_right_of(last_coord) {
                ew.goto_coord(coord)?;
            }
            clear_entry.draw(&mut ew)?;
        }
        ew.flush()?;

        std::mem::swap(&mut self.buffer_a, &mut self.buffer_b);
        Ok(())
    }
}
