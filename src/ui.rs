use crate::app::TimeBlock;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    prelude::*,
    style::{Color, Stylize},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Widget, Wrap},
};

use crate::app::App;
use crate::app::Warning;

fn map_idx_to_time(index: usize) -> usize {
    index + 7
}

const ROW_DISPLAY_COUNT: usize = 21 - 7;

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
        for i in 0..columns_layout.len() {
            rows_layout.push(
                Layout::default()
                    .direction(Direction::Vertical)
                    .constraints({
                        let mut constraints = Vec::new();
                        for _ in 0..ROW_DISPLAY_COUNT {
                            constraints.push(Constraint::Percentage(20));
                        }
                        constraints
                    })
                    .split(columns_layout[i]),
            );
        }

        for column in rows_layout.iter() {
            for h in 0..ROW_DISPLAY_COUNT {
                let block = Block::bordered()
                    .title("")
                    .title_alignment(Alignment::Center)
                    .border_type(BorderType::Plain);

                let paragraph = Paragraph::new("")
                    .block(block)
                    .fg(Color::DarkGray)
                    .bg(Color::Black)
                    .centered();
                paragraph.clone().render(column[h as usize], buf);
            }
        }

        for (d, column) in rows_layout.iter().enumerate() {
            let blocks_filtered: Vec<TimeBlock> = self
                .timetable_data
                .clone()
                .into_iter()
                .filter(|block| block.day as usize == d)
                .collect::<Vec<TimeBlock>>();
            for h in 0..ROW_DISPLAY_COUNT {
                let blocks_time_filtered = blocks_filtered
                    .clone()
                    .into_iter()
                    .filter(|block| block.time as usize == map_idx_to_time(h))
                    .collect::<Vec<TimeBlock>>();
                if !blocks_time_filtered.is_empty() {
                    let subject_color = {
                        let i = self
                            .colors
                            .clone()
                            .iter()
                            .filter(|t| t.0 == blocks_time_filtered[0].subject.name)
                            .collect::<Vec<_>>()[0]
                            .1;
                        i
                    };
                    let block = Block::bordered()
                        .title({
                            if blocks_time_filtered[0].duration > 0 {
                                format!("{}:00", &blocks_time_filtered[0].time)
                            } else {
                                "".to_string()
                            }
                        })
                        .title_alignment(Alignment::Center)
                        .border_type(BorderType::Plain)
                        .border_style(Style::default().fg(subject_color));

                    let joined_area = {
                        let mut joined_area = column[h as usize];
                        if blocks_time_filtered[0].duration > 1 {
                            for c in 1..blocks_time_filtered[0].duration {
                                joined_area = joined_area.union(column[(h as usize) + c as usize]);
                            }
                        }
                        joined_area
                    };
                    let paragraph = Paragraph::new({
                        let b = blocks_time_filtered[0].clone();
                        if b.duration > 0 {
                            format!(
                                "{}\n\
                                    {}\n\
                                    {}\n\
                                    Classroom: {}",
                                &b.professor, &b.subject.name, &b.subject.r#type, &b.classroom
                            )
                        } else {
                            "".to_string()
                        }
                    })
                    .block(block)
                    .fg(Color::White)
                    .bg(Color::Black)
                    .centered();

                    Clear.render(joined_area, buf);
                    paragraph.clone().render(joined_area, buf);
                } else {
                }
            }
            if let Some(warn_data) = self.warning.clone() {
                let popup_area = Rect {
                    x: area.width / 4,
                    y: area.height / 3,
                    width: area.width / 2,
                    height: area.height / 3,
                };

                if popup_area.height < 1 {
                    return;
                }

                Clear.render(popup_area, buf);

                let help_block = Block::new()
                    .title(warn_data.title)
                    .title_style(Style::new().white().bold())
                    .borders(Borders::ALL)
                    .border_style(Style::new().fg(warn_data.color));
                Paragraph::new(warn_data.message)
                    .style(Style::new())
                    .block(help_block)
                    .render(popup_area, buf);

                let mut hint_popup_area = popup_area;
                hint_popup_area.y += popup_area.height - 1;
                hint_popup_area.x += 1;

                Paragraph::new(warn_data.bottom_hint)
                    .alignment(Alignment::Left)
                    .render(hint_popup_area, buf);
            }
        }
    }
}
