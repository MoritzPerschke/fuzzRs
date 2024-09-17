mod fuzzer;
mod gui;
mod constants;

use std::env;
use std::io::{ self };
use crate::gui::Gui;

fn main() -> io::Result<()>{
    let mut terminal = ratatui::init();
    terminal.clear()?;
    let app_result = Gui::default().run(&mut terminal);
    ratatui::restore();
    app_result

}
