use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};

use crate::app::{App, InputMode, ScrollState, StatusType};

pub async fn handle_event(app: &mut App, event: Event) -> Result<bool, Box<dyn std::error::Error>> {
    if let Event::Key(key) = event {
        if key.kind == KeyEventKind::Press {
            match app.input_mode {
                InputMode::Normal => match key.code {
                    KeyCode::Char('q') => return Ok(true),
                    KeyCode::Char('i') => app.input_mode = InputMode::Editing,
                    KeyCode::Char('h') => app.show_help = !app.show_help,
                    KeyCode::Char('t') => {
                        app.show_thinking = !app.show_thinking;
                        app.status_message = Some((
                            format!(
                                "Thinking tokens: {}",
                                if app.show_thinking { "visible" } else { "hidden" }
                            ),
                            StatusType::Info,
                        ));
                    }
                    KeyCode::Up => app.scroll_up(),
                    KeyCode::Down => app.scroll_down(),
                    KeyCode::PageUp => {
                        for _ in 0..10 {
                            app.scroll_up();
                        }
                    }
                    KeyCode::PageDown => {
                        for _ in 0..10 {
                            app.scroll_down();
                        }
                    }
                    KeyCode::Home => {
                        app.scroll_offset = 0;
                        app.scroll_state = ScrollState::Fixed(0);
                    }
                    KeyCode::End => {
                        app.scroll_to_bottom();
                    }
                    _ => {}
                },
                InputMode::Editing => match key.code {
                    KeyCode::Enter => {
                        if key.modifiers.contains(KeyModifiers::SHIFT) {
                            app.input.push('\n');
                        } else {
                            if let Err(e) = app.send_message().await {
                                app.status_message =
                                    Some((format!("✗ Error: {}", e), StatusType::Error));
                            }
                        }
                    }
                    KeyCode::Char(c) => {
                        if key.modifiers.contains(KeyModifiers::CONTROL) {
                            match c {
                                's' => {
                                    if let Err(e) = app.save_conversation("conversation.json") {
                                        app.status_message = Some((
                                            format!("Save failed: {}", e),
                                            StatusType::Error,
                                        ));
                                    } else {
                                        app.status_message =
                                            Some(("Saved!".to_string(), StatusType::Success));
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
    Ok(false)
}
