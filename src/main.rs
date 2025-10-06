use chrono::{DateTime, Local};
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use openai_api_rs::v1::{
    api::Client as OpenAIClient,
    chat_completion::{ChatCompletionMessage, ChatCompletionRequest, MessageRole},
};
use pulldown_cmark::{Event as MdEvent, HeadingLevel, Parser, Tag, TagEnd};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame, Terminal,
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    fs, io,
    sync::Arc,
    time::{Duration, Instant},
};
use syntect::{highlighting::ThemeSet, parsing::SyntaxSet};
use tokio::sync::mpsc;

/// Configuration for the chat client
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatConfig {
    base_url: String,
    token: String,
    model: String,
    max_tokens: Option<i64>,
    temperature: Option<f64>,
    system_prompt: Option<String>,
    max_context_messages: usize,
}

impl ChatConfig {
    fn from_env() -> Result<Self, Box<dyn Error>> {
        dotenv::dotenv().ok();

        // Check if using local Ollama (no token needed)
        let base_url = std::env::var("HF_BASE_URL")
            .unwrap_or_else(|_| {
                eprintln!("\n⚠️  No HF_BASE_URL set. Please check WORKING_CONFIGS.md for setup instructions.");
                eprintln!("   Tip: For free local usage, install Ollama: brew install ollama && ollama pull llama3.2\n");
                "http://localhost:11434/v1".to_string()
            });

        // Token is optional for local services like Ollama
        let token = std::env::var("HUGGINGFACE_TOKEN").unwrap_or_else(|_| {
            if base_url.contains("localhost") || base_url.contains("127.0.0.1") {
                "unused".to_string() // Local services don't need tokens
            } else {
                eprintln!("\n⚠️  HUGGINGFACE_TOKEN not set but using remote API!");
                eprintln!("   Please set your API token in the .env file.\n");
                "missing-token".to_string()
            }
        });

        let model = std::env::var("HF_MODEL").unwrap_or_else(|_| "llama3.2".to_string());

        let system_prompt = std::env::var("SYSTEM_PROMPT").ok();

        Ok(Self {
            base_url,
            token,
            model,
            max_tokens: Some(500),
            temperature: Some(0.7),
            system_prompt,
            max_context_messages: 20, // Keep last 20 messages
        })
    }
}

/// Local MessageRole wrapper to enable comparison
#[derive(Clone, PartialEq, Serialize, Deserialize)]
enum Role {
    User,
    Assistant,
    System,
}

impl From<MessageRole> for Role {
    fn from(role: MessageRole) -> Self {
        match role {
            MessageRole::user => Role::User,
            MessageRole::assistant => Role::Assistant,
            MessageRole::system => Role::System,
            _ => Role::System,
        }
    }
}

impl Into<MessageRole> for Role {
    fn into(self) -> MessageRole {
        match self {
            Role::User => MessageRole::user,
            Role::Assistant => MessageRole::assistant,
            Role::System => MessageRole::system,
        }
    }
}

/// Message content type
#[derive(Clone, Debug, Serialize, Deserialize)]
enum MessageContent {
    Text(String),
    WithThinking { thinking: String, output: String },
}

/// UI Application state
#[derive(Clone, Serialize, Deserialize)]
struct Message {
    role: Role,
    content: MessageContent,
    #[serde(skip, default = "Instant::now")]
    timestamp: Instant,
    datetime: DateTime<Local>,
}

/// API Response message
#[derive(Debug)]
enum ApiMessage {
    Response(String),
    Error(String),
}

/// UI Application state
struct App {
    client: Arc<OpenAIClient>,
    config: ChatConfig,
    messages: Vec<Message>,
    input: String,
    input_mode: InputMode,
    show_help: bool,
    show_thinking: bool,  // Toggle for showing thinking tokens
    status_message: Option<(String, StatusType)>,
    is_loading: bool,
    loading_frame: usize,  // For animated loader
    scroll_offset: usize,
    scroll_state: ScrollState,  // Improved scroll tracking
    api_receiver: Option<mpsc::UnboundedReceiver<ApiMessage>>,
    #[allow(dead_code)]
    syntax_set: SyntaxSet,
    #[allow(dead_code)]
    theme_set: ThemeSet,
}

