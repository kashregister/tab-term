use crate::event::{AppEvent, Event, EventHandler};
use crossterm::event::KeyEventKind;
use rand::prelude::*;
use ratatui::{
    DefaultTerminal,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    style::Color,
};
use std::path::Path;
use ureq::Error;

use serde_derive::Deserialize;
use serde_derive::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeBlock {
    pub day: usize,
    pub time: usize,
    pub duration: usize,
    pub professor: String,
    pub classroom: String,
    pub subject: Subject,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Subject {
    pub name: String,
    pub abbreviation: String,
    pub location: String,
    pub r#type: String,
}
#[derive(Debug, Clone)]
pub struct Warning {
    pub title: String,
    pub message: String,
    pub bottom_hint: String,
    pub color: Color,
}

/// Application.
#[derive(Debug)]
pub struct App {
    pub running: bool,
    pub scroll_index: u8,
    pub events: EventHandler,
    pub timetable_data: Vec<TimeBlock>,
    pub colors: Vec<(String, Color)>,
    pub warning: Option<Warning>,
    pub config: Option<String>,
}

impl Default for App {
    fn default() -> Self {
        let mut init = Self {
            running: true,
            scroll_index: 0,
            events: EventHandler::new(),
            timetable_data: Vec::new(),
            colors: Vec::new(),
            warning: None,
            config: None,
        };
        init.events.send(AppEvent::Refresh);
        init
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
                        self.config = App::check_config();
                        'fetching_json: {
                            // http://localhost:8080/timetable/fri/61310
                            // let url = "http://localhost:8080/timetable/fri/61310";
                            if let Some(url) = self.config.clone() {
                                match ureq::get(url).call() {
                                    Ok(mut valid) => {
                                        let data = valid.body_mut().read_to_string().unwrap();

                                        _ = data.trim();
                                        let json: Vec<TimeBlock> =
                                            serde_json::from_str(&data).unwrap();
                                        self.timetable_data = json;
                                        'colors: {
                                            if !self.timetable_data.is_empty() {
                                                let mut rng = rand::rng();
                                                let mut colors_rand: Vec<(String, Color)> =
                                                    Vec::new();
                                                let mut subjects: Vec<String> = self
                                                    .timetable_data
                                                    .clone()
                                                    .iter()
                                                    .map(|entry| entry.subject.name.clone())
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
                                    }
                                    Err(Error::StatusCode(code)) => {
                                        if code == 429 {
                                            self.warning = Some(Warning {
                                                message: "Too many requests...".into(),
                                                title: "Error".into(),
                                                color: Color::Yellow,
                                                bottom_hint: "Press <Esc> to close the window"
                                                    .into(),
                                            });
                                        } else if code == 404 {
                                            self.warning = Some(Warning {
                                                message: "Page not found".into(),
                                                title: "Error".into(),
                                                color: Color::Red,
                                                bottom_hint: "Press <Esc> to close the window"
                                                    .into(),
                                            });
                                        } else if code == 408 {
                                            self.warning = Some(Warning {
                                                message: "Request timed out...".into(),
                                                title: "Error".into(),
                                                color: Color::Red,
                                                bottom_hint: "Press <Esc> to close the window"
                                                    .into(),
                                            });
                                        }
                                    }
                                    Err(_) => {
                                        self.warning = Some(Warning {
                                            message: "Host unreachable\n\
                                            Check if the config url works"
                                                .into(),
                                            title: "Error".into(),
                                            color: Color::Red,
                                            bottom_hint: "Press <Esc> to close the window".into(),
                                        });
                                    }
                                }
                            } else {
                                let mut ret = "Empty config file...\n\
                                    Add your api provider in:\n\
                                    "
                                .to_string();
                                if let Some(cfg_dir) = dirs::config_dir() {
                                    let str = cfg_dir
                                        .join("tab-term")
                                        .join("config.txt")
                                        .display()
                                        .to_string();
                                    ret.push_str(&str);
                                }
                                ret.push_str(
                                    "\n\
                                    \n\
                                    Example:\n\
                                    http://localhost:8080/timetable/fri/61310
                                    ",
                                );
                                self.warning = Some(Warning {
                                    message: ret,
                                    title: "Error".into(),
                                    color: Color::Red,
                                    bottom_hint: "Press <Esc> to close the window".into(),
                                });
                            }
                        }
                    } // _ => {}
                },
            }
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    pub fn handle_key_events(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        if key_event.kind == KeyEventKind::Press {
            match key_event.code {
                KeyCode::Char('q') => self.events.send(AppEvent::Quit),
                KeyCode::Char('c' | 'C') if key_event.modifiers == KeyModifiers::CONTROL => {
                    self.events.send(AppEvent::Quit)
                }
                KeyCode::Char('r') if self.warning.is_none() => self.events.send(AppEvent::Refresh),
                KeyCode::Esc if self.warning.is_some() => {
                    self.warning = None;
                }
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

    pub fn check_config() -> Option<String> {
        if let Some(cfg_dir) = dirs::config_dir() {
            let exists = cfg_dir.join("tab-term").join("config.txt");
            if !exists.is_file() {
                let mut config_file = cfg_dir.join("tab-term");
                std::fs::create_dir_all(config_file.clone()).unwrap();
                config_file = cfg_dir.join("tab-term/config.txt");
                std::fs::write(config_file, "").unwrap();
            }
        }

        if let Some(cfg_dir) = dirs::config_dir() {
            let config_file = cfg_dir.join("tab-term").join("config.txt");
            let file_contents: String =
                std::fs::read_to_string(config_file).unwrap_or_else(|_| "~~~~".to_string());
            if file_contents == "~~~~" || file_contents.is_empty() {
                return None;
            }
            return Some(file_contents.trim().to_string());
        } else {
            None
        }
    }
}
