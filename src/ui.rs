use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::{
    app::{App, InputMode, ScrollState, StatusType},
    markdown::MarkdownRenderer,
    types::{Message, MessageContent, Role},
    widgets::{help::HelpWidget, loading::LoadingWidget},
};

pub fn draw(f: &mut Frame, app: &App) {
    let background = Block::default().style(Style::default().bg(Color::Black));
    f.render_widget(background, f.area());

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(f.area());

    draw_header(f, app, chunks[0]);
    draw_messages(f, app, chunks[1]);
    draw_input(f, app, chunks[2]);
    draw_status(f, app, chunks[3]);

    if app.show_help {
        HelpWidget::draw(f, f.area());
    }

    if app.is_loading {
        let loading_widget = LoadingWidget::new(app.loading_frame);
        loading_widget.draw(f, chunks[1]);
    }
}

fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    let header = Paragraph::new(Text::from(vec![Line::from(vec![
        Span::styled("◉ ", Style::default().fg(Color::Cyan)),
        Span::styled(
            "hugging",
            Style::default()
                .fg(Color::White)
                .bg(Color::Black)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            " code",
            Style::default()
                .fg(Color::Cyan)
                .bg(Color::Black)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" │ "),
        Span::styled(&app.config.model, Style::default().fg(Color::Yellow)),
    ])]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(" Hugging Face ")
            .title_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
    )
    .style(Style::default().bg(Color::Black))
    .alignment(Alignment::Center);
    f.render_widget(header, area);
}

fn draw_messages(f: &mut Frame, app: &App, area: Rect) {
    let renderer = MarkdownRenderer::new();
    let mut all_lines: Vec<Line> = Vec::new();

    let messages_to_render: Vec<&Message> = match app.scroll_state {
        ScrollState::Bottom => app.messages.iter().collect(),
        ScrollState::Fixed(_) => app.messages.iter().skip(app.scroll_offset).collect(),
    };

    for msg in messages_to_render.iter() {
        let (base_style, prefix, role_color) = match msg.role {
            Role::User => (
                Style::default().fg(Color::Green).bg(Color::Black),
                "👤 You",
                Color::Green,
            ),
            Role::Assistant => (
                Style::default().fg(Color::Blue).bg(Color::Black),
                "🤖 AI",
                Color::Blue,
            ),
            Role::System => (
                Style::default().fg(Color::Gray).bg(Color::Black),
                "⚙️ System",
                Color::Gray,
            ),
        };

        let time_str = msg.datetime.format("%H:%M:%S").to_string();

        all_lines.push(Line::from(vec![
            Span::styled(
                prefix,
                Style::default().fg(role_color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" [{}]", time_str),
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                ":",
                Style::default().fg(role_color).add_modifier(Modifier::BOLD),
            ),
        ]));

        match &msg.content {
            MessageContent::Text(text) => {
                let rendered = if msg.role == Role::Assistant {
                    renderer.render(text, base_style)
                } else {
                    render_plain_text(text, base_style)
                };
                for line in rendered.lines {
                    all_lines.push(line.clone());
                }
            }
            MessageContent::WithThinking { thinking, output } => {
                if app.show_thinking {
                    all_lines.push(Line::from(vec![
                        Span::styled("  🤔 ", Style::default().fg(Color::Magenta)),
                        Span::styled(
                            "[Thinking Process] ",
                            Style::default()
                                .fg(Color::Magenta)
                                .add_modifier(Modifier::BOLD),
                        ),
                    ]));

                    for thinking_line in thinking.lines() {
                        all_lines.push(Line::from(vec![
                            Span::raw("    "),
                            Span::styled(
                                thinking_line.to_string(),
                                Style::default()
                                    .fg(Color::Magenta)
                                    .add_modifier(Modifier::ITALIC | Modifier::DIM),
                            ),
                        ]));
                    }

                    all_lines.push(Line::from(vec![Span::styled(
                        "  ════════════════════",
                        Style::default().fg(Color::DarkGray),
                    )]));
                } else if !thinking.is_empty() {
                    all_lines.push(Line::from(vec![
                        Span::styled(
                            "  🤔 ",
                            Style::default()
                                .fg(Color::Magenta)
                                .add_modifier(Modifier::DIM),
                        ),
                        Span::styled(
                            "[Thinking hidden - press 't' to show] ",
                            Style::default()
                                .fg(Color::Magenta)
                                .add_modifier(Modifier::ITALIC | Modifier::DIM),
                        ),
                    ]));
                }

                let rendered = renderer.render(output, base_style);
                for line in rendered.lines {
                    let mut indented_line = vec![Span::raw("  ")];
                    indented_line.extend(line.spans.into_iter());
                    all_lines.push(Line::from(indented_line));
                }
            }
        }

        all_lines.push(Line::default());
    }

    let viewport_height = area.height.saturating_sub(2) as usize;
    let total_lines = all_lines.len();

    let visible_lines: Vec<Line> = match app.scroll_state {
        ScrollState::Bottom => {
            if total_lines > viewport_height {
                all_lines
                    .into_iter()
                    .skip(total_lines - viewport_height)
                    .collect()
            } else {
                all_lines
            }
        }
        ScrollState::Fixed(_) => all_lines.into_iter().take(viewport_height).collect(),
    };

    let scroll_info = if app.messages.is_empty() {
        String::new()
    } else {
        match app.scroll_state {
            ScrollState::Bottom => " [BOTTOM ↓] ".to_string(),
            ScrollState::Fixed(offset) => {
                format!(
                    " [MSG {}/{}] ",
                    (offset + 1).min(app.messages.len()),
                    app.messages.len()
                )
            }
        }
    };

    let messages_paragraph = Paragraph::new(Text::from(visible_lines))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
                .title(format!(" Messages {}", scroll_info))
                .title_style(Style::default().fg(Color::White).bg(Color::Black)),
        )
        .style(Style::default().bg(Color::Black))
        .wrap(Wrap { trim: false });

    f.render_widget(messages_paragraph, area);
}

