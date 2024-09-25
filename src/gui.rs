use ratatui::prelude::*;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{
        block::Title,
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

#[derive(Debug, Default, Clone)]
pub struct AppState {
    pub target: String,
    pub wordlist: String,
    pub data: Vec<String>,
    pub headers: Vec<String>,
    pub matchrules: Vec<String>,
    pub filterrules: Vec<String>,
    pub query_results: Vec<fuzzer::Data>,
}

impl Gui {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<(), Box<dyn std::error::Error>> {
        /*
        * 0: Target
        * 1: Wordlist
        * 2: Data
        * 3: Headers
        * 4: Match rules
        * 5: Filter rules */
        let mut input_fields = [
            ("Target", TextArea::default()),
            ("Wordlist", TextArea::default()),
            ("Data", TextArea::default()),
            ("Headers", TextArea::default()),
            ("Matchrules", TextArea::default()),
            ("Filterrules", TextArea::default()),
        ];
        let mut current_input: usize = 0;
        for i in 1..input_fields.len(){
            input_fields[i].1.set_cursor_style(Style::default());
            input_fields[i].1.set_block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(input_fields[i].0),
            );
        }
        input_fields[0].1.set_cursor_style(Style::default().add_modifier(Modifier::REVERSED).add_modifier(Modifier::RAPID_BLINK));
        input_fields[0].1.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title(input_fields[0].0),
        );

        loop {
            terminal.draw(|frame| self.draw(frame, &input_fields))?;
            self.handle_events(&mut input_fields, &mut current_input)?;
            if self.exit {
                break;
            }
        }
        Ok(())
    }
    
    fn draw(&mut self, f: &mut Frame, input_fields: &[(&str, TextArea); 6]) {
        /* Definitions of Layouts */
        let root_layout = Layout::vertical(
            vec![
                Constraint::Percentage(20), // header bar
                Constraint::Percentage(80)  // main body
            ]
        ).split(f.area());

        let body_layout = Layout::horizontal(
            vec![
                Constraint::Percentage(60), // input windows
                Constraint::Percentage(40)  // results window and help
            ]
        ).split(root_layout[1]);

        let input_layout = Layout::vertical(
            vec![
                Constraint::Percentage(10), // target input
                Constraint::Percentage(10), // wordlist input
                Constraint::Percentage(50), // data/header input
                Constraint::Percentage(30), // filter/match input
            ]
        ).split(body_layout[0]);

        let dh_layout = Layout::horizontal(
            vec![
                Constraint::Percentage(50), // data
                Constraint::Percentage(50), // headers
            ]
        ).split(input_layout[2]);

        let fm_layout = Layout::horizontal(
            vec![
                Constraint::Percentage(50), // matchrules
                Constraint::Percentage(50), // filterrules
            ]
        ).split(input_layout[3]);

        // this can be moved to future 'rightwidget' or whatever
        let right_body_layout = Layout::vertical(
            vec![
                Constraint::Percentage(70), // results
                Constraint::Percentage(30)  // help
            ]
        ).split(body_layout[1]);

        /* Header Bar */
        f.render_widget(HeaderWidget, root_layout[0]);

        /* Body */
        // render input field in the corresponding area as a widget
        f.render_widget(&input_fields[0].1, input_layout[0]);
        f.render_widget(&input_fields[1].1, input_layout[1]);
        f.render_widget(&input_fields[2].1, dh_layout[0]);
        f.render_widget(&input_fields[3].1, dh_layout[1]);
        f.render_widget(&input_fields[4].1, fm_layout[0]);
        f.render_widget(&input_fields[5].1, fm_layout[1]);
        // render other widgets
        f.render_stateful_widget(ResultsWidget, right_body_layout[0], self);
        f.render_widget(HelpWidget {}, right_body_layout[1]);
    }

    fn handle_events(&mut self, input_fields: &mut [(&str, TextArea); 6], active_input: &mut usize) -> Result<(), Box<dyn std::error::Error>> {
        match crossterm::event::read()?.into() {
            Input {key: Key::Char('q'), ctrl: true, ..} => self.exit(),
            Input {key: Key::Char('r'), ctrl: true, ..} => fuzzer::fuzz(&mut self.state),
            Input {key: Key::Char('t'), ctrl: true, ..} => self.change_active_input(input_fields, 0, active_input),
            Input {key: Key::Char('w'), ctrl: true, ..} => self.change_active_input(input_fields, 1, active_input),
            Input {key: Key::Char('d'), ctrl: true, ..} => self.change_active_input(input_fields, 2, active_input),
            Input {key: Key::Char('h'), ctrl: true, ..} => self.change_active_input(input_fields, 3, active_input),
            Input {key: Key::Char('m'), ctrl: true, ..} => self.change_active_input(input_fields, 4, active_input),
            Input {key: Key::Char('f'), ctrl: true, ..} => self.change_active_input(input_fields, 5, active_input),
            input => {
                // 'give' the input of the window to the input field based on 'acitve_input'
                input_fields[*active_input].1.input(input);
                match *active_input {
                    0 => self.state.target = input_fields[0].1.lines().first().unwrap().to_string(),
                    1 => self.state.wordlist = input_fields[1].1.lines().first().unwrap().to_string(),
                    2 => self.state.data = input_fields[2].1.lines().to_vec(),
                    3 => self.state.headers = input_fields[3].1.lines().to_vec(),
                    4 => self.state.matchrules = input_fields[4].1.lines().to_vec(),
                    5 => self.state.filterrules = input_fields[5].1.lines().to_vec(),
                    _ => {}
                }
            }
        };
        Ok(())
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn change_active_input(&mut self, input_fields: &mut [(&str, TextArea); 6], next_active: usize, current_active: &mut usize) {
        input_fields[*current_active].1.set_cursor_style(Style::default());
        input_fields[next_active].1.set_cursor_style(Style::default().add_modifier(Modifier::REVERSED).add_modifier(Modifier::RAPID_BLINK));
        *current_active = next_active;
    }
    
}

