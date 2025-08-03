use crate::app::{Subject, TimeBlock, Warning};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    prelude::*,
    style::Color,
    widgets::{Block, BorderType, Borders, Paragraph, Widget},
};

use crate::app::App;

const ROW_DISPLAY_COUNT: usize = 21 - 7;
const ROW_CONSTRAINT_PERCENTAGE: u16 = (100 / ROW_DISPLAY_COUNT) as u16 - 1;
fn map_idx_to_time(index: usize) -> usize {
    index + 7
}

fn get_color(sub: Subject, c_list: &Vec<(String, Color)>) -> Color {
    for i in c_list {
        if i.0 == sub.name {
            return i.1;
        }
    }
    Color::Red
}
fn group_by_time(blocks: &Vec<TimeBlock>) -> Vec<Vec<TimeBlock>> {
    let mut out: Vec<_> = Vec::new();
    for h in 0..ROW_DISPLAY_COUNT {
        let blocks_time_filtered = blocks
            .clone()
            .into_iter()
            .filter(|block| block.time == map_idx_to_time(h))
            .collect::<Vec<TimeBlock>>();
        out.push(blocks_time_filtered);
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
        let columns_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints({
                let mut constraints = Vec::new();
                for _ in 0..5 {
                    constraints.push(Constraint::Percentage(20));
                }
                constraints
            })
            .split(area);

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

        let blocks_grouped = group_by_time(&self.timetable_data);
        for i in 0..4 {
            for time_group in blocks_grouped.iter().filter(|b| !b.is_empty()) {
                let area_render = rows_layout[time_group[0].day][time_group[0].time - 7];
                let constraint_perc = 100 / time_group.len() as u16;
                let group_area = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints({
                        let mut constraints = Vec::new();
                        for _ in 0..time_group.len() {
                            constraints.push(Constraint::Percentage(constraint_perc));
                        }
                        constraints
                    })
                    .split(area_render);
                for (i, block) in time_group.iter().enumerate() {
                    let block_render = Block::default()
                        .border_type(BorderType::Plain)
                        .borders(Borders::ALL);
                    _ = Paragraph::new(block.format_block())
                        .block(block_render)
                        .bg(Color::Red)
                        .render(group_area[i], buf);
                }
            }
        }

        if self.warning.is_some() {
            _ = Warning::default();
        }
    }
}