fn draw_input(f: &mut Frame, app: &App, area: Rect) {
    let input_style = match app.input_mode {
        InputMode::Normal => Style::default().fg(Color::White).bg(Color::Black),
        InputMode::Editing => Style::default().fg(Color::Green).bg(Color::Black),
    };

    let char_count = app.input.len();
    let input_title = match app.input_mode {
        InputMode::Normal => " Input (Press 'i' to edit) ",
        InputMode::Editing => &format!(
            " Input [Esc=cancel | Enter=SEND | Shift+Enter=newline | {}ch] ",
            char_count
        ),
    };

    let input = Paragraph::new(app.input.as_str())
        .style(input_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(input_title)
                .border_style(input_style)
                .title_style(Style::default().fg(Color::White).bg(Color::Black)),
        )
        .wrap(Wrap { trim: false });
    f.render_widget(input, area);

    if let InputMode::Editing = app.input_mode {
        f.set_cursor_position((area.x + app.input.len() as u16 + 1, area.y + 1));
    }
}

fn draw_status(f: &mut Frame, app: &App, area: Rect) {
    if let Some((message, status_type)) = &app.status_message {
        let status_style = match status_type {
            StatusType::Info => Style::default().fg(Color::Cyan).bg(Color::Black),
            StatusType::Success => Style::default().fg(Color::Green).bg(Color::Black),
            StatusType::Warning => Style::default().fg(Color::Yellow).bg(Color::Black),
            StatusType::Error => Style::default().fg(Color::Red).bg(Color::Black),
        };

        f.render_widget(Clear, area);
        let status = Paragraph::new(message.as_str()).style(status_style);
        f.render_widget(status, area);
    }
}

fn render_plain_text(text: &str, base_style: Style) -> Text<'static> {
    let mut lines = Vec::new();
    for line in text.split('\n') {
        lines.push(Line::from(Span::styled(line.to_string(), base_style)));
    }
    Text::from(lines)
}
