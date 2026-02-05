use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub struct LoadingWidget {
    pub frame: usize,
}

impl LoadingWidget {
    pub fn new(frame: usize) -> Self {
        Self { frame }
    }

    pub fn draw(&self, f: &mut Frame, area: Rect) {
        let loading_area = centered_rect(area, 44, 7);

        let loader_frames = ["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"];
        let current_frame = loader_frames[self.frame % loader_frames.len()];
        let loading_text = format!("{} Loading response...", current_frame);

        let loading = Paragraph::new(loading_text)
            .style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow))
                    .title(" AI is thinking... ")
                    .title_style(Style::default().fg(Color::Yellow).bg(Color::Black))
                    .style(Style::default().bg(Color::Black)),
            );

        f.render_widget(Clear, loading_area);
        f.render_widget(loading, loading_area);
    }
}

fn centered_rect(area: Rect, max_width: u16, max_height: u16) -> Rect {
    let width = area.width.min(max_width).max(10);
    let height = area.height.min(max_height).max(5);
    let x = area.x + area.width.saturating_sub(width) / 2;
    let y = area.y + area.height.saturating_sub(height) / 2;
    Rect {
        x,
        y,
        width,
        height,
    }
}
