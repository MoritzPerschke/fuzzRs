mod fuzzer;
mod gui;
mod constants;

use std::env;
use std::io::{ self };
use crate::gui::Gui;

fn main() -> io::Result<()>{
    let args: Vec<String> = env::args().collect();

    let wordlist = &args[1];
    let host = &args[2];

    let mut terminal = ratatui::init();
    terminal.clear()?;
    let app_result = Gui::default().run(&mut terminal, wordlist, host);
    ratatui::restore();
    app_result

}
