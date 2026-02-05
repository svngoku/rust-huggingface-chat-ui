use pulldown_cmark::{Event as MdEvent, HeadingLevel, Options, Parser, Tag, TagEnd};
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
};
use syntect::{
    easy::HighlightLines,
    highlighting::{Style as SyntectStyle, Theme, ThemeSet},
    parsing::SyntaxSet,
};

pub struct MarkdownRenderer {
    #[allow(dead_code)]
    syntax_set: SyntaxSet,
    #[allow(dead_code)]
    theme_set: ThemeSet,
}

impl Default for MarkdownRenderer {
    fn default() -> Self {
        Self {
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
        }
    }
}

impl MarkdownRenderer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn render(&self, markdown: &str, base_style: Style) -> Text<'static> {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_STRIKETHROUGH);
        let parser = Parser::new_ext(markdown, options);
        let mut lines = vec![Line::default()];
        let mut current_line_spans = Vec::new();
        let mut in_code_block = false;
        let mut code_block_lang: Option<String> = None;
        let mut code_block_content = String::new();
        let mut in_bold = false;
        let mut in_italic = false;
        let mut list_depth: usize = 0;
        let mut in_link = false;
        let mut link_url: Option<String> = None;

        let mut in_table = false;
        let mut in_table_cell = false;
        let mut table_rows: Vec<Vec<String>> = Vec::new();
        let mut table_header_rows: usize = 0;
        let mut current_row: Vec<String> = Vec::new();
        let mut current_cell = String::new();

        for event in parser {
            match event {
                MdEvent::Start(tag) => match tag {
                    Tag::Table(_) => {
                        in_table = true;
                        table_rows.clear();
                        table_header_rows = 0;
                        current_row.clear();
                        current_cell.clear();
                    }
                    Tag::TableHead => {
                        in_table = true;
                    }
                    Tag::TableRow => {
                        if in_table {
                            current_row.clear();
                        }
                    }
                    Tag::TableCell => {
                        if in_table {
                            in_table_cell = true;
                            current_cell.clear();
                        }
                    }
                    Tag::Link { dest_url, .. } => {
                        in_link = true;
                        link_url = Some(dest_url.to_string());
                    }
                    Tag::Heading { level, .. } => {
                        if !current_line_spans.is_empty() {
                            lines.push(Line::from(current_line_spans.clone()));
                            current_line_spans.clear();
                        }
                        let prefix = match level {
                            HeadingLevel::H1 => "# ",
                            HeadingLevel::H2 => "## ",
                            HeadingLevel::H3 => "### ",
                            _ => "#### ",
                        };
                        current_line_spans.push(Span::styled(
                            prefix,
                            base_style.fg(Color::Cyan).add_modifier(Modifier::BOLD),
                        ));
                    }
                    Tag::CodeBlock(kind) => {
                        if !current_line_spans.is_empty() {
                            lines.push(Line::from(current_line_spans.clone()));
                            current_line_spans.clear();
                        }
                        in_code_block = true;
                        code_block_lang = match kind {
                            pulldown_cmark::CodeBlockKind::Fenced(lang) => Some(lang.to_string()),
                            _ => None,
                        };
                        code_block_content.clear();
                    }
                    Tag::Emphasis => in_italic = true,
                    Tag::Strong => in_bold = true,
                    Tag::List(_) => {
                        list_depth += 1;
                        if !current_line_spans.is_empty() {
                            lines.push(Line::from(current_line_spans.clone()));
                            current_line_spans.clear();
                        }
                    }
                    Tag::Item => {
                        let indent = "  ".repeat(list_depth.saturating_sub(1));
                        current_line_spans.push(Span::raw(format!("{}• ", indent)));
                    }
                    Tag::Paragraph => {
                        if !current_line_spans.is_empty() {
                            lines.push(Line::from(current_line_spans.clone()));
                            current_line_spans.clear();
                        }
                    }
                    _ => {}
                },
                MdEvent::End(tag) => match tag {
                    TagEnd::TableCell => {
                        if in_table {
                            in_table_cell = false;
                            current_row.push(current_cell.trim().to_string());
                        }
                    }
                    TagEnd::TableRow => {
                        if in_table {
                            if !current_row.is_empty() {
                                table_rows.push(current_row.clone());
                                current_row.clear();
                            }
                        }
                    }
                    TagEnd::TableHead => {
                        if in_table {
                            table_header_rows = table_rows.len();
                        }
                    }
                    TagEnd::Table => {
                        if in_table {
                            if !current_line_spans.is_empty() {
                                lines.push(Line::from(current_line_spans.clone()));
                                current_line_spans.clear();
                            }
                            let table_lines =
                                render_table(&table_rows, table_header_rows, base_style);
                            lines.extend(table_lines);
                            lines.push(Line::default());

                            in_table = false;
                            in_table_cell = false;
                            table_rows.clear();
                            current_row.clear();
                            current_cell.clear();
                            table_header_rows = 0;
                        }
                    }
                    TagEnd::Heading(_) => {
                        lines.push(Line::from(current_line_spans.clone()));
                        current_line_spans.clear();
                    }
                    TagEnd::CodeBlock => {
                        in_code_block = false;

                        let code_lines = self.render_code_block(
                            &code_block_content,
                            code_block_lang.as_deref(),
                            base_style,
                        );
                        lines.extend(code_lines);

                        code_block_content.clear();
                        code_block_lang = None;

                        if !current_line_spans.is_empty() {
                            lines.push(Line::from(current_line_spans.clone()));
                            current_line_spans.clear();
                        }
                    }
                    TagEnd::Emphasis => in_italic = false,
                    TagEnd::Strong => in_bold = false,
                    TagEnd::Link => {
                        in_link = false;
                        if let Some(url) = link_url.take() {
                            current_line_spans.push(Span::styled(
                                format!(" <{}>", url),
                                base_style.fg(Color::DarkGray),
                            ));
                        }
                    }
                    TagEnd::List(_) => {
                        list_depth = list_depth.saturating_sub(1);
                        if !current_line_spans.is_empty() {
                            lines.push(Line::from(current_line_spans.clone()));
                            current_line_spans.clear();
                        }
                    }
                    TagEnd::Item => {
                        lines.push(Line::from(current_line_spans.clone()));
                        current_line_spans.clear();
                    }
                    TagEnd::Paragraph => {
                        if !current_line_spans.is_empty() {
                            lines.push(Line::from(current_line_spans.clone()));
                            current_line_spans.clear();
                        }
                        lines.push(Line::default());
                    }
                    _ => {}
                },
                MdEvent::Text(text) => {
                    if in_code_block {
                        code_block_content.push_str(text.as_ref());
                    } else if in_table && in_table_cell {
                        current_cell.push_str(text.as_ref());
                    } else {
                        let style = if in_bold && in_italic {
                            base_style.add_modifier(Modifier::BOLD | Modifier::ITALIC)
                        } else if in_bold {
                            base_style.add_modifier(Modifier::BOLD)
                        } else if in_italic {
                            base_style.add_modifier(Modifier::ITALIC)
                        } else {
                            base_style
                        };

                        let style = if in_link {
                            style.fg(Color::Blue).add_modifier(Modifier::UNDERLINED)
                        } else {
                            style
                        };

                        for (i, line) in text.as_ref().split('\n').enumerate() {
                            if i > 0 {
                                lines.push(Line::from(current_line_spans.clone()));
                                current_line_spans.clear();
                            }
                            if !line.is_empty() {
                                current_line_spans.push(Span::styled(line.to_string(), style));
                            }
                        }
                    }
                }
                MdEvent::Code(code) => {
                    if in_table && in_table_cell {
                        current_cell.push_str(code.as_ref());
                    } else {
                        current_line_spans.push(Span::styled(
                            format!(" {} ", code.as_ref()),
                            base_style.fg(Color::Yellow).bg(Color::Black),
                        ));
                    }
                }
                MdEvent::SoftBreak => {
                    if in_table && in_table_cell {
                        current_cell.push(' ');
                    } else {
                        current_line_spans.push(Span::raw(" "));
                    }
                }
                MdEvent::HardBreak => {
                    if in_table && in_table_cell {
                        current_cell.push(' ');
                    } else {
                        lines.push(Line::from(current_line_spans.clone()));
                        current_line_spans.clear();
                    }
                }
                _ => {}
            }
        }

        if !current_line_spans.is_empty() {
            lines.push(Line::from(current_line_spans));
        }

        while lines.first().map_or(false, |l| l.spans.is_empty()) {
            lines.remove(0);
        }
        while lines.last().map_or(false, |l| l.spans.is_empty()) {
            lines.pop();
        }

        Text::from(lines)
    }

    fn render_code_block(
        &self,
        content: &str,
        lang: Option<&str>,
        base_style: Style,
    ) -> Vec<Line<'static>> {
        let mut output = Vec::new();
        let label = lang.unwrap_or("code");
        let border_style = base_style.fg(Color::DarkGray).bg(Color::Black);

        let line_count = content.lines().count().max(1);
        let number_width = line_count.to_string().len().max(2);
        let line_number_style = base_style.fg(Color::DarkGray).bg(Color::Black);

        output.push(Line::from(vec![Span::styled(
            format!("╭─ {} ", label),
            border_style,
        )]));

        let syntax = self.syntax_for_lang(lang);
        let theme = self.theme();
        let mut highlighter = HighlightLines::new(syntax, theme);

        for line in content.lines() {
            let line_no = output.len();
            let number = line_no + 1;
            let mut spans = vec![
                Span::styled("│ ", border_style),
                Span::styled(
                    format!("{:>width$} ", number, width = number_width),
                    line_number_style,
                ),
            ];
            match highlighter.highlight_line(line, &self.syntax_set) {
                Ok(ranges) => {
                    for (style, text) in ranges {
                        spans.push(Span::styled(text.to_string(), syntect_to_style(style)));
                    }
                }
                Err(_) => {
                    spans.push(Span::styled(
                        line.to_string(),
                        base_style.fg(Color::Yellow).bg(Color::Black),
                    ));
                }
            }
            output.push(Line::from(spans));
        }

        if content.is_empty() {
            output.push(Line::from(vec![
                Span::styled("│ ", border_style),
                Span::styled(
                    format!("{:>width$} ", 1, width = number_width),
                    line_number_style,
                ),
                Span::styled(" ", base_style.bg(Color::Black)),
            ]));
        }

        output.push(Line::from(vec![Span::styled("╰─", border_style)]));
        output
    }

    fn syntax_for_lang(&self, lang: Option<&str>) -> &syntect::parsing::SyntaxReference {
        if let Some(token) = lang {
            if let Some(syntax) = self.syntax_set.find_syntax_by_token(token) {
                return syntax;
            }
            if let Some(syntax) = self.syntax_set.find_syntax_by_extension(token) {
                return syntax;
            }
        }
        self.syntax_set.find_syntax_plain_text()
    }

    fn theme(&self) -> &Theme {
        self.theme_set
            .themes
            .get("base16-ocean.dark")
            .or_else(|| self.theme_set.themes.values().next())
            .expect("theme set is not empty")
    }
}