#[derive(Clone)]
enum ScrollState {
    Bottom,  // Auto-scroll to bottom
    Fixed(usize),  // Fixed at specific line offset
}

#[derive(PartialEq)]
enum InputMode {
    Normal,
    Editing,
}

#[derive(Clone)]
enum StatusType {
    Info,
    Success,
    Warning,
    Error,
}

/// Parse thinking tokens from response
fn parse_thinking_tokens(content: &str) -> MessageContent {
    // Common patterns for thinking tokens
    // Pattern 1: <thinking>...</thinking>
    // Pattern 2: [THINKING]...[/THINKING]
    // Pattern 3: 🤔 Thinking: ... (until \n\n or end)

    let thinking_regex = Regex::new(
        r"(?s)(<thinking>.*?</thinking>|\[THINKING\].*?\[/THINKING\]|🤔\s*Thinking:.*?(?:\n\n|$))",
    )
    .unwrap();

    if let Some(captures) = thinking_regex.find(content) {
        let thinking = captures.as_str().to_string();
        let output = content.replace(captures.as_str(), "").trim().to_string();

        // Clean up the thinking markers
        let thinking_clean = thinking
            .replace("<thinking>", "")
            .replace("</thinking>", "")
            .replace("[THINKING]", "")
            .replace("[/THINKING]", "")
            .replace("🤔 Thinking:", "")
            .trim()
            .to_string();

        if !output.is_empty() {
            return MessageContent::WithThinking {
                thinking: thinking_clean,
                output,
            };
        }
    }

    MessageContent::Text(content.to_string())
}

/// Convert markdown to styled ratatui Text
fn markdown_to_styled_text(markdown: &str, base_style: Style) -> Text<'static> {
    markdown_to_styled_text_with_syntax(markdown, base_style, None)
}

/// Convert markdown to styled ratatui Text with optional syntax highlighting
fn markdown_to_styled_text_with_syntax(
    markdown: &str,
    base_style: Style,
    _syntax_set: Option<&SyntaxSet>,
) -> Text<'static> {
    let parser = Parser::new(markdown);
    let mut lines = vec![Line::default()];
    let mut current_line_spans = Vec::new();
    let mut in_code_block = false;
    let mut code_block_lang: Option<String> = None;
    let mut code_block_content = String::new();
    let mut in_bold = false;
    let mut in_italic = false;
    let mut list_depth: usize = 0;

    for event in parser {
        match event {
            MdEvent::Start(tag) => match tag {
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
                TagEnd::Heading(_) => {
                    lines.push(Line::from(current_line_spans.clone()));
                    current_line_spans.clear();
                }
                TagEnd::CodeBlock => {
                    in_code_block = false;

                    // Add code block header with language
                    if let Some(lang) = &code_block_lang {
                        lines.push(Line::from(vec![Span::styled(
                            format!("╭─ {} ", lang),
                            base_style.fg(Color::DarkGray),
                        )]));
                    } else {
                        lines.push(Line::from(vec![Span::styled(
                            "╭─ code ",
                            base_style.fg(Color::DarkGray),
                        )]));
                    }

                    // Render code block content (simple coloring for now)
                    for code_line in code_block_content.lines() {
                        lines.push(Line::from(vec![
                            Span::styled("│ ", base_style.fg(Color::DarkGray)),
                            Span::styled(
                                code_line.to_string(),
                                base_style.fg(Color::Yellow).bg(Color::Black),
                            ),
                        ]));
                    }

                    // Add code block footer
                    lines.push(Line::from(vec![Span::styled(
                        "╰─",
                        base_style.fg(Color::DarkGray),
                    )]));

                    code_block_content.clear();
                    code_block_lang = None;

                    if !current_line_spans.is_empty() {
                        lines.push(Line::from(current_line_spans.clone()));
                        current_line_spans.clear();
                    }
                }
                TagEnd::Emphasis => in_italic = false,
                TagEnd::Strong => in_bold = false,
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
                    // Add empty line after paragraph
                    lines.push(Line::default());
                }
                _ => {}
            },
            MdEvent::Text(text) => {
                if in_code_block {
                    // Accumulate code block content for later processing
                    code_block_content.push_str(text.as_ref());
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

                    // Split text by newlines and handle each line
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
                current_line_spans.push(Span::styled(
                    format!(" {} ", code.as_ref()),
                    base_style.fg(Color::Yellow).bg(Color::Black),
                ));
            }
            MdEvent::SoftBreak => {
                current_line_spans.push(Span::raw(" "));
            }
            MdEvent::HardBreak => {
                lines.push(Line::from(current_line_spans.clone()));
                current_line_spans.clear();
            }
            _ => {}
        }
    }

    // Add any remaining spans
    if !current_line_spans.is_empty() {
        lines.push(Line::from(current_line_spans));
    }

    // Remove leading/trailing empty lines
    while lines.first().map_or(false, |l| l.spans.is_empty()) {
        lines.remove(0);
    }
    while lines.last().map_or(false, |l| l.spans.is_empty()) {
        lines.pop();
    }

    Text::from(lines)
}

