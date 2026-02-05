use crate::{
    config::ChatConfig,
    types::{Message, MessageContent, Role},
};
use openai_api_rs::v1::{
    api::Client as OpenAIClient,
    chat_completion::{ChatCompletionMessage, ChatCompletionRequest, MessageRole},
};
use regex::Regex;
use std::{
    fs,
    sync::Arc,
    time::Instant,
};
use tokio::sync::mpsc;

#[derive(Clone)]
pub enum ScrollState {
    Bottom,
    Fixed(usize),
}

#[derive(PartialEq)]
pub enum InputMode {
    Normal,
    Editing,
}

#[derive(Clone)]
pub enum StatusType {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Debug)]
pub enum ApiMessage {
    Response(String),
    Error(String),
}

pub struct App {
    pub client: Arc<OpenAIClient>,
    pub config: ChatConfig,
    pub messages: Vec<Message>,
    pub input: String,
    pub input_mode: InputMode,
    pub show_help: bool,
    pub show_thinking: bool,
    pub status_message: Option<(String, StatusType)>,
    pub is_loading: bool,
    pub loading_frame: usize,
    pub scroll_offset: usize,
    pub scroll_state: ScrollState,
    pub api_receiver: Option<mpsc::UnboundedReceiver<ApiMessage>>,
}

impl App {
    pub fn new(config: ChatConfig) -> Result<Self, Box<dyn std::error::Error>> {
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
        };

        if let Some(system_prompt) = &config.system_prompt {
            app.add_message(Role::System, system_prompt.clone());
        }

        Ok(app)
    }

    pub fn add_message(&mut self, role: Role, content: String) {
        let message_content = if role == Role::Assistant {
            parse_thinking_tokens(&content)
        } else {
            MessageContent::Text(content)
        };

        self.messages.push(Message {
            role,
            content: message_content,
            timestamp: Instant::now(),
            datetime: chrono::Local::now(),
        });
        self.scroll_state = ScrollState::Bottom;
        self.scroll_offset = 0;
    }

    pub fn estimate_tokens(&self, text: &str) -> usize {
        text.len() / 4
    }

    pub fn prepare_api_messages(&self) -> Vec<ChatCompletionMessage> {
        let max_messages = self.config.max_context_messages;

        let messages_to_send = if self.messages.len() > max_messages {
            let mut result = Vec::new();

            if let Some(first) = self.messages.first() {
                if first.role == Role::System {
                    result.push(first.clone());
                }
            }

            let start_idx = if result.is_empty() {
                self.messages.len().saturating_sub(max_messages)
            } else {
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

    pub async fn send_message(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.input.trim().is_empty() {
            self.status_message =
                Some(("Cannot send empty message".to_string(), StatusType::Warning));
            return Ok(());
        }

        let user_input = self.input.clone();
        self.input.clear();
        self.input_mode = InputMode::Normal;


        if let Some(command) = user_input.strip_prefix('/') {
            self.handle_command(command).await?;
            return Ok(());
        }

        self.add_message(Role::User, user_input.clone());
        self.is_loading = true;
        self.status_message = Some(("Sending message...".to_string(), StatusType::Info));

        let api_messages = self.prepare_api_messages();

        let mut req = ChatCompletionRequest::new(self.config.model.clone(), api_messages);
        if let Some(max_tokens) = self.config.max_tokens {
            req.max_tokens = Some(max_tokens);
        }
        if let Some(temperature) = self.config.temperature {
            req.temperature = Some(temperature);
        }

        let (tx, rx) = mpsc::unbounded_channel();
        self.api_receiver = Some(rx);

        let client = self.client.clone();

        tokio::spawn(async move {
            match client.chat_completion(req) {
                Ok(response) => {
                    if let Some(choice) = response.choices.first() {
                        if let Some(content) = choice.message.content.as_ref() {
                            let _ = tx.send(ApiMessage::Response(content.clone()));
                        } else {
                            let _ = tx.send(ApiMessage::Error("No content received".to_string()));
                        }
                    } else {
                        let _ = tx.send(ApiMessage::Error("No response received".to_string()));
                    }
                }
                Err(e) => {
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

    pub fn process_api_response(&mut self) {
        if let Some(receiver) = &mut self.api_receiver {
            match receiver.try_recv() {
                Ok(msg) => {
                    self.is_loading = false;
                    match msg {
                        ApiMessage::Response(content) => {
                            self.add_message(Role::Assistant, content);
                            self.status_message = Some((
                                "✓ Message sent successfully".to_string(),
                                StatusType::Success,
                            ));
                        }
                        ApiMessage::Error(error_msg) => {
                            if let Some(last_msg) = self.messages.last() {
                                if last_msg.role == Role::User {
                                    self.messages.pop();
                                }
                            }
                            self.status_message =
                                Some((format!("✗ {}", error_msg), StatusType::Error));
                        }
                    }
                    self.api_receiver = None;
                }
                Err(mpsc::error::TryRecvError::Empty) => {}
                Err(mpsc::error::TryRecvError::Disconnected) => {
                    self.is_loading = false;
                    self.status_message =
                        Some(("✗ API connection lost".to_string(), StatusType::Error));
                    self.api_receiver = None;
                }
            }
        }
    }

    pub fn save_conversation(&self, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(&self.messages)?;
        fs::write(filename, json)?;
        Ok(())
    }

    pub fn load_conversation(&mut self, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = fs::read_to_string(filename)?;
        self.messages = serde_json::from_str(&json)?;
        for msg in &mut self.messages {
            msg.timestamp = Instant::now();
        }
        Ok(())
    }

    pub async fn handle_command(
        &mut self,
        command: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
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
                let user_count = self.messages.iter().filter(|m| m.role == Role::User).count();
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

    pub fn scroll_up(&mut self) {
        self.scroll_state = match self.scroll_state {
            ScrollState::Bottom => ScrollState::Fixed(self.scroll_offset),
            ScrollState::Fixed(offset) => ScrollState::Fixed(offset),
        };
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    pub fn scroll_down(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_add(1);
        self.scroll_state = ScrollState::Fixed(self.scroll_offset);
    }

    pub fn scroll_to_bottom(&mut self) {
        self.scroll_state = ScrollState::Bottom;
        self.scroll_offset = 0;
    }

    pub fn update_loader_animation(&mut self) {
        if self.is_loading {
            self.loading_frame = (self.loading_frame + 1) % 8;
        }
    }
}

fn parse_thinking_tokens(content: &str) -> MessageContent {
    let thinking_regex = Regex::new(
        r"(?s)(<thinking>.*?</thinking>|\[THINKING\].*?\[/THINKING\]|🤔\s*Thinking:.*?(?:\n\n|$))",
    )
    .unwrap();

    if let Some(captures) = thinking_regex.find(content) {
        let thinking = captures.as_str().to_string();
        let output = content.replace(captures.as_str(), "").trim().to_string();

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
