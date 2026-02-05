# Troubleshooting Guide for AI Chat TUI

## Input Sending Issues - Fixed! ✅

### What Was Fixed

1. **Enhanced Debug Logging**: Added comprehensive debug output to trace message flow
2. **Better Error Messages**: Improved error handling with clear, actionable feedback
3. **Alternative Send Key**: Added **Shift+Enter** as an alternative to Ctrl+Enter
4. **Empty Message Prevention**: Added validation to prevent sending empty messages
5. **Connection Error Detection**: Better handling of connection issues

### How to Send Messages

#### Single-Line Messages (Default)
1. Press **'i'** to enter editing mode
2. Type your message
3. Press **Enter** to send

#### Multiline Messages
1. Press **'i'** to enter editing mode
2. Type your message
3. Press **Shift+Enter** to add new lines
4. Press **Enter** to send when done

**Note**: This is the standard chat UX - just like Discord, Slack, or most messaging apps!

### Debug Mode

Run the application with debug output to see what's happening:

```bash
# Run with debug output visible
cargo run 2>&1 | tee debug.log
```

You'll see debug messages like:
```
[DEBUG] Sending message: 25 chars
[DEBUG] API call started...
[DEBUG] API call successful
[DEBUG] Response received: 150 chars
[DEBUG] Received API response
[DEBUG] Processing successful response
```

### Common Issues & Solutions

#### 1. **Message not sending**

**Symptoms**: You press Enter but nothing happens

**Solutions**:
- ✅ Make sure you're in **editing mode** (press 'i' first)
- ✅ Look for the green border around the input box (indicates editing mode)
- ✅ Make sure you typed at least one character
- ✅ Check the status bar for error messages
- ✅ Run with debug output: `cargo run 2>&1 | tee debug.log`

#### 2. **Connection errors**

**Symptoms**: "Connection Error: Cannot reach API"

**Solutions**:
```bash
# Check your .env file
cat .env

# For local Ollama (no API key needed)
HF_BASE_URL=http://localhost:11434/v1
HF_MODEL=llama3.2

# For Hugging Face (requires API key)
HF_BASE_URL=https://api-inference.huggingface.co/v1
HUGGINGFACE_TOKEN=hf_your_token_here
HF_MODEL=meta-llama/Llama-3.2-3B-Instruct

# Test connection
curl -X POST http://localhost:11434/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "llama3.2",
    "messages": [{"role": "user", "content": "hi"}]
  }'
```

#### 3. **Empty message warning**

**Symptoms**: "Cannot send empty message"

**Solution**: Type at least one character before sending

#### 4. **API errors**

**Error 404**: Model or endpoint not found
```bash
# Check your model name
echo $HF_MODEL

# Common valid models:
# - llama3.2 (Ollama)
# - meta-llama/Llama-3.2-3B-Instruct (HuggingFace)
```

**Error 401**: Invalid API token
```bash
# Get a new token from https://huggingface.co/settings/tokens
# Update your .env file
```

**Error 429**: Rate limit exceeded
```bash
# Wait a few minutes or use local Ollama instead
```

### Verify Your Setup

```bash
# 1. Check .env exists and is configured
test -f .env && echo "✓ .env exists" || echo "✗ Create .env from .env.example"

# 2. Check environment variables
source .env
echo "Base URL: $HF_BASE_URL"
echo "Model: $HF_MODEL"
echo "Token: ${HUGGINGFACE_TOKEN:0:10}..."

# 3. Test build
cargo build

# 4. Run with debug
cargo run 2>&1 | tee debug.log
```

### Expected Behavior Flow

1. **Start app**: `cargo run`
2. **Enter input mode**: Press **'i'** → Input border turns green
3. **Type message**: "Hello, how are you?"
4. **Send**: Press **Enter**
5. **See feedback**:
   ```
   [DEBUG] Send key pressed (Enter)
   [DEBUG] Sending message: 18 chars
   [DEBUG] Send initiated successfully
   [DEBUG] API call started...
   [DEBUG] API call successful
   [DEBUG] Response received: 150 chars
   [DEBUG] Received API response
   [DEBUG] Processing successful response
   ```
6. **Status bar**: "✓ Message sent successfully" (green)
7. **Response appears**: AI response shows in the message area

### Visual Indicators

| Indicator | Meaning |
|-----------|---------|
| White border on input | Normal mode - press 'i' to edit |
| Green border on input | Editing mode - ready to type |
| "⏳ Loading..." | Waiting for API response |
| "✓ Message sent successfully" (green) | Success! |
| "✗ Error: ..." (red) | Something went wrong |
| "⚠ Cannot send empty message" (yellow) | Type something first |

### Quick Test

```bash
# 1. Start the app
cargo run

# 2. In the TUI:
# - Press 'i' (enter edit mode - border turns green)
# - Type: "test message"
# - Press Enter to send
# - Watch the status bar and message area

# 3. If it doesn't work:
# - Exit with 'q'
# - Run: cargo run 2>&1 | tee debug.log
# - Repeat steps
# - Check debug.log for error details
```

### Getting Help

If you're still having issues:

1. Run with debug output: `cargo run 2>&1 | tee debug.log`
2. Try to send a message
3. Check `debug.log` for error messages
4. Look for patterns in the debug output:
   - Does it reach "API call started"?
   - Does it show "API call successful"?
   - Are there any error messages?

### Local Development Setup (Recommended)

For easiest testing without API keys:

```bash
# Install Ollama (local AI server)
brew install ollama

# Pull a model
ollama pull llama3.2

# Start Ollama (in a separate terminal)
ollama serve

# Update your .env
cat > .env << 'EOF'
HF_BASE_URL=http://localhost:11434/v1
HF_MODEL=llama3.2
HUGGINGFACE_TOKEN=unused
EOF

# Run your app
cargo run
```

## Additional Features

### Keyboard Shortcuts

- **'i'**: Enter input/editing mode
- **Esc**: Exit input mode (back to normal)
- **Enter**: Send message
- **Shift+Enter**: New line (multiline input)
- **'h'**: Toggle help overlay
- **'t'**: Toggle thinking tokens visibility
- **'q'**: Quit application
- **↑/↓**: Scroll through messages
- **Ctrl+S**: Quick save (while editing)

### Commands

Type these in the input box:

- `/help` or `/h` - Toggle help
- `/clear` or `/c` - Clear conversation
- `/stats` or `/s` - Show statistics
- `/save [filename]` - Save conversation
- `/load [filename]` - Load conversation

## Success Checklist ✓

- [ ] .env file exists and configured
- [ ] Ollama running (for local) OR valid API token (for remote)
- [ ] App builds successfully: `cargo build`
- [ ] Can enter edit mode (press 'i', border turns green)
- [ ] Can type in input field
- [ ] Can send with Enter
- [ ] See "Loading..." indicator
- [ ] See success message in status bar
- [ ] See AI response in message area