impl App {
    fn new(config: ChatConfig) -> Result<Self, Box<dyn Error>> {
        println!("\n🔧 API Configuration:");
        println!("  URL: {}", config.base_url);
        println!("  Model: {}", config.model);
        println!(
            "  Token: {}...\n",
            &config.token[..10.min(config.token.len())]
        );

        let client = Arc::new(OpenAIClient::new_with_endpoint(
            config.base_url.clone(),
            config.token.clone(),
        ));

        let mut app = Self {
            client,
            config: config.clone(),
            messages: Vec::new(),
            input: String::new(),
            input_mode: InputMode::Normal,
            show_help: false,
            show_thinking: false,
            status_message: Some((
                "Welcome! Press 'i' to start typing, 'h' for help, 'q' to quit".to_string(),
                StatusType::Info,
            )),
            is_loading: false,
            loading_frame: 0,
            scroll_offset: 0,
            scroll_state: ScrollState::Bottom,
            api_receiver: None,
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
        };

        // Add system prompt if configured
        if let Some(system_prompt) = &config.system_prompt {
            app.add_message(Role::System, system_prompt.clone());
        }

        Ok(app)
    }

    fn add_message(&mut self, role: Role, content: String) {
        let message_content = if role == Role::Assistant {
            // Parse thinking tokens for assistant messages
            parse_thinking_tokens(&content)
        } else {
            MessageContent::Text(content)
        };

        self.messages.push(Message {
            role,
            content: message_content,
            timestamp: Instant::now(),
            datetime: Local::now(),
        });
        // Auto-scroll to bottom when new message arrives
        self.scroll_state = ScrollState::Bottom;
        self.scroll_offset = 0;
    }

    /// Estimate token count (rough: ~4 chars = 1 token)
    fn estimate_tokens(&self, text: &str) -> usize {
        text.len() / 4
    }

    /// Prepare messages for API with context window management
    fn prepare_api_messages(&self) -> Vec<ChatCompletionMessage> {
        let max_messages = self.config.max_context_messages;

        let messages_to_send = if self.messages.len() > max_messages {
            // Keep system message if exists, then most recent messages
            let mut result = Vec::new();

            // Add system message if it exists
            if let Some(first) = self.messages.first() {
                if first.role == Role::System {
                    result.push(first.clone());
                }
            }

            // Add recent messages
            let start_idx = if result.is_empty() {
                self.messages.len().saturating_sub(max_messages)
            } else {
                // If we have system message, take max_messages - 1 recent messages
                self.messages.len().saturating_sub(max_messages - 1).max(1)
            };

            result.extend(self.messages[start_idx..].iter().cloned());
            result
        } else {
            self.messages.clone()
        };

        messages_to_send
            .iter()
            .map(|msg| {
                let text_content = match &msg.content {
                    MessageContent::Text(text) => text.clone(),
                    MessageContent::WithThinking { output, .. } => output.clone(),
                };
                ChatCompletionMessage {
                    role: msg.role.clone().into(),
                    content: text_content,
                    name: None,
                    function_call: None,
                }
            })
            .collect()
    }

