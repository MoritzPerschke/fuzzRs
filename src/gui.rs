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
        Block, Paragraph, Widget, Borders
    },
    DefaultTerminal,
    Frame
};
use tui_textarea::{Input, Key, TextArea};

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

        let mut wordlist = TextArea::default();
        wordlist.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title("Wordlist"),
        );

        while !self.exit {
            terminal.draw(|frame| self.draw(frame, &mut state, &wordlist))?;
            self.handle_events(&mut wordlist)?;
        }
        Ok(())
    }
    
    fn draw(&self, f: &mut Frame, state: &mut AppState, wordlist: &TextArea) {
        /* Definitions of Layouts */
        let root_layout = Layout::vertical(
            vec![
                Constraint::Percentage(20),
                Constraint::Percentage(80)
            ]
        ).split(f.area());

        let body_layout = Layout::horizontal(
            vec![
                Constraint::Percentage(60),
                Constraint::Percentage(40)
            ]
        ).split(root_layout[1]);

        let input_layout = Layout::vertical(
            vec![
                Constraint::Percentage(10), // target input
                Constraint::Percentage(10), // wordlist input
                Constraint::Percentage(50), // data/header input
                Constraint::Percentage(30), // filter/match input
            ]
        ).split(root_layout[1]);

        let fm_layout = Layout::horizontal(
            vec![
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ]
        ).split(input_layout[2]);


        let dh_layout = Layout::horizontal(
            vec![
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ]
        ).split(input_layout[3]);

        // this can be moved to future 'rightwidget' or whatever
        let right_body_layout = Layout::vertical(
            vec![
                Constraint::Percentage(70),
                Constraint::Percentage(30)
            ]
        ).split(body_layout[1]);

        /* Header Bar */
        // doesn't need to be stateful methinks
        f.render_stateful_widget(HeaderWidget, root_layout[0], state);

        /* Body */
        // f.render_widget(targetinput{}, right_body_layout[1]);
        // f.render_widget(wordlistinput{}, right_body_layout[1]);
        // f.render_widget(datainput{}, right_body_layout[1]);
        // f.render_widget(headerinput{}, right_body_layout[1]);
        // f.render_widget(matchinput{}, right_body_layout[1]);
        // f.render_widget(filterinput{}, right_body_layout[1]);
        f.render_widget(EmptyWidget {title: "Results".to_string()}, right_body_layout[0]);
        f.render_widget(HelpWidget {}, right_body_layout[1]);
    }

    fn handle_events(&mut self, wordlist: &mut TextArea) -> io::Result<()> {
        match crossterm::event::read()?.into() {
            // Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
            //     self.handle_key_event(key_event)
            // }
            Input {key: Key::Char('q'), ctrl: true, ..} => self.exit(),
            Input {key: Key::Char('r'), ctrl: true, ..} => fuzzer::fuzz(&self.state.wordlist, &self.state.host, &self.state.query_results),
            input => {
                wordlist.input(input);
            }
        };
        Ok(())
    }

    // fn handle_key_event(&mut self, key_event: KeyEvent) {
    //     match key_event.code {
    //         KeyCode::Char('r') =>  {
    //             self.state.title = " FuzzRs (running...) ".to_string();
    //             fuzzer::fuzz(&self.state.wordlist, &self.state.host, &self.state.query_results);
    //         }
    //         KeyCode::Char('q') =>  self.exit(),
    //         _ => {}
    //     }
    // }

    fn exit(&mut self) {
        self.exit = true;
    }
}

struct HeaderWidget;
impl StatefulWidget for HeaderWidget {
    type State = AppState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut AppState) {

        let header_layout = Layout::horizontal(
            vec![
                Constraint::Percentage(30),
                Constraint::Percentage(30),
                Constraint::Percentage(40)
            ]
        ).split(area);

        LogoWidget {}.render(header_layout[0], buf);
        ProgressWidget {}.render(header_layout[1], buf);
        StatsWidget {}.render(header_layout[2], buf);
    }
}

struct LogoWidget;
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

struct ProgressWidget;
impl Widget for ProgressWidget {
    fn render(self, area: Rect, buf: &mut Buffer){
        Paragraph::new("Progress")
            .centered()
            .block(Block::bordered().border_set(border::THICK))
            .render(area, buf)
    }
}

struct StatsWidget;
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
            Line::from(vec!["<C-t> ".bold().into(), " Set Target ".into()]), // why does it need this?
            Line::from(vec!["<C-t> ".bold().into(), " Set target ".into()]),
            Line::from(vec!["<C-w> ".bold().into(), " Set wordlist ".into()]),
            Line::from(vec!["<C-d> ".bold().into(), " Edit request data ".into()]),
            Line::from(vec!["<C-h> ".bold().into(), " Edit request headers ".into()]),
            Line::from(vec!["<C-m> ".bold().into(), " Edit matching rules ".into()]),
            Line::from(vec!["<C-f> ".bold().into(), " Edit filter rules ".into()]),
        ]))
        .centered()
        .render(layout[1], buf);

        Paragraph::new(Text::from(vec![
            Line::from(vec!["<C-r> ".bold().into(), " Run fuzzer ".into()]),
            Line::from(vec!["<C-q> ".bold().into(), " Quit fuzzer ".into()]),
            Line::from(vec!["<C-s> ".bold().into(), " Edit settings ".into()]),
        ]))
        .centered()
        .block(
                Block::bordered()
                .title(Title::from(" Help ".bold()).alignment(Alignment::Left))
                .border_set(border::ROUNDED)
            )
        .render(layout[0], buf);

        Paragraph::new("")
        .block(
                Block::bordered()
                .title(Title::from(" Help ".bold()).alignment(Alignment::Center))
                .border_set(border::ROUNDED)
            )
        .render(area, buf)

    }
}

struct InputWidget{
    state: AppState,
}
impl StatefulWidget for InputWidget {
    type State = AppState;
    fn render(self, area:Rect, buf: &mut Buffer, state: &mut AppState){


        EmptyWidget {title: state.host.to_string()}.render(settings_layout[0], buf);
        state.wordlist.render(settings_layout[1], buf);
        EmptyWidget {title: "Data".to_string()}.render(dh_layout[0], buf);
        EmptyWidget {title: "Headers".to_string()}.render(dh_layout[1], buf);
        EmptyWidget {title: "Match".to_string()}.render(fm_layout[0], buf);
        EmptyWidget {title: "Filter".to_string()}.render(fm_layout[1], buf);
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