struct ResultsWidget;
impl StatefulWidget for ResultsWidget{
    type State = Gui;
    fn render(self, area: Rect, buf: &mut Buffer, gui: &mut Gui) {
        let lines: Vec<Line> = gui.state.query_results.iter().map(|result| {
                Line::from(format!("{}: {} [Length: {}]", result.url, result.status, result.content_length))
                    .alignment(Alignment::Left)
        }).collect();

        Paragraph::new(Text::from(lines))
            .centered()
            .block(Block::bordered().border_set(border::PLAIN).title(" Results "))
            .render(area, buf)
    }
}

struct HeaderWidget;
impl Widget for HeaderWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {

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
            Line::from(vec!["<C-t> ".bold().into(), " Set target ".into()]).alignment(Alignment::Left),
            Line::from(vec!["<C-w> ".bold().into(), " Set wordlist ".into()]).alignment(Alignment::Left),
            Line::from(vec!["<C-d> ".bold().into(), " Edit request data ".into()]).alignment(Alignment::Left),
            Line::from(vec!["<C-h> ".bold().into(), " Edit request headers ".into()]).alignment(Alignment::Left),
            Line::from(vec!["<C-m> ".bold().into(), " Edit matching rules ".into()]).alignment(Alignment::Left),
            Line::from(vec!["<C-f> ".bold().into(), " Edit filter rules ".into()]).alignment(Alignment::Left),
        ]))
        .centered()
        .block(
                Block::bordered()
                .title(Title::from(" Help ".bold()).alignment(Alignment::Left))
                .border_set(border::PLAIN)
            )
        .render(layout[0], buf);

        Paragraph::new(Text::from(vec![
            Line::from(vec!["<C-r> ".bold().into(), " Run fuzzer ".into()]).alignment(Alignment::Left),
            Line::from(vec!["<C-q> ".bold().into(), " Quit fuzzer ".into()]).alignment(Alignment::Left),
            Line::from(vec!["<C-s> ".bold().into(), " Edit settings ".into()]).alignment(Alignment::Left),
        ]))
        .centered()
        .block(
                Block::bordered()
                .border_set(border::PLAIN)
            )
        .render(layout[1], buf);

    }
}

struct EmptyWidget {
    title: String,
}
impl Widget for EmptyWidget {
    fn render(self, area: Rect, buf: &mut Buffer){
        Paragraph::new(self.title)
            .centered()
            .block(Block::bordered().border_set(border::PLAIN))
            .render(area, buf)
    }
}