    async fn send_message(&mut self) -> Result<(), Box<dyn Error>> {
        if self.input.trim().is_empty() {
            self.status_message =
                Some(("Cannot send empty message".to_string(), StatusType::Warning));
            return Ok(());
        }

        let user_input = self.input.clone();
        self.input.clear();
        self.input_mode = InputMode::Normal;

        // Provide immediate feedback
        eprintln!("[DEBUG] Sending message: {} chars", user_input.len());

        // Handle commands
        if let Some(command) = user_input.strip_prefix('/') {
            self.handle_command(command).await?;
            return Ok(());
        }

        // Add user message
        self.add_message(Role::User, user_input.clone());
        self.is_loading = true;
        self.status_message = Some(("Sending message...".to_string(), StatusType::Info));

        // Prepare API request with context management
        let api_messages = self.prepare_api_messages();

        let mut req = ChatCompletionRequest::new(self.config.model.clone(), api_messages);
        if let Some(max_tokens) = self.config.max_tokens {
            req.max_tokens = Some(max_tokens);
        }
        if let Some(temperature) = self.config.temperature {
            req.temperature = Some(temperature);
        }

        // Create channel for communication
        let (tx, rx) = mpsc::unbounded_channel();
        self.api_receiver = Some(rx);

        // Clone necessary data for the background task
        let client = self.client.clone();

        // Spawn background task for API call
        tokio::spawn(async move {
            eprintln!("[DEBUG] API call started...");
            // Send request in background
            match client.chat_completion(req) {
                Ok(response) => {
                    eprintln!("[DEBUG] API call successful");
                    if let Some(choice) = response.choices.first() {
                        if let Some(content) = choice.message.content.as_ref() {
                            eprintln!("[DEBUG] Response received: {} chars", content.len());
                            let _ = tx.send(ApiMessage::Response(content.clone()));
                        } else {
                            eprintln!("[DEBUG] No content in response");
                            let _ = tx.send(ApiMessage::Error("No content received".to_string()));
                        }
                    } else {
                        eprintln!("[DEBUG] No choices in response");
                        let _ = tx.send(ApiMessage::Error("No response received".to_string()));
                    }
                }
                Err(e) => {
                    eprintln!("[DEBUG] API error: {:?}", e);
                    // Provide helpful error messages
                    let error_msg = if e.to_string().contains("404") {
                        format!("Error 404: API endpoint not found. Check WORKING_CONFIGS.md for valid configurations.")
                    } else if e.to_string().contains("401") {
                        format!("Error 401: Invalid API key. Please check your token.")
                    } else if e.to_string().contains("429") {
                        format!("Error 429: Rate limit exceeded. Please wait and try again.")
                    } else if e.to_string().contains("Connection refused")
                        || e.to_string().contains("connection")
                    {
                        format!("Connection Error: Cannot reach API. Check if service is running and HF_BASE_URL is correct.")
                    } else {
                        format!("API Error: {}. See WORKING_CONFIGS.md for help.", e)
                    };

                    let _ = tx.send(ApiMessage::Error(error_msg));
                }
            }
        });

        Ok(())
    }

    fn process_api_response(&mut self) {
        if let Some(receiver) = &mut self.api_receiver {
            // Try to receive without blocking
            match receiver.try_recv() {
                Ok(msg) => {
                    eprintln!("[DEBUG] Received API response");
                    self.is_loading = false;
                    match msg {
                        ApiMessage::Response(content) => {
                            eprintln!("[DEBUG] Processing successful response");
                            self.add_message(Role::Assistant, content);
                            self.status_message = Some((
                                "✓ Message sent successfully".to_string(),
                                StatusType::Success,
                            ));
                        }
                        ApiMessage::Error(error_msg) => {
                            eprintln!("[DEBUG] Processing error response: {}", error_msg);
                            // Remove the user message if request failed
                            if let Some(last_msg) = self.messages.last() {
                                if last_msg.role == Role::User {
                                    self.messages.pop();
                                }
                            }
                            self.status_message =
                                Some((format!("✗ {}", error_msg), StatusType::Error));
                        }
                    }
                    self.api_receiver = None; // Clear the receiver
                }
                Err(mpsc::error::TryRecvError::Empty) => {
                    // Still waiting for response
                }
                Err(mpsc::error::TryRecvError::Disconnected) => {
                    eprintln!("[DEBUG] API channel disconnected");
                    // Channel closed unexpectedly
                    self.is_loading = false;
                    self.status_message =
                        Some(("✗ API connection lost".to_string(), StatusType::Error));
                    self.api_receiver = None;
                }
            }
        }
    }

