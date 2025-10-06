# AI Chat TUI 🤖

A modern, terminal-based chat client for interacting with AI models through Hugging Face's API or any OpenAI-compatible API endpoint. Built with Rust for performance and featuring a beautiful TUI (Terminal User Interface).

## Features ✨

- 🎨 **Beautiful TUI**: Clean and intuitive terminal interface with color-coded messages
- 💬 **Real-time Chat**: Seamless conversation with AI models
- 🔄 **Message History**: Scroll through previous messages with arrow keys
- 📊 **Status Bar**: Real-time status updates and error messages
- 🎮 **Interactive Commands**: Built-in commands for clearing chat, viewing stats, and more
- 📖 **Help System**: In-app help overlay with keyboard shortcuts
- 🚀 **Fast & Efficient**: Built with Rust for optimal performance
- 🔧 **Configurable**: Support for different models and API endpoints

## Screenshots

```
┌─────────────────────────────────────────────────────┐
│     🤖 AI Chat Client | meta-llama/Llama-3.2-3B     │
├─────────────────────────────────────────────────────┤
│ Messages                                            │
│                                                     │
│ 👤 You: Hello, how are you today?                  │
│                                                     │
│ 🤖 AI: I'm doing well, thank you for asking!       │
│       How can I help you today?                     │
│                                                     │
├─────────────────────────────────────────────────────┤
│ Input (Press 'i' to edit)                          │
│                                                     │
├─────────────────────────────────────────────────────┤
│ Welcome! Press 'i' to start typing, 'h' for help   │
└─────────────────────────────────────────────────────┘
```

## Prerequisites 📋

- Rust 1.70 or higher
- A Hugging Face account and API token (free at [huggingface.co](https://huggingface.co))
- Terminal with UTF-8 support for emoji display

## Installation 🚀

### From Source

1. Clone the repository:
```bash
git clone https://github.com/yourusername/ai-chat-tui.git
cd ai-chat-tui
```

2. Copy the environment example file and configure:
```bash
cp .env.example .env
```

3. Edit `.env` file with your Hugging Face token:
```env
HUGGINGFACE_TOKEN=your_token_here
```

4. Build and run:
```bash
cargo build --release
cargo run --release
```

### Using Cargo Install

```bash
cargo install --path .
ai-chat-tui
```

## Configuration ⚙️

Create a `.env` file in the project root or set environment variables:

### Required
- `HUGGINGFACE_TOKEN`: Your Hugging Face API token

### Optional
- `HF_BASE_URL`: API endpoint URL (defaults to `https://api-inference.huggingface.co/v1`)
- `HF_MODEL`: Model to use (defaults to `meta-llama/Llama-3.2-3B-Instruct`)

### Available Models

You can use any model available on Hugging Face's Inference API:

- **Llama Models**: `meta-llama/Llama-3.2-3B-Instruct`, `meta-llama/Llama-2-7b-chat-hf`
- **Mistral Models**: `mistralai/Mistral-7B-Instruct-v0.2`, `mistralai/Mixtral-8x7B-Instruct-v0.1`
- **Google Models**: `google/flan-t5-xxl`, `google/flan-ul2`
- **Code Models**: `codellama/CodeLlama-7b-Instruct-hf`, `Salesforce/codegen-2B-multi`

## Usage 🎮

### Keyboard Shortcuts

| Key | Action | Mode |
|-----|--------|------|
| `i` | Enter input mode | Normal |
| `Esc` | Exit input mode | Editing |
| `Enter` | Send message | Editing |
| `h` | Toggle help overlay | Normal |
| `↑` / `↓` | Scroll messages | Normal |
| `q` | Quit application | Normal |
| `Backspace` | Delete character | Editing |

### Chat Commands

Type these commands in the input field:

| Command | Description |
|---------|-------------|
| `/help` or `/h` | Toggle help overlay |
| `/clear` or `/c` | Clear conversation history |
| `/stats` or `/s` | Show conversation statistics |

## Development 🛠️

### Project Structure

```
ai-chat-tui/
├── src/
│   └── main.rs        # Main application code
├── Cargo.toml         # Dependencies and metadata
├── .env.example       # Environment configuration template
└── README.md          # This file
```

### Dependencies

- `openai_api_rs`: OpenAI-compatible API client
- `ratatui`: Terminal UI framework
- `crossterm`: Cross-platform terminal manipulation
- `tokio`: Async runtime
- `dotenv`: Environment variable management
- `serde`: Serialization/deserialization

### Building from Source

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run
```

## Troubleshooting 🔧

### Common Issues

1. **"HUGGINGFACE_TOKEN environment variable not set"**
   - Make sure you have created a `.env` file with your token
   - Or export the variable: `export HUGGINGFACE_TOKEN=your_token`

2. **API Rate Limiting**
   - Free Hugging Face accounts have rate limits
   - Consider upgrading your account or using a different model

3. **Emoji/Unicode Display Issues**
   - Ensure your terminal supports UTF-8
   - On Windows, use Windows Terminal or WSL

4. **Model Not Available**
   - Some models may require authentication or be temporarily unavailable
   - Try a different model or check the model's page on Hugging Face

## Contributing 🤝

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## License 📄

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments 🙏

- Built with [Ratatui](https://github.com/ratatui/ratatui) for the beautiful TUI
- Uses [OpenAI API RS](https://github.com/openai-rs/openai-api) for API communication
- Powered by [Hugging Face](https://huggingface.co) models

## Future Enhancements 🚀

- [ ] Support for multiple conversation threads
- [ ] Save/load conversation history
- [ ] Export conversations to markdown
- [ ] Support for streaming responses
- [ ] Custom themes and color schemes
- [ ] Plugin system for custom commands
- [ ] Support for system prompts
- [ ] Token usage tracking
- [ ] Multi-language support

## Support 💬

If you encounter any issues or have questions, please:
1. Check the [Troubleshooting](#troubleshooting-) section
2. Open an issue on GitHub
3. Contact the maintainers

---

Made with ❤️ and Rust 🦀