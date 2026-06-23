extern crate fi_ui;
use fi_ui::prelude::*;

struct AppState {
    loading: f32,
    should_run: bool,
}
fn main() -> TerminalRes<()> {
    let loading_bar =
        LoadingBar::new().with_progress_calculator(|app_state: &AppState| app_state.loading);
    let loading_bar_2 =
        LoadingBar::new().with_progress_calculator(|app_state: &AppState| app_state.loading / 2.0);

    let app_state = AppState {
        loading: 0.0,
        should_run: true,
    };

    let main_component = SplitWindow::new()
        .with_component(1, loading_bar_2)
        .with_component(1, loading_bar);
    //     .with_component(1, canvas)
    //     .with_direction(SplitDir::Vertical);

    TerminalAppBuilder::new(app_state, main_component)
        .run_while(|app_state| app_state.should_run)
        .on_input_event(|app_state, input_event| {
            match input_event {
                InputEvent::Key { ctrl, key } if key == &'q' && *ctrl => {
                    app_state.should_run = false;
                }
                _ => {}
            }

            Ok(())
        })
        .on_update(|app_state| {
            if app_state.loading > 100.0 {
                app_state.loading = 0.0;
            }
            app_state.loading += 0.1;
            Ok(())
        })
        .finalize()
        .run()?;

    Ok(())
}
