use std::env;
use std::fs::File;
use std::io::{self, BufRead};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use reqwest::{Client, StatusCode};
use futures::stream::{self, StreamExt};
use tokio::runtime::Runtime;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
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

// Request fuzzing work
async fn request_url(client: &Client, url: &str) -> Result<(String, StatusCode), reqwest::Error> {
    let res = client.get(url).send().await;
    let result = match res {
        Ok(res) => {
            Ok((url.to_string(), res.status()))
        }
        Err(err) => {
            Err(err)
        }
    };
    result
}

fn fuzz(wordlist: &str, host: &str, query_results: &Arc<Mutex<HashMap<String, StatusCode>>>) {

    let wordlist = File::open(wordlist);
    let wordlist = io::BufReader::new(wordlist.unwrap()).lines();

    // really no point in continuing execution if this doesn't work
    let rt = Runtime::new().expect("Failed to create Async runtime");

    rt.block_on(async {
        let client = Client::new();
        let _ = stream::iter(wordlist.into_iter().map(|word| {
            let client = client.clone();
            let query_results = query_results.clone();
            let url = format!("{}{}", &host, &word.unwrap());

            tokio::spawn(async move {
                // println!("Requesting...");
                let result = request_url(&client, &url).await;
                match result {
                    Ok(result) => {
                        // lock only panicks when current thread already holds mutex
                        let mut map = query_results.lock().unwrap(); 
                        map.insert(result.0, result.1);
                    }
                    _ => ()
                }
            })
        }))
        .buffer_unordered(100)
        .collect::<Vec<_>>()
        .await;
    });
    
}

#[derive(Debug, Default)]
pub struct App {
    title: String,
    host: String,
    wordlist: String,
    query_results: Arc<Mutex<HashMap<String, StatusCode>>>,
    exit: bool,
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal, wordlist: &str, host: &str) -> io::Result<()> {
        self.title = " FuzzRS ".to_string();
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
                self.title = " FuzzRS (running...) ".to_string();
                fuzz(&self.wordlist, &self.host, &self.query_results);
            }
            KeyCode::Char('q') =>  self.exit(),
            _ => {}
        }
    }
    
    fn exit(&mut self) {
        self.exit = true;
    }
}

impl Widget for &App {
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
                .map(|(url, statuscode)| Line::from(format!(" {} => {} ", statuscode, url)))
                .collect::<Vec<Line>>()
        );

        Paragraph::new(center_text)
            .centered()
            .block(block)
            .render(area, buf);
    }
}

fn main() -> io::Result<()>{
    let args: Vec<String> = env::args().collect();

    let wordlist = &args[1];
    let host = &args[2];

    let mut terminal = ratatui::init();
    terminal.clear()?;
    let app_result = App::default().run(&mut terminal, wordlist, host);
    ratatui::restore();
    app_result

}
