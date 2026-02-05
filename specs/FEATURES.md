# AI Chat TUI - Advanced Features

## 🤔 Thinking Tokens Support

The chat client now automatically detects and separates "thinking tokens" from the actual model output. This is especially useful for reasoning models like OpenAI's o1 or Claude's thinking process.

### Supported Thinking Token Formats

The client detects the following patterns:

1. **XML-style tags**: `<thinking>...</thinking>`
2. **Bracket notation**: `[THINKING]...[/THINKING]`
3. **Emoji prefix**: `🤔 Thinking: ...`

### How It Works

- When the assistant responds, the client automatically parses the response
- Thinking tokens are extracted and stored separately
- The actual output is displayed as the main response
- Press `t` key to toggle visibility of thinking tokens

### Display Modes

- **Hidden** (default): Shows only `[Thinking hidden - press 't' to show]` indicator
- **Visible**: Shows full thinking process in purple/magenta italic text with the output below

## 📝 Markdown Rendering

All AI responses are now rendered with full markdown support, including:

### Supported Markdown Features

- **Headers** (H1-H6) - Displayed in cyan with bold formatting
- **Bold text** - Rendered with bold modifier
- **Italic text** - Rendered with italic modifier
- **Code inline** - Yellow text on black background
- **Code blocks** - Green text on black background
- **Lists** - Properly indented with bullet points
  - Nested lists supported
  - Multiple levels of indentation
- **Paragraphs** - Proper spacing between paragraphs

### Visual Enhancements

- Color-coded elements for better readability
- Proper indentation and spacing
- Code highlighting for better visibility
- Clean separation between different content types

## 🎮 New Keyboard Shortcuts

| Key | Function | Mode |
|-----|----------|------|
| `t` | Toggle thinking tokens visibility | Normal |

## Usage Examples

### Example with Thinking Tokens

When a model responds with thinking tokens:

```
<thinking>
I need to analyze this request carefully...
The user is asking about...
</thinking>

Here's my response to your question...
```

The client will:
1. Extract the thinking part
2. Display it separately (if toggled on)
3. Show the clean response

### Example with Markdown

When the model responds with markdown:

```markdown
# Title

Here's a **bold** statement and some *italic* text.

## Code Example

```python
def hello():
    print("Hello, World!")
```

- First item
- Second item
  - Nested item
```

The client will render this with:
- Cyan headers
- Bold and italic text properly formatted
- Code blocks with syntax highlighting
- Properly indented lists

## Configuration

No additional configuration is needed. These features work automatically with any OpenAI-compatible API that returns markdown or thinking tokens in the response.

## Tips

1. **For best results with thinking tokens**: Use reasoning models that output thinking processes
2. **For markdown**: Most modern LLMs automatically format responses in markdown
3. **Toggle thinking**: Use the `t` key to quickly show/hide thinking tokens without interrupting your conversation
4. **Scrolling**: Long responses with markdown are properly formatted and can be scrolled with arrow keys