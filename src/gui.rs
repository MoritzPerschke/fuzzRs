use std::collections::HashMap;
use std::sync::{ Arc, Mutex };
use std::io::{ self };

use ratatui::prelude::*;
use reqwest::{header, StatusCode};

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
use crate::constants;

#[derive(Debug, Default)]
pub struct Gui {
    state: AppState,
    exit: bool,
}

#[derive(Debug, Default)]
pub struct AppState {
    title: String,
    host: String,
    wordlist: String,
    query_results: Arc<Mutex<HashMap<String, StatusCode>>>,
}

impl Gui {
    pub fn run(&mut self, terminal: &mut DefaultTerminal, wordlist: &str, host: &str) -> io::Result<()> {
        let mut state = AppState::default();
        state.title = " FuzzRs ".to_string();
        state.wordlist = wordlist.to_string();
        state.host = host.to_string();

        while !self.exit {
            terminal.draw(|frame| self.draw(frame, &mut state))?;
            self.handle_events()?;
        }
        Ok(())
    }
    
    fn draw(&self, frame: &mut Frame, state: &mut AppState) {
        let root_layout = Layout::vertical(
            vec![
                Constraint::Percentage(20),
                Constraint::Percentage(80)
            ]
        ).split(frame.area());

        let header_layout = Layout::horizontal(
            vec![
                Constraint::Percentage(30),
                Constraint::Percentage(30),
                Constraint::Percentage(40)
            ]
        ).split(root_layout[0]);

        let body_layout = Layout::horizontal(
            vec![
                Constraint::Percentage(60),
                Constraint::Percentage(40)
            ]
        ).split(root_layout[1]);

        let settings_layout = Layout::vertical(
            vec![
                Constraint::Percentage(10),
                Constraint::Percentage(10),
                Constraint::Percentage(50),
                Constraint::Percentage(30),
            ]
        ).split(body_layout[0]);

        let dh_layout = Layout::horizontal(
            vec![
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ]
        ).split(settings_layout[3]);

        let fm_layout = Layout::horizontal(
            vec![
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ]
        ).split(settings_layout[2]);

        let right_body_layout = Layout::vertical(
            vec![
                Constraint::Percentage(70),
                Constraint::Percentage(30)
            ]
        ).split(body_layout[1]);

        // This can be simplified by constructing the individual widgets within e.g. 'LeftBodyWidget' (similar to helpwidget rn)
        /* Header Bar */
        frame.render_widget(LogoWidget {}, header_layout[0]);
        frame.render_widget(ProgressWidget {}, header_layout[1]);
        frame.render_widget(StatsWidget {}, header_layout[2]);

        /* Body */
        frame.render_widget(EmptyWidget {title: state.host.to_string()}, settings_layout[0]);
        frame.render_widget(EmptyWidget {title: state.wordlist.to_string()}, settings_layout[1]);
        frame.render_widget(EmptyWidget {title: "Data".to_string()}, dh_layout[0]);
        frame.render_widget(EmptyWidget {title: "Headers".to_string()}, dh_layout[1]);
        frame.render_widget(EmptyWidget {title: "Match".to_string()}, fm_layout[0]);
        frame.render_widget(EmptyWidget {title: "Filter".to_string()}, fm_layout[1]);
        frame.render_widget(EmptyWidget {title: "Results".to_string()}, right_body_layout[0]);
        frame.render_widget(HelpWidget {}, right_body_layout[1]);
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
                self.state.title = " FuzzRs (running...) ".to_string();
                fuzzer::fuzz(&self.state.wordlist, &self.state.host, &self.state.query_results);
            }
            KeyCode::Char('q') =>  self.exit(),
            _ => {}
        }
    }
    
    fn exit(&mut self) {
        self.exit = true;
    }
}

struct HeaderWidget {
}

impl StatefulWidget for HeaderWidget {
    type State = AppState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut AppState) {
        let block = Block::bordered()
            .border_set(border::THICK);

        let results = state.query_results.lock().unwrap();
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

struct LogoWidget {
}
impl Widget for LogoWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = constants::LOGO.to_string();

        let block = Block::bordered()
            .border_set(border::THICK);

        Paragraph::new(title)
            .centered()
            .block(block)
            .render(area, buf);

    }
}

struct ProgressWidget {}
impl Widget for ProgressWidget {
    fn render(self, area: Rect, buf: &mut Buffer){
        Paragraph::new("Progress")
            .centered()
            .block(Block::bordered().border_set(border::THICK))
            .render(area, buf)
    }
}

struct StatsWidget {}
impl Widget for StatsWidget {
    fn render(self, area: Rect, buf: &mut Buffer){
        Paragraph::new("Stats")
            .centered()
            .block(Block::bordered().border_set(border::THICK))
            .render(area, buf)
    }
}

struct HelpWidget;
impl Widget for HelpWidget {
    fn render(self, area: Rect, buf: &mut Buffer){

        let layout = Layout::horizontal(
            vec![
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ]
        ).split(area);

        Paragraph::new(Text::from(vec![
            Line::from(vec!["<r> ".bold().into(), " Run fuzzer ".into()]),
            Line::from(vec!["<q> ".bold().into(), " Quit fuzzer ".into()]),
        ]))
        .centered()
        .block(
                Block::bordered()
                .title(Title::from(" Help ".bold()).alignment(Alignment::Center))
                .border_set(border::ROUNDED)
            )
        .render(layout[0], buf);

        Paragraph::new(Text::from(vec![
            Line::from(vec!["<t> ".bold().into(), " Set Target ".into()]), // why does it need this?
            Line::from(vec!["<t> ".bold().into(), " Set Target ".into()]),
            Line::from(vec!["<w> ".bold().into(), " Set Wordlist ".into()]),
        ]))
        .centered()
        .render(layout[1], buf);

        Paragraph::new("")
        .block(
                Block::bordered()
                .title(Title::from(" Help ".bold()).alignment(Alignment::Center))
                .border_set(border::ROUNDED)
            )
        .render(area, buf)

    }
}

struct EmptyWidget {
    title: String,
}
impl Widget for EmptyWidget {
    fn render(self, area: Rect, buf: &mut Buffer){
        Paragraph::new(self.title)
            .centered()
            .block(Block::bordered().border_set(border::ROUNDED))
            .render(area, buf)
    }
}
