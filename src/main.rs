mod fuzzer;
mod gui;
mod constants;
use crate::gui::Gui;

fn main() -> Result<(), Box<dyn std::error::Error>>{
    let mut terminal = ratatui::init();
    terminal.clear()?;
    let app_result = Gui::default().run(&mut terminal);
    ratatui::restore();
    app_result
}
