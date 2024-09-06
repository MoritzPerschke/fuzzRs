use std::collections::HashMap;
use std::sync::{ Arc, Mutex };
use std::io::{ self };

use reqwest::StatusCode;

use crossterm::event::{ self, Event, KeyCode, KeyEvent, KeyEventKind };
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{
        block::{ Position, Title },
        Block, Paragraph, Widget,
    },
    DefaultTerminal,
    Frame
};

use crate::fuzzer;

#[derive(Debug, Default)]
pub struct Gui {
    title: String,
    host: String,
    wordlist: String,
    query_results: Arc<Mutex<HashMap<String, StatusCode>>>,
    exit: bool,
}

impl Gui {
    pub fn run(&mut self, terminal: &mut DefaultTerminal, wordlist: &str, host: &str) -> io::Result<()> {
        self.title = " FuzzRs ".to_string();
        self.wordlist = wordlist.to_string();
        self.host = host.to_string();

        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }
    
    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area())
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('r') =>  {
                self.title = " FuzzRs (running...) ".to_string();
                fuzzer::fuzz(&self.wordlist, &self.host, &self.query_results);
            }
            KeyCode::Char('q') =>  self.exit(),
            _ => {}
        }
    }
    
    fn exit(&mut self) {
        self.exit = true;
    }
}

impl Widget for &Gui {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Title::from(self.title.clone().bold());
        let instructions = Title::from(Line::from(vec![
            " Run Fuzzer ".into(),
            " <R> ".blue().bold(),
            " Quit ".into(),
            " <Q> ".blue().bold()
        ]));

        let block = Block::bordered()
            .title(title.alignment(Alignment::Center))
            .title(
                instructions
                .alignment(Alignment::Center)
                .position(Position::Bottom)
            )
            .border_set(border::THICK);

        let results = self.query_results.lock().unwrap();
        let center_text = Text::from(
            results.iter()
                .filter(|x| x.1.eq(&StatusCode::OK))
                .map(|(url, statuscode)| Line::from(format!(" {} => {} ", statuscode, url)))
                .collect::<Vec<Line>>()
        );

        Paragraph::new(center_text)
            .centered()
            .block(block)
            .render(area, buf);
    }
}

