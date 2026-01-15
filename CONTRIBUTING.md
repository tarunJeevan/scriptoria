# Contributing to Scriptoria

Thank you for your interest in contributing to Scriptoria! This document provides guidelines and instructions for developers.

---

## Table of Contents

1. [Code of Conduct](#code-of-conduct)
2. [Getting Started](#getting-started)
3. [Development Workflow](#development-workflow)
4. [Code Style](#code-style)
5. [Testing](#testing)
6. [Pull Request Process](#pull-request-process)
7. [Project Structure](#project-structure)
8. [Architecture Decisions](#architecture-decisions)

---

## Code of Conduct

### Our Standards

- **Be respectful** and considerate in all interactions
- **Be collaborative** - we're building something together
- **Be constructive** - critique ideas, not people
- **Be patient** - everyone was new once

### Unacceptable Behavior

- Harassment, discrimination, or personal attacks
- Trolling, inflammatory comments, or spam
- Publishing others' private information

Violations will result in removal from the project.

---

## Getting Started

### Prerequisites

Install these tools before contributing:

**Required:**
- [Node.js](https://nodejs.org/) 20+
- [pnpm](https://pnpm.io/) 8+
- [Rust](https://rustup.rs/) 1.70+
- [Git](https://git-scm.com/)

**Platform-specific:**
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

**Recommended:**
- [VS Code](https://code.visualstudio.com/) with extensions:
  - `rust-analyzer` (Rust)
  - `svelte.svelte-vscode` (Svelte)
  - `dbaeumer.vscode-eslint` (ESLint)
  - `esbenp.prettier-vscode` (Prettier)

### Initial Setup

1. **Fork the repository** on GitHub

2. **Clone your fork**:
   ```bash
   git clone https://github.com/tarunJeevan/scriptoria.git
   cd scriptoria
   ```

3. **Add upstream remote**:
   ```bash
   git remote add upstream https://github.com/tarunJeevan/scriptoria.git
   ```

4. **Install dependencies**:
   ```bash
   pnpm install
   pnpm check
   ```

5. **Verify setup**:
   ```bash
   # Frontend checks
   pnpm check

   # Backend checks
   cd src-tauri
   cargo check

   # Run app
   cd ..
   pnpm tauri dev
   ```

---

## Development Workflow

### Branching Strategy

- `main` - Production-ready code
- `develop` - Integration branch for features
- `feature/*` - New features (branch from `develop`)
- `fix/*` - Bug fixes (branch from `develop` or `main`)
- `docs/*` - Documentation updates

**Example:**
```bash
git checkout develop
git pull upstream develop
git checkout -b feature/add-timeline-view
```

### Keeping Your Fork Updated

```bash
# Fetch upstream changes
git fetch upstream

# Update your local develop branch
git checkout develop
git merge upstream/develop

# Update your feature branch
git checkout feature/your-feature
git rebase develop
```

### Making Changes

1. **Create a feature branch**:
   ```bash
   git checkout -b feature/my-feature
   ```

2. **Make your changes** following [Code Style](#code-style) guidelines

3. **Test your changes**:
   ```bash
   # Frontend
   pnpm check
   pnpm lint
   pnpm format

   # Backend
   cd src-tauri
   cargo fmt -- --check
   cargo clippy -- -D warnings
   cargo test
   ```

4. **Commit with conventional commits**:
   ```bash
   git commit -m "feat(editor): add markdown shortcuts"
   git commit -m "fix(db): resolve migration ordering issue"
   git commit -m "docs(readme): update installation instructions"
   ```

   **Commit types:**
   - `feat` - New feature
   - `fix` - Bug fix
   - `docs` - Documentation only
   - `style` - Formatting, missing semicolons, etc.
   - `refactor` - Code restructuring
   - `test` - Adding tests
   - `chore` - Maintenance (deps, config)

5. **Push to your fork**:
   ```bash
   git push origin feature/my-feature
   ```

---

## Code Style

### TypeScript/Svelte

**Tools:**
- Formatter: Prettier (`.prettierrc`)
- Linter: ESLint (`eslint.config.js`)
- Type checker: `svelte-check`

**Rules:**
- Use tabs for indentation
- Single quotes for strings
- No trailing commas
- 100 character line width

**Example:**
```typescript
// ✅ Good
export function createDocument(title: string, content: string): Document {
	return {
		id: generateId(),
		title,
		content,
		createdAt: new Date()
	};
}

// ❌ Bad
export function createDocument(title:string,content:string):Document{
  return {id:generateId(),title,content,createdAt:new Date()}}
```

**Run checks:**
```bash
pnpm format        # Auto-fix formatting
pnpm lint          # Check for errors
pnpm check         # Type checking
```

### Rust

**Tools:**
- Formatter: `rustfmt` (`rustfmt.toml`)
- Linter: Clippy (`clippy.toml`)

**Rules:**
- 4-space indentation
- 100 character line width
- Organize imports (Std → External → Crate)
- Use idiomatic error handling (`Result`, `Option`)

**Example:**
```rust
// ✅ Good
pub async fn create_document(
    pool: &SqlitePool,
    params: CreateDocumentParams,
) -> Result<Document, Error> {
    let doc = sqlx::query_as!(
        Document,
        "INSERT INTO documents (title, content) VALUES (?, ?)",
        params.title,
        params.content
    )
    .fetch_one(pool)
    .await?;

    Ok(doc)
}

// ❌ Bad
pub async fn create_document(pool: &SqlitePool, params: CreateDocumentParams) -> Result<Document, Error> {
let doc=sqlx::query_as!(Document,"INSERT INTO documents (title, content) VALUES (?, ?)",params.title,params.content).fetch_one(pool).await?;
Ok(doc)}
```

**Run checks:**
```bash
cd src-tauri
cargo fmt         # Auto-fix formatting
cargo clippy      # Check for errors
cargo test        # Run tests
```

---

## Testing

### Test Requirements

**All pull requests must include tests** for new functionality:

- **Frontend**: Component tests, integration tests
- **Backend**: Unit tests, integration tests

### Running Tests

**Frontend:**
```bash
# Type checking (required)
pnpm check

# Unit tests (when implemented)
pnpm test

# E2E tests (when implemented)
pnpm test:e2e
```

**Backend:**
```bash
cd src-tauri

# All tests
cargo test

# Specific test
cargo test test_create_document

# With output
cargo test -- --nocapture
```

### Writing Tests

**Rust example:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_document() {
        let pool = setup_test_db().await;
        let params = CreateDocumentParams {
            title: "Test".to_string(),
            content: "Content".to_string(),
        };

        let doc = create_document(&pool, params).await.unwrap();

        assert_eq!(doc.title, "Test");
        assert_eq!(doc.content, "Content");
    }
}
```

**TypeScript example:**
```typescript
import { describe, it, expect } from 'vitest';
import { createDocument } from '$lib/utils/documents';

describe('createDocument', () => {
	it('should create a document with valid inputs', () => {
		const doc = createDocument('Title', 'Content');

		expect(doc.title).toBe('Title');
		expect(doc.content).toBe('Content');
		expect(doc.id).toBeDefined();
	});
});
```

---

## Pull Request Process

### Before Submitting

1. **Update your branch**:
   ```bash
   git fetch upstream
   git rebase upstream/develop
   ```

2. **Run all checks**:
   ```bash
   # Frontend
   pnpm format && pnpm lint && pnpm check

   # Backend
   cd src-tauri
   cargo fmt -- --check && cargo clippy -- -D warnings && cargo test
   ```

3. **Update documentation** if needed

4. **Add/update tests** for new features

### Submitting

1. **Push to your fork**:
   ```bash
   git push origin feature/my-feature
   ```

2. **Open a Pull Request** on GitHub

3. **Fill out the PR template**:
   - Description of changes
   - Related issue number (if applicable)
   - Testing performed
   - Screenshots (for UI changes)

4. **Wait for review** - maintainers will review within 1-3 days

### PR Review Process

**Reviewers will check:**
- Code quality and style
- Test coverage
- Documentation updates
- Breaking changes
- Security implications

**Possible outcomes:**
- ✅ **Approved** - Will be merged
- 🔄 **Changes requested** - Address feedback and push updates
- ❌ **Closed** - Duplicate, out of scope, or not aligned with project goals

### After Approval

1. Maintainer will merge your PR
2. Delete your feature branch:
   ```bash
   git branch -d feature/my-feature
   git push origin --delete feature/my-feature
   ```

---

## Project Structure

### Frontend (`src/`)

```
src/
├── lib/                    # Shared code
│   ├── components/        # Reusable UI components
│   ├── stores/            # Svelte stores (state management)
│   ├── editor/            # Tiptap extensions
│   └── utils/             # Utility functions
├── routes/                # SvelteKit pages
└── app.html               # HTML template
```

### Backend (`src-tauri/src/`)

```
src-tauri/src/
├── main.rs                # Binary entry point
├── lib.rs                 # Library entry point
├── commands/              # Tauri IPC commands
├── models/                # Database models
├── db/                    # Database operations
├── encryption/            # Encryption service
└── ai/                    # AI inference
```

---

## Architecture Decisions

### Key Principles

1. **Local-first** - All features work offline by default
2. **Privacy-respecting** - No telemetry, no cloud by default
3. **Type-safe** - Strict TypeScript, idiomatic Rust
4. **Testable** - All modules include tests
5. **Documented** - All public APIs documented

### Technology Choices

**Why Tauri?**
- Smaller bundle size than Electron (~3MB vs ~120MB)
- Native performance (Rust backend)
- Better security model
- Cross-platform support

**Why SvelteKit?**
- Reactive paradigm fits UI well
- Small runtime (~20KB)
- Excellent TypeScript support
- SSR/SSG capabilities (future)

**Why SQLite?**
- Embedded database (no server)
- Reliable and battle-tested
- Excellent performance for local apps
- Easy backups (single file)

**Why Ollama?**
- Local inference (privacy)
- Easy model management
- Good performance on consumer hardware
- Open-source

### Design Patterns

**Backend:**
- Repository pattern for database access
- Command pattern for Tauri IPC
- Service layer for business logic

**Frontend:**
- Component composition (atomic design)
- Store-based state management
- Utility-first CSS (Tailwind planned)

---

## Common Tasks

### Adding a New Tauri Command

1. **Define in `src-tauri/src/commands/mod.rs`**:
   ```rust
   #[tauri::command]
   pub async fn my_command(param: String) -> Result<String, Error> {
       Ok(format!("Received: {}", param))
   }
   ```

2. **Register in `src-tauri/src/lib.rs`**:
   ```rust
   .invoke_handler(tauri::generate_handler![
       commands::my_command,
   ])
   ```

3. **Call from frontend**:
   ```typescript
   import { invoke } from '@tauri-apps/api/core';

   const result = await invoke<string>('my_command', { param: 'test' });
   ```

### Adding a Database Migration

1. **Create migration file** in `src-tauri/migrations/`:
   ```sql
   -- 20250115000000_add_feature.sql
   ALTER TABLE documents ADD COLUMN new_field TEXT;
   ```

2. **Run migration**:
   ```bash
   cd src-tauri
   sqlx migrate run
   ```

3. **Update models** in `src-tauri/src/models/`

---

## Getting Help

- 💬 **Discussions**: [GitHub Discussions](https://github.com/YOUR_USERNAME/scriptoria/discussions)
- 🐛 **Bug Reports**: [GitHub Issues](https://github.com/YOUR_USERNAME/scriptoria/issues)
- 📖 **Documentation**: [docs/](../docs/)

---

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

---

**Thank you for contributing to Scriptoria!** 🎉
