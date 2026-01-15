# Scriptoria

[![CI](https://github.com/tarunJeevan/scriptoria/workflows/CI/badge.svg)](https://github.com/tarunJeevan/scriptoria/actions)
**AI-Enhanced Creative Writing Studio** | Local-first | Privacy-respecting | Offline-capable

A writing tool designed to give users a distraction free writing experience while leveraging the power of AI to provide
a writing assistant while maintaining security, privacy, and ownership of all of the user's content.

## Usage

...

---

## Overview

Scriptoria is a modular, AI-enhanced creative writing ecosystem designed as a local-first, privacy-respecting platform. It aims to replace fragmented writing toolchains with an integrated workspace combining intelligent assistants, rich editing, knowledge management, and deep personalization.

**Core Vision**: A personal creative studio - offline-capable, deeply customizable, and AI-aware.

### Key Features

- 📝 **Rich Text Editing** - Powered by Tiptap with custom extensions
- 🤖 **Local AI Assistance** - 1B-7B parameter models via Ollama (offline)
- 🗂️ **Knowledge Management** - Characters, timelines, maps, worldbuilding tools
- 🔒 **Privacy-First** - End-to-end encryption, local-only by default
- 🎨 **Customizable Workspace** - Multi-pane layouts, project organization
- 🌐 **Cross-Platform** - Windows, Linux (macOS optional)

---

## Technology Stack

### Core
- **Frontend**: SvelteKit 5 + TypeScript
- **Backend**: Rust + Tauri v2
- **Database**: SQLite + SQLCipher (encrypted)
- **Editor**: Tiptap (extensible rich text)

### AI Infrastructure
- **Local Inference**: Ollama (1B-7B models)
- **Embeddings**: all-MiniLM-L6-v2
- **Vector Store**: FAISS/Chroma
- **Fine-tuning**: LoRA/QLoRA pipeline

### Security
- **Encryption**: ChaCha20-Poly1305 (AEAD)
- **Key Derivation**: Argon2id (64MB memory, 3 iterations)
- **Key Storage**: System keyring (Windows Credential Manager, Linux Secret Service)

---

## Quick Start

### Prerequisites

- **Node.js** 20+ (for frontend)
- **pnpm** 8+ (package manager)
- **Rust** 1.70+ (via [rustup](https://rustup.rs/))
- **System Dependencies**:
  - **Ubuntu/Debian**:
    ```bash
    sudo apt-get install -y \
      libwebkit2gtk-4.1-dev \
      build-essential \
      curl \
      wget \
      file \
      libxdo-dev \
      libssl-dev \
      libayatana-appindicator3-dev \
      librsvg2-dev \
      libdbus-1-dev \
      libsqlite3-dev \
      gnome-keyring
    ```
  - **Windows**: No additional dependencies needed

### Installation

```bash
# Clone repository
git clone https://github.com/tarunJeevan/scriptoria.git
cd scriptoria

# Install frontend dependencies
pnpm install

# Generate SvelteKit types
pnpm check

# Run in development mode
pnpm tauri dev
```

### Building for Production

```bash
# Create distributable packages
pnpm tauri build

# Outputs:
# - Ubuntu: .deb, .AppImage (in src-tauri/target/release/bundle/)
# - Windows: .msi, .exe (in src-tauri/target/release/bundle/)
```

---

## Development

### Project Structure

```
scriptoria/
├── src/                    # SvelteKit frontend
│   ├── lib/               # Shared components, stores, utilities
│   ├── routes/            # Application pages
│   └── app.html           # HTML template
├── src-tauri/             # Rust backend
│   ├── src/
│   │   ├── main.rs        # Entry point
│   │   ├── lib.rs         # Application logic
│   │   ├── commands/      # Tauri IPC commands
│   │   ├── models/        # Database models
│   │   ├── db/            # Database operations
│   │   ├── encryption/    # Encryption service
│   │   └── ai/            # AI inference
│   ├── Cargo.toml         # Rust dependencies
│   └── tauri.conf.json    # Tauri configuration
├── package.json           # Frontend dependencies
├── vite.config.ts         # Vite bundler config
└── svelte.config.js       # SvelteKit config
```

### Available Scripts

**Frontend:**
```bash
pnpm dev            # Start Vite dev server
pnpm build          # Build frontend
pnpm check          # Type check
pnpm lint           # Run ESLint
pnpm format         # Format code with Prettier
```

**Backend:**
```bash
cd src-tauri
cargo fmt           # Format Rust code
cargo clippy        # Lint Rust code
cargo test          # Run tests
cargo build         # Build backend
```

**Tauri:**
```bash
pnpm tauri dev      # Development mode (hot reload)
pnpm tauri build    # Production build
```

---

## Development Phases

### Phase 1: MVP Foundation (Current)
- ✅ **Chunk 1**: Project infrastructure
- ⏳ **Chunk 0**: Database schema & encryption
- ⏳ **Chunk 2**: Rich text editor (Tiptap)
- ⏳ **Chunk 3-4**: Document management
- ⏳ **Chunk 5-7**: AI inference & chat interface
- ⏳ **Chunk 8**: Style adaptation (LoRA)

### Phase 2: Knowledge Base Features
- Notes system (bidirectional linking)
- Character creator
- Timeline designer
- Map system

### Phase 3: Advanced Features
- Multi-pane workspace
- E2E encrypted sync (optional)
- Performance optimization

---

## Security & Privacy

### Local-First Architecture
- All data stored locally by default
- No telemetry or analytics
- Optional cloud sync (explicit opt-in)

### Encryption
- **Database**: SQLCipher with ChaCha20 cipher
- **Files**: ChaCha20-Poly1305 per-file encryption
- **Key Management**: Master key never persisted, derived on-demand

### Threat Model
**Protects against:**
- Physical device theft
- Unauthorized local access
- Offline attacks

**Does NOT protect against:**
- Malware with user privileges
- Keyloggers
- Nation-state actors

See [security_architecture.md](docs/security_architecture.md) for details.

---

## Contributing

We welcome contributions! Please read [CONTRIBUTING.md](CONTRIBUTING.md) for:
- Development setup
- Code style guidelines
- Testing requirements
- Pull request process

### Quick Contribution Guide

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests and linting:
   ```bash
   pnpm format && pnpm lint && pnpm check
   cd src-tauri && cargo fmt -- --check && cargo clippy -- -D warnings && cargo test
   ```
5. Commit (`git commit -m 'Add amazing feature'`)
6. Push (`git push origin feature/amazing-feature`)
7. Open a Pull Request

---

## Roadmap

**Q1 2025**:
- ✅ Phase 1: Chunk 1 (Project infrastructure)
- ⏳ Phase 1: Chunks 0-8 (MVP foundation)

**Q2 2025**:
- Phase 2: Knowledge base features
- Public beta release

**Q3 2025**:
- Phase 3: Advanced features
- Plugin system

See [project-roadmap.md](docs/project-roadmap.md) for detailed roadmap.

---

## License

This project is licensed under the ? License.

---

## Acknowledgments

- **Tauri** - Desktop application framework
- **SvelteKit** - Frontend framework
- **Ollama** - Local LLM inference
- **Tiptap** - Rich text editor
- **RustSec** - Security advisories

---

## Support

- 📖 **Documentation**: [docs/](..docs/)
- 🐛 **Bug Reports**: [GitHub Issues](https://github.com/tarunJeevan/scriptoria/issues)
- 💬 **Discussions**: [GitHub Discussions](https://github.com/tarunJeevan/scriptoria/discussions)
- 📧 **Security**: See [SECURITY.md](SECURITY.md)

---

**Built with ❤️ for writers, by writers**