    fn save_conversation(&self, filename: &str) -> Result<(), Box<dyn Error>> {
        let json = serde_json::to_string_pretty(&self.messages)?;
        fs::write(filename, json)?;
        Ok(())
    }

    fn load_conversation(&mut self, filename: &str) -> Result<(), Box<dyn Error>> {
        let json = fs::read_to_string(filename)?;
        self.messages = serde_json::from_str(&json)?;
        // Update timestamps for loaded messages
        for msg in &mut self.messages {
            msg.timestamp = Instant::now();
        }
        Ok(())
    }

    async fn handle_command(&mut self, command: &str) -> Result<(), Box<dyn Error>> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        let cmd = parts.first().unwrap_or(&"");

        match *cmd {
            "help" | "h" => {
                self.show_help = !self.show_help;
                self.status_message = Some(("Help toggled".to_string(), StatusType::Info));
            }
            "clear" | "c" => {
                self.messages.clear();
                self.scroll_offset = 0;
                self.status_message =
                    Some(("Conversation cleared".to_string(), StatusType::Success));
            }
            "stats" | "s" => {
                let total = self.messages.len();
                let user_count = self
                    .messages
                    .iter()
                    .filter(|m| m.role == Role::User)
                    .count();
                let assistant_count = self
                    .messages
                    .iter()
                    .filter(|m| m.role == Role::Assistant)
                    .count();
                let total_chars: usize = self
                    .messages
                    .iter()
                    .map(|m| match &m.content {
                        MessageContent::Text(t) => t.len(),
                        MessageContent::WithThinking { thinking, output } => {
                            thinking.len() + output.len()
                        }
                    })
                    .sum();
                let estimated_tokens = self.estimate_tokens(&total_chars.to_string()) * total;
                self.status_message = Some((
                    format!(
                        "Messages: {} (U:{} A:{}) | ~{} tokens",
                        total, user_count, assistant_count, estimated_tokens
                    ),
                    StatusType::Info,
                ));
            }
            "save" => {
                let filename = parts.get(1).unwrap_or(&"conversation.json");
                match self.save_conversation(filename) {
                    Ok(_) => {
                        self.status_message = Some((
                            format!("Saved conversation to {}", filename),
                            StatusType::Success,
                        ));
                    }
                    Err(e) => {
                        self.status_message =
                            Some((format!("Failed to save: {}", e), StatusType::Error));
                    }
                }
            }
            "load" => {
                let filename = parts.get(1).unwrap_or(&"conversation.json");
                match self.load_conversation(filename) {
                    Ok(_) => {
                        self.status_message = Some((
                            format!("Loaded conversation from {}", filename),
                            StatusType::Success,
                        ));
                    }
                    Err(e) => {
                        self.status_message =
                            Some((format!("Failed to load: {}", e), StatusType::Error));
                    }
                }
            }
            _ => {
                self.status_message = Some((
                    format!("Unknown command: /{}", command),
                    StatusType::Warning,
                ));
            }
        }
        Ok(())
    }

    fn scroll_up(&mut self) {
        // Switch to fixed scrolling mode
        self.scroll_state = match self.scroll_state {
            ScrollState::Bottom => ScrollState::Fixed(self.scroll_offset),
            ScrollState::Fixed(offset) => ScrollState::Fixed(offset),
        };
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    fn scroll_down(&mut self) {
        // Try to scroll down
        self.scroll_offset = self.scroll_offset.saturating_add(1);
        // Stay in fixed mode unless we can't scroll anymore
        self.scroll_state = ScrollState::Fixed(self.scroll_offset);
    }
    
    fn scroll_to_bottom(&mut self) {
        self.scroll_state = ScrollState::Bottom;
        self.scroll_offset = 0;
    }
    
    fn update_loader_animation(&mut self) {
        if self.is_loading {
            self.loading_frame = (self.loading_frame + 1) % 8;
        }
    }
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Messages
            Constraint::Length(3), // Input
            Constraint::Length(1), // Status
        ])
        .split(f.area());

    // Header
    let header = Paragraph::new(Text::from(vec![Line::from(vec![
        Span::styled("🤖 ", Style::default().fg(Color::Cyan)),
        Span::styled(
            "AI Chat Client",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled(&app.config.model, Style::default().fg(Color::Yellow)),
    ])]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    )
    .alignment(Alignment::Center);
    f.render_widget(header, chunks[0]);

    // Messages area - now with markdown support and thinking tokens
    let mut all_lines: Vec<Line> = Vec::new();
    
    // Calculate which messages to show based on scroll state
    let messages_to_render: Vec<&Message> = match app.scroll_state {
        ScrollState::Bottom => {
            // Show all messages (will be clipped to viewport)
            app.messages.iter().collect()
        }
        ScrollState::Fixed(_) => {
            // Show from scroll_offset onwards
            app.messages.iter().skip(app.scroll_offset).collect()
        }
    };

    for msg in messages_to_render.iter() {
        let (base_style, prefix, role_color) = match msg.role {
            Role::User => (Style::default().fg(Color::Green), "👤 You", Color::Green),
            Role::Assistant => (Style::default().fg(Color::Blue), "🤖 AI", Color::Blue),
            Role::System => (Style::default().fg(Color::Gray), "⚙️ System", Color::Gray),
        };

        // Format timestamp
        let time_str = msg.datetime.format("%H:%M:%S").to_string();

        // Add role prefix line with timestamp
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
                // Render markdown for all messages
                let rendered = if msg.role == Role::Assistant {
                    markdown_to_styled_text(text, base_style)
                } else {
                    // For user messages, just display as plain text
                    Text::from(text.clone())
                };
                for line in rendered.lines {
                    all_lines.push(line.clone());
                }
            }
            MessageContent::WithThinking { thinking, output } => {
                // Show thinking tokens if enabled
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

                    // Show full thinking content with proper formatting
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

                    // Add separator
                    all_lines.push(Line::from(vec![Span::styled(
                        "  ════════════════════",
                        Style::default().fg(Color::DarkGray),
                    )]));
                } else if !thinking.is_empty() {
                    // Show indicator that thinking tokens exist but are hidden
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

                // Render the actual output with markdown
                let rendered = markdown_to_styled_text(output, base_style);
                for line in rendered.lines {
                    // Indent the output slightly
                    let mut indented_line = vec![Span::raw("  ")];
                    indented_line.extend(line.spans.into_iter());
                    all_lines.push(Line::from(indented_line));
                }
            }
        }

        // Add spacing between messages
        all_lines.push(Line::default());
    }

    // Calculate visible lines based on viewport and scroll state
    let viewport_height = chunks[1].height.saturating_sub(2) as usize;
    let total_lines = all_lines.len();
    
    let visible_lines: Vec<Line> = match app.scroll_state {
        ScrollState::Bottom => {
            // Show the last viewport_height lines
            if total_lines > viewport_height {
                all_lines
                    .into_iter()
                    .skip(total_lines - viewport_height)
                    .collect()
            } else {
                all_lines
            }
        }
        ScrollState::Fixed(_) => {
            // Show from current position
            all_lines
                .into_iter()
                .take(viewport_height)
                .collect()
        }
    };

    // Create scroll position indicator
    let scroll_info = if app.messages.is_empty() {
        String::new()
    } else {
        let scroll_indicator = match app.scroll_state {
            ScrollState::Bottom => " [BOTTOM ↓] ".to_string(),
            ScrollState::Fixed(offset) => {
                format!(" [MSG {}/{}] ", (offset + 1).min(app.messages.len()), app.messages.len())
            }
        };
        scroll_indicator
    };

    let messages_paragraph = Paragraph::new(Text::from(visible_lines))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::White))
                .title(format!(" Messages (Markdown Enabled) {}", scroll_info)),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(messages_paragraph, chunks[1]);

    // Animated Loading indicator
    if app.is_loading {
        let loading_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(40),
                Constraint::Length(30),
                Constraint::Percentage(40),
            ])
            .split(chunks[1])[1];

        // Animated loader frames
        let loader_frames = ["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"];
        let current_frame = loader_frames[app.loading_frame % loader_frames.len()];
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
                    .title(" AI is thinking... "),
            );

        f.render_widget(Clear, loading_area);
        f.render_widget(loading, loading_area);
    }

    // Input area
    let input_style = match app.input_mode {
        InputMode::Normal => Style::default().fg(Color::White),
        InputMode::Editing => Style::default().fg(Color::Green),
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
                .border_style(input_style),
        )
        .wrap(Wrap { trim: false });
    f.render_widget(input, chunks[2]);

    // Set cursor position when editing
    if let InputMode::Editing = app.input_mode {
        f.set_cursor_position((chunks[2].x + app.input.len() as u16 + 1, chunks[2].y + 1));
    }

    // Status bar
    if let Some((message, status_type)) = &app.status_message {
        let status_style = match status_type {
            StatusType::Info => Style::default().fg(Color::Cyan),
            StatusType::Success => Style::default().fg(Color::Green),
            StatusType::Warning => Style::default().fg(Color::Yellow),
            StatusType::Error => Style::default().fg(Color::Red),
        };

        let status = Paragraph::new(message.as_str()).style(status_style);
        f.render_widget(status, chunks[3]);
    }

    // Help overlay
    if app.show_help {
        let help_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(15),
                Constraint::Length(20),
                Constraint::Percentage(15),
            ])
            .split(f.area())[1];

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

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    mut app: App,
) -> Result<(), Box<dyn Error>> {
    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(100); // Reduced tick rate for better responsiveness

    loop {
        terminal.draw(|f| ui(f, &app))?;

        // Update loader animation
        app.update_loader_animation();
        
        // Process any pending API responses
        app.process_api_response();

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match app.input_mode {
                        InputMode::Normal => match key.code {
                            KeyCode::Char('q') => return Ok(()),
                            KeyCode::Char('i') => app.input_mode = InputMode::Editing,
                            KeyCode::Char('h') => app.show_help = !app.show_help,
                            KeyCode::Char('t') => {
                                app.show_thinking = !app.show_thinking;
                                app.status_message = Some((
                                    format!("Thinking tokens: {}", if app.show_thinking { "visible" } else { "hidden" }),
                                    StatusType::Info
                                ));
                            },
                            KeyCode::Up => app.scroll_up(),
                            KeyCode::Down => app.scroll_down(),
                            KeyCode::PageUp => {
                                // Scroll up by 10 messages
                                for _ in 0..10 {
                                    app.scroll_up();
                                }
                            },
                            KeyCode::PageDown => {
                                // Scroll down by 10 messages
                                for _ in 0..10 {
                                    app.scroll_down();
                                }
                            },
                            KeyCode::Home => {
                                // Jump to top
                                app.scroll_offset = 0;
                                app.scroll_state = ScrollState::Fixed(0);
                            },
                            KeyCode::End => {
                                // Jump to bottom
                                app.scroll_to_bottom();
                            },
                            _ => {}
                        },
                        InputMode::Editing => match key.code {
                            KeyCode::Enter => {
                                // Plain Enter sends message, Shift+Enter adds newline
                                if key.modifiers.contains(KeyModifiers::SHIFT) {
                                    // Shift+Enter adds newline for multiline input
                                    app.input.push('\n');
                                } else {
                                    // Plain Enter sends the message
                                    eprintln!("[DEBUG] Send key pressed (Enter)");
                                    if let Err(e) = app.send_message().await {
                                        eprintln!("[DEBUG] Send error: {:?}", e);
                                        app.status_message =
                                            Some((format!("✗ Error: {}", e), StatusType::Error));
                                    } else {
                                        eprintln!("[DEBUG] Send initiated successfully");
                                    }
                                }
                            }
                            KeyCode::Char(c) => {
                                if key.modifiers.contains(KeyModifiers::CONTROL) {
                                    // Handle Ctrl+shortcuts
                                    match c {
                                        's' => {
                                            // Ctrl+S to save
                                            if let Err(e) =
                                                app.save_conversation("conversation.json")
                                            {
                                                app.status_message = Some((
                                                    format!("Save failed: {}", e),
                                                    StatusType::Error,
                                                ));
                                            } else {
                                                app.status_message = Some((
                                                    "Saved!".to_string(),
                                                    StatusType::Success,
                                                ));
                                            }
                                        }
                                        _ => {}
                                    }
                                } else {
                                    app.input.push(c);
                                }
                            }
                            KeyCode::Backspace => {
                                app.input.pop();
                            }
                            KeyCode::Esc => app.input_mode = InputMode::Normal,
                            _ => {}
                        },
                    }
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Load configuration
    let config = ChatConfig::from_env()?;

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run
    let app = App::new(config)?;
    let res = run_app(&mut terminal, app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}
