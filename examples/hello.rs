extern crate fi_ui;
use fi_ui::prelude::*;

struct AppState {
    buf: GapBuffer,
    should_run: bool,
    buf_offset: usize,
    line_offset: usize,
    cursor: SelectionTree,
}
fn main() -> TerminalRes<()> {
    // let canvas = CharCanvas::new().on_render(|bounds, this, app_state: &AppState| {
    //     let props = Props::new().with_bold().with_underline();

    //     let virt_cursor_props = Props::new().with_reverse_video();
    //     let mut y = 0;
    //     let mut x = 0;
    //     this.clear();
    //     for c in app_state.buf.iter() {
    //         if c == &'\n' {
    //             y += 1;
    //             x = 0;
    //         } else {
    //             let entry = CharEntry::new(*c as char).with_props(props);
    //             this.set_entry(x, y, entry);
    //             x += 1;
    //         }
    //     }
    //     let entry = CharEntry::new(' ').with_props(virt_cursor_props.clone());
    //     this.set_entry(x, y, entry);
    // });
    // let canvas2 = CharCanvas::new().on_render(|bounds, this, _| {
    //     let props = Props::new().with_bold().with_underline();
    //     for y in 0..*bounds.height() as isize {
    //         for x in y..*bounds.width() as isize {
    //             this.set_entry(
    //                 x as isize,
    //                 y as isize,
    //                 CharEntry::new(((((x % (y + 1)) % 26) as u8) + 65) as char)
    //                     .with_props(props.clone()),
    //             );
    //         }
    //     }
    // });

    let editor = TextEditorBuilder::<AppState>::new()
        .use_buffer(|app_state| &app_state.buf)
        .with_line_counter(|app_state, buf_offset: usize, line_count: &mut [char; 6]| {
            let line_count_num = app_state.line_offset + buf_offset;
            let line_count_str = line_count_num.to_string();
            let offset = line_count.len() - line_count_str.len();
            for (mut i, c) in line_count_str.chars().enumerate() {
                i += offset - 1;
                if i >= line_count.len() {
                    break;
                }
                line_count[i] = c;
            }
        })
        .with_cursor(|app_state| &app_state.cursor)
        .finalize();
    let mut cursor = SelectionTree::new();
    cursor.insert(0, 0);

    let app_state = AppState {
        buf: GapBuffer::new(),
        should_run: true,
        buf_offset: 0usize,
        line_offset: 0usize,
        cursor,
    };

    // let main_component = SplitWindow::new()
    //     .with_component(1, canvas2)
    //     .with_component(1, editor)
    //     .with_component(1, canvas)
    //     .with_direction(SplitDir::Vertical);

    TerminalAppBuilder::new(app_state, editor)
        .run_while(|app_state| app_state.should_run)
        .on_input_event(|app_state, input_event| {
            match input_event {
                InputEvent::Key { ctrl, key } if key == &'q' && *ctrl => {
                    app_state.should_run = false;
                }
                InputEvent::Key { ctrl, key } if key == &'h' && *ctrl => {
                    app_state.cursor.translate(-1);
                }
                InputEvent::Key { ctrl, key }
                    if key == &'j' && *ctrl && app_state.buf_offset < app_state.buf.len() =>
                {
                    let buf = app_state.buf.iter_from(app_state.buf_offset..);

                    for (i, c) in buf.enumerate() {
                        if c == &'\n' {
                            app_state.buf_offset += i + 1;
                            app_state.line_offset += 1;
                            break;
                        }
                    }
                }
                InputEvent::Key { ctrl, key }
                    if key == &'k' && *ctrl && app_state.buf_offset > 0 =>
                {
                    let buf = app_state.buf.iter_to(..app_state.buf_offset);

                    let mut found = false;
                    for (i, c) in buf.rev().enumerate() {
                        if c == &'\n' && i != 0 {
                            app_state.buf_offset -= i;
                            found = true;
                            app_state.line_offset -= 1;
                            break;
                        }
                    }
                    if !found {
                        app_state.line_offset -= 1;
                        app_state.buf_offset = 0;
                    }
                }
                InputEvent::Key { ctrl, key } if !ctrl => {
                    app_state.buf.insert(app_state.cursor.main_cursor(), *key);
                    app_state.cursor.translate(1);
                }
                InputEvent::Enter => {
                    app_state.buf.insert(app_state.cursor.main_cursor(), '\n');
                    app_state.cursor.translate(1);
                }
                InputEvent::Backspace => {
                    // app_state.buf.remove(app_state.cursor);
                }
                _ => {}
            }

            Ok(())
        })
        .on_update(|app_state| {
            // if app_state.1 {
            //     app_state.buf += 1;
            // }
            Ok(())
        })
        .only_render_on_input_event()
        .finalize()
        .run()?;

    Ok(())
}