fn render_table(rows: &[Vec<String>], header_rows: usize, base_style: Style) -> Vec<Line<'static>> {
    if rows.is_empty() {
        return Vec::new();
    }

    let col_count = rows.iter().map(|r| r.len()).max().unwrap_or(0);
    let mut widths = vec![0usize; col_count];

    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            widths[i] = widths[i].max(cell.len());
        }
    }

    let header_style = base_style
        .fg(Color::Cyan)
        .bg(Color::Black)
        .add_modifier(Modifier::BOLD);
    let row_style = base_style.bg(Color::Black);

    let mut lines = Vec::new();

    for (idx, row) in rows.iter().enumerate() {
        let mut line = String::new();
        for col in 0..col_count {
            let cell = row.get(col).map(String::as_str).unwrap_or("");
            let pad = widths[col].saturating_sub(cell.len());
            line.push_str(cell);
            line.push_str(&" ".repeat(pad));
            if col + 1 < col_count {
                line.push_str(" | ");
            }
        }
        let style = if idx < header_rows {
            header_style
        } else {
            row_style
        };
        lines.push(Line::from(vec![Span::styled(line, style)]));

        if idx + 1 == header_rows {
            let mut sep = String::new();
            for col in 0..col_count {
                sep.push_str(&"-".repeat(widths[col].max(1)));
                if col + 1 < col_count {
                    sep.push_str("-+-");
                }
            }
            lines.push(Line::from(vec![Span::styled(
                sep,
                base_style.fg(Color::DarkGray).bg(Color::Black),
            )]));
        }
    }

    lines
}

fn syntect_to_style(style: SyntectStyle) -> Style {
    let fg = Color::Rgb(style.foreground.r, style.foreground.g, style.foreground.b);
    let mut result = Style::default().fg(fg);

    if style.background.a > 0 {
        let bg = Color::Rgb(style.background.r, style.background.g, style.background.b);
        result = result.bg(bg);
    } else {
        result = result.bg(Color::Black);
    }

    if style
        .font_style
        .contains(syntect::highlighting::FontStyle::BOLD)
    {
        result = result.add_modifier(Modifier::BOLD);
    }
    if style
        .font_style
        .contains(syntect::highlighting::FontStyle::ITALIC)
    {
        result = result.add_modifier(Modifier::ITALIC);
    }
    if style
        .font_style
        .contains(syntect::highlighting::FontStyle::UNDERLINE)
    {
        result = result.add_modifier(Modifier::UNDERLINED);
    }

    result
}
