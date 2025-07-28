use crate::event::{AppEvent, Event, EventHandler};
use crossterm::event::KeyEventKind;
use rand::prelude::*;
use ratatui::{
    DefaultTerminal,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    style::Color,
};

use serde_derive::Deserialize;
use serde_derive::Serialize;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeBlock {
    pub dan: isize,
    pub predmet: Predmet,
    pub profesor: String,
    pub tip: String,
    pub trajanje: isize,
    pub ucilnica: String,
    pub ura: isize,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Predmet {
    pub abbr: String,
    pub location: String,
    pub name: String,
}

/// Application.
#[derive(Debug)]
pub struct App {
    pub running: bool,
    pub scroll_index: u8,
    pub events: EventHandler,
    pub timetable_data: Vec<TimeBlock>,
    pub colors: Vec<(String, Color)>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            scroll_index: 0,
            events: EventHandler::new(),
            timetable_data: Vec::new(),
            colors: Vec::new(),
        }
    }
}

impl App {
    /// Run the application's main loop.
    pub async fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        while self.running {
            terminal.draw(|frame| frame.render_widget(&self, frame.area()))?;
            match self.events.next().await? {
                Event::Tick => self.tick(),
                Event::Crossterm(event) => {
                    if let crossterm::event::Event::Key(key_event) = event {
                        self.handle_key_events(key_event)?
                    }
                }
                Event::App(app_event) => match app_event {
                    AppEvent::Quit => self.quit(),
                    AppEvent::Refresh => {
                        'fetching_json: {
                            let url = "http://localhost:8080/timetable/fri/61310";
                            let mut data = ureq::get(url)
                                .call()
                                .unwrap()
                                .body_mut()
                                .read_to_string()
                                .unwrap();
                            _ = data.trim();
                            let json: Vec<TimeBlock> = serde_json::from_str(&data).unwrap();
                            self.timetable_data = json;
                        }
                        'colors: {
                            let mut rng = rand::rng();
                            let mut colors_rand: Vec<(String, Color)> = Vec::new();
                            let mut subjects: Vec<String> = self
                                .timetable_data
                                .clone()
                                .iter()
                                .map(|entry| entry.predmet.name.clone())
                                .collect();
                            if !subjects.is_empty() {
                                subjects.sort();
                                subjects.dedup();
                            }
                            for sub in subjects {
                                let temp_color = Color::Rgb(
                                    rng.random::<u8>(),
                                    rng.random::<u8>(),
                                    rng.random::<u8>(),
                                );
                                colors_rand.push((sub, temp_color))
                            }
                            self.colors = colors_rand;
                        }
                    }
                },
                _ => {}
            }
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    pub fn handle_key_events(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        if key_event.kind == KeyEventKind::Press {
            match key_event.code {
                KeyCode::Esc | KeyCode::Char('q') => self.events.send(AppEvent::Quit),
                KeyCode::Char('c' | 'C') if key_event.modifiers == KeyModifiers::CONTROL => {
                    self.events.send(AppEvent::Quit)
                }
                KeyCode::Char('r') => self.events.send(AppEvent::Refresh),
                _ => {}
            }
        }
        Ok(())
    }

    /// Handles the tick event of the terminal.
    ///
    /// The tick event is where you can update the state of your application with any logic that
    /// needs to be updated at a fixed frame rate. E.g. polling a server, updating an animation.
    pub fn tick(&self) {}

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }
}
