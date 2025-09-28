use crate::app::{Subject, TimeBlock};
use ratatui::layout::Rect;
use ratatui::{
    buffer::Buffer,
    prelude::*,
    style::{Color, Stylize},
    widgets::{Block, BorderType, Borders, Paragraph, Widget},
};

use crate::app::App;

const ROW_DISPLAY_COUNT: usize = 21 - 7;
const ROW_CONSTRAINT_PERCENTAGE: u16 = (100.0 / ROW_DISPLAY_COUNT as f32) as u16 - 1;
fn map_idx_to_time(index: usize) -> usize {
    index + 7
}
const DAYS: [&str; 5] = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday"];

fn get_color(sub: &Subject, c_list: &Vec<(String, Color)>) -> Color {
    for i in c_list {
        if i.0 == sub.name {
            return i.1;
        }
    }
    Color::Red
}
// First vec is for each day
// Second vec is for each hour
// Third is for all the subjects on that same hour and day
fn group_by_time(blocks: &Vec<TimeBlock>) -> Vec<Vec<Vec<TimeBlock>>> {
    let mut out: Vec<Vec<_>> = Vec::new();
    let mut day_filtered: Vec<_> = Vec::new();
    for d in 0..5 {
        let blocks_day_filtered = blocks
            .clone()
            .into_iter()
            .filter(|block| block.day == d)
            .collect::<Vec<TimeBlock>>();
        day_filtered.push(blocks_day_filtered);
    }
    for d in day_filtered {
        let mut tmp = Vec::new();
        for h in 0..ROW_DISPLAY_COUNT {
            let blocks_time_filtered = d
                .clone()
                .into_iter()
                .filter(|block| block.time == map_idx_to_time(h))
                .collect::<Vec<TimeBlock>>();
            if !blocks_time_filtered.is_empty() {
                tmp.push(blocks_time_filtered);
            }
        }
        out.push(tmp);
    }
    out
}

impl Widget for &App {
    /// Renders the user interface widgets.
    ///
    // This is where you add new widgets.
    // See the following resources:
    // - https://docs.rs/ratatui/latest/ratatui/widgets/index.html
    // - https://github.com/ratatui/ratatui/tree/master/examples
    fn render(self, area: Rect, buf: &mut Buffer) {
        let base_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(5), Constraint::Percentage(95)])
            .split(area);
        let columns_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints({
                let mut constraints = Vec::new();
                for _ in 0..5 {
                    constraints.push(Constraint::Percentage(20));
                }
                constraints
            })
            .split(base_layout[1]);
        let days_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints({
                let mut constraints = Vec::new();
                for _ in 0..5 {
                    constraints.push(Constraint::Percentage(20));
                }
                constraints
            })
            .split(base_layout[0]);

        let mut rows_layout: Vec<_> = Vec::new();
        for column in 0..columns_layout.len() {
            rows_layout.push(
                Layout::default()
                    .direction(Direction::Vertical)
                    .constraints({
                        let mut constraints = Vec::new();
                        for _ in 0..ROW_DISPLAY_COUNT {
                            constraints.push(Constraint::Percentage(ROW_CONSTRAINT_PERCENTAGE));
                        }
                        constraints
                    })
                    .split(columns_layout[column]),
            );
        }
        for col in columns_layout.iter() {
            Paragraph::default()
                .block(Block::default().borders(Borders::ALL))
                .fg(Color::DarkGray)
                .render(*col, buf);
        }

        for day in 0..5 {
            Paragraph::new(DAYS[day])
                .block(Block::default().borders(Borders::RIGHT))
                .alignment(Alignment::Center)
                .fg(Color::White)
                .render(days_layout[day], buf);
        }

        let blocks_grouped = group_by_time(&self.timetable_data);
        for (d, day) in blocks_grouped.iter().enumerate() {
            for hour in day {
                for (b, block) in hour.iter().enumerate() {
                    // If the class is longer than 1h, merge the rows
                    let idx = block.time - 7;
                    let mut area_render = rows_layout[d][idx];
                    if block.duration > 1 {
                        area_render = area_render.union(rows_layout[d][idx + (block.duration - 1)]);
                    }
                    // By how many columns will the cell be split up
                    let split_by = hour.len();
                    // Get the right constraints by dividing 100% with the above
                    let split_area = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints({
                            let mut constraints = Vec::new();
                            for _ in 0..split_by {
                                constraints
                                    .push(Constraint::Percentage((100.0 / split_by as f32) as u16));
                            }
                            constraints
                        })
                        .split(area_render);
                    // Render the block in its own column
                    let block_render = Block::default()
                        .border_type(BorderType::Plain)
                        .fg(get_color(&block.subject, &self.colors))
                        .title(format!("{}:00", &block.time))
                        .borders(Borders::ALL);
                    Paragraph::new(block.format_block())
                        .block(block_render)
                        .fg(Color::White)
                        .render(split_area[b], buf);
                }
            }
        }
    }
}
