# Quick Start Guide 🚀

## Test the New Enter-to-Send Feature

### Step 1: Build the Application
```bash
cd /Users/svngoku/Documents/global_projects/Rust/ai-rust/ai-chat-tui
cargo build
```

### Step 2: Configure Your Environment

**Option A: Local Ollama (Recommended for testing - no API key needed)**
```bash
# Install Ollama if you haven't
brew install ollama

# Pull a model
ollama pull llama3.2

# Start Ollama server (in a separate terminal)
ollama serve

# Create .env file
cat > .env << 'EOF'
HF_BASE_URL=http://localhost:11434/v1
HF_MODEL=llama3.2
HUGGINGFACE_TOKEN=unused
EOF
```

**Option B: Hugging Face API**
```bash
# Create .env file with your Hugging Face token
cat > .env << 'EOF'
HF_BASE_URL=https://api-inference.huggingface.co/v1
HF_MODEL=meta-llama/Llama-3.2-3B-Instruct
HUGGINGFACE_TOKEN=your_hf_token_here
EOF
```

### Step 3: Run the Application with Debug Output
```bash
cargo run 2>&1 | tee debug.log
```

### Step 4: Test the New Input System

#### Test 1: Single-Line Message (Most Common)
1. Press `i` → You should see the input border turn **green**
2. Type: `Hello, this is a test message!`
3. Press **Enter** → Message should send immediately
4. Watch for:
   - Debug output: `[DEBUG] Send key pressed (Enter)`
   - Loading indicator: `⏳ Loading...`
   - Success message: `✓ Message sent successfully` (green)
   - AI response appears in the message area

#### Test 2: Multiline Message
1. Press `i` → Input border turns green
2. Type: `This is line 1`
3. Press **Shift+Enter** → Cursor moves to new line (message NOT sent)
4. Type: `This is line 2`
5. Press **Shift+Enter** → Another new line
6. Type: `This is line 3`
7. Press **Enter** → All three lines send as one message
8. Verify the AI receives and responds to the multiline message

#### Test 3: Empty Message Prevention
1. Press `i` → Enter edit mode
2. Press **Enter** without typing anything
3. You should see: `⚠ Cannot send empty message` (yellow warning)

#### Test 4: Other Shortcuts Still Work
- Press `h` → Help overlay should toggle
- Press `t` → Thinking tokens visibility should toggle
- Press `Ctrl+S` (while editing) → Quick save should work
- Press `↑` / `↓` → Should scroll through messages

### Step 5: Verify Debug Output

Your `debug.log` should show something like:

```
[DEBUG] Send key pressed (Enter)
[DEBUG] Sending message: 28 chars
[DEBUG] Send initiated successfully
[DEBUG] API call started...
[DEBUG] API call successful
[DEBUG] Response received: 142 chars
[DEBUG] Received API response
[DEBUG] Processing successful response
```

## Keyboard Shortcuts Reference

| Key | Action |
|-----|--------|
| `i` | Enter input/editing mode |
| `Esc` | Exit input mode (back to normal) |
| **`Enter`** | **Send message** ⭐ NEW! |
| **`Shift+Enter`** | **New line (multiline)** ⭐ NEW! |
| `Backspace` | Delete character |
| `Ctrl+S` | Quick save conversation |
| `h` | Toggle help overlay |
| `t` | Toggle thinking tokens |
| `q` | Quit application |
| `↑` / `↓` | Scroll messages |

## Commands (Type in Input)

| Command | Description |
|---------|-------------|
| `/help` or `/h` | Toggle help |
| `/clear` or `/c` | Clear conversation |
| `/stats` or `/s` | Show statistics |
| `/save [file]` | Save conversation |
| `/load [file]` | Load conversation |

## Visual Indicators

| What You See | Meaning |
|--------------|---------|
| **White border** on input | Normal mode - press `i` to edit |
| **Green border** on input | Editing mode - ready to type |
| `⏳ Loading...` | Waiting for API response |
| `✓ Message sent successfully` (green) | Success! |
| `✗ Error: ...` (red) | Something went wrong |
| `⚠ Cannot send empty message` (yellow) | Type something first |

## Troubleshooting

### "Connection Error: Cannot reach API"
- **If using Ollama**: Make sure `ollama serve` is running
- **If using HuggingFace**: Check your API token and internet connection
- Test connection:
  ```bash
  curl -X POST http://localhost:11434/v1/chat/completions \
    -H "Content-Type: application/json" \
    -d '{"model": "llama3.2", "messages": [{"role": "user", "content": "hi"}]}'
  ```

### "Message doesn't send"
- Make sure you're in **editing mode** (green border)
- Make sure you've typed at least one character
- Check `debug.log` for error messages

### "Shift+Enter doesn't work"
- Some terminals may not support Shift+Enter properly
- Try using iTerm2 or a different terminal emulator on macOS
- As a workaround, you can still type single-line messages with Enter

## What Changed?

**Before (Old Behavior):**
- Ctrl+Enter or Shift+Enter: Send message
- Plain Enter: Add newline

**After (New Behavior - Standard Chat UX):**
- **Plain Enter: Send message** ⭐
- **Shift+Enter: Add newline** ⭐

This matches Discord, Slack, WhatsApp Web, and most modern chat applications!

## Next Steps

Once you've tested and confirmed everything works:

1. ✅ Review the changes in `CHANGES_SUMMARY.md`
2. ✅ Check the troubleshooting guide in `TROUBLESHOOTING.md`
3. ✅ If everything works, you're ready to use the app!

## Need Help?

Check these files:
- `TROUBLESHOOTING.md` - Detailed troubleshooting guide
- `CHANGES_SUMMARY.md` - Summary of all changes
- `README.md` - General project information
- `WORKING_CONFIGS.md` - Working API configurations

Or run with debug output and check the logs:
```bash
cargo run 2>&1 | tee debug.log
# Then examine debug.log for error messages
```

---

**Happy chatting! 🎉**
