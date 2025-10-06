# Working API Configurations

The `openai-api-rs` library expects OpenAI-compatible API endpoints. Here are several working options:

## Option 1: Use OpenAI API (Recommended)

```env
HF_BASE_URL=https://api.openai.com/v1
HUGGINGFACE_TOKEN=sk-your-openai-api-key-here
HF_MODEL=gpt-3.5-turbo
```

## Option 2: Use Together AI (OpenAI-Compatible)

Together AI provides OpenAI-compatible endpoints for open-source models:

```env
HF_BASE_URL=https://api.together.xyz/v1
HUGGINGFACE_TOKEN=your-together-api-key
HF_MODEL=meta-llama/Llama-3.2-3B-Instruct
```

Get a free API key at: https://api.together.xyz/

## Option 3: Use Groq (OpenAI-Compatible, Fast)

Groq provides very fast inference with OpenAI-compatible API:

```env
HF_BASE_URL=https://api.groq.com/openai/v1
HUGGINGFACE_TOKEN=your-groq-api-key
HF_MODEL=llama3-8b-8192
```

Get a free API key at: https://console.groq.com/

## Option 4: Use Anyscale (OpenAI-Compatible)

```env
HF_BASE_URL=https://api.endpoints.anyscale.com/v1
HUGGINGFACE_TOKEN=your-anyscale-api-key
HF_MODEL=meta-llama/Llama-2-7b-chat-hf
```

## Option 5: Use Local Ollama (Free, Local)

First, install and run Ollama with a model:
```bash
# Install ollama (macOS)
brew install ollama

# Start ollama and pull a model
ollama serve
ollama pull llama3.2

# Or use a smaller model
ollama pull phi3
```

Then configure:
```env
HF_BASE_URL=http://localhost:11434/v1
HUGGINGFACE_TOKEN=unused
HF_MODEL=llama3.2
```

## Option 6: Use Perplexity AI

```env
HF_BASE_URL=https://api.perplexity.ai
HUGGINGFACE_TOKEN=pplx-your-api-key
HF_MODEL=llama-3.1-sonar-small-128k-online
```

## Option 7: Use DeepInfra (OpenAI-Compatible)

```env
HF_BASE_URL=https://api.deepinfra.com/v1/openai
HUGGINGFACE_TOKEN=your-deepinfra-api-key
HF_MODEL=meta-llama/Llama-3.2-3B-Instruct
```

Get API key at: https://deepinfra.com/

## Why Hugging Face Direct API Doesn't Work

Hugging Face's Inference API uses a different format than OpenAI's chat completions API:
- HF uses: `POST https://api-inference.huggingface.co/models/{model-id}`
- OpenAI uses: `POST https://api.openai.com/v1/chat/completions`

The request/response formats are also different. To use Hugging Face models with OpenAI-compatible clients, you need:
1. A service that provides OpenAI-compatible endpoints (like Together AI, Groq, etc.)
2. Or use Hugging Face's Text Generation Inference (TGI) servers with OpenAI compatibility enabled
3. Or modify the code to use Hugging Face's native API format

## Testing Your Configuration

After updating your `.env` file with one of the above configurations, test it:

```bash
cargo run --release
```

Then press 'i' to enter input mode and type a message like "Hello!" to test the connection.