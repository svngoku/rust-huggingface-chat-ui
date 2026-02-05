use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Text},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

pub struct HelpWidget;

impl HelpWidget {
    pub fn draw(f: &mut Frame, area: Rect) {
        let help_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(15),
                Constraint::Length(20),
                Constraint::Percentage(15),
            ])
            .split(area)[1];

        let help_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20),
                Constraint::Length(60),
                Constraint::Percentage(20),
            ])
            .split(help_area)[1];

        let help_text = Text::from(vec![
            Line::from("📖 Help & Commands"),
            Line::from(""),
            Line::from("🎮 Navigation:"),
            Line::from("  i              - Enter input mode"),
            Line::from("  Esc            - Exit input mode"),
            Line::from("  Enter          - Send message"),
            Line::from("  Shift+Enter    - New line in message (for multiline)"),
            Line::from("  ↑/↓            - Scroll messages up/down"),
            Line::from("  Home/End       - Jump to top/bottom"),
            Line::from("  PageUp/PageDn  - Scroll page up/down"),
            Line::from("  h              - Toggle this help"),
            Line::from("  t              - Toggle thinking tokens visibility"),
            Line::from("  q              - Quit application"),
            Line::from(""),
            Line::from("💬 Commands (type in input):"),
            Line::from("  /help, /h      - Toggle help"),
            Line::from("  /clear, /c     - Clear conversation"),
            Line::from("  /stats, /s     - Show statistics & token estimate"),
            Line::from("  /save [file]   - Save conversation (default: conversation.json)"),
            Line::from("  /load [file]   - Load conversation (default: conversation.json)"),
            Line::from(""),
            Line::from("⌨️  Shortcuts:"),
            Line::from("  Ctrl+S         - Quick save (while editing)"),
            Line::from(""),
            Line::from("✨ Features:"),
            Line::from("  • Markdown rendering with timestamps"),
            Line::from("  • Thinking tokens detection and display"),
            Line::from("  • Smart context window management (last 20 msgs)"),
            Line::from("  • Multiline input with Shift+Enter, send with Enter"),
            Line::from("  • Conversation save/load as JSON"),
            Line::from("  • Character counter & scroll position"),
        ]);

        let help_popup = Paragraph::new(help_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan))
                    .title(" Help ")
                    .title_alignment(Alignment::Center),
            )
            .style(Style::default().bg(Color::Black))
            .wrap(Wrap { trim: true });

        f.render_widget(Clear, help_area);
        f.render_widget(help_popup, help_area);
    }
}
