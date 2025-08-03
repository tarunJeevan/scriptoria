# Scriptoria Project Roadmap

## Tech Stack

- **Framework**: Tauri
- **Frontend**: React
  + ***Editor***: Tiptap
    * [Tiptap React Starter Guide](https://tiptap.dev/docs/editor/getting-started/install/react)
    * [Tiptap Core Concepts Guide](https://tiptap.dev/docs/editor/core-concepts/introduction)
    * [Tiptap Open Source Extensions](https://tiptap.dev/docs/editor/extensions/overview?filter=opensource)
    * [Tiptap Custom Extensions Guide](https://tiptap.dev/docs/editor/extensions/custom-extensions)
    * [Tiptap API Guide](https://tiptap.dev/docs/editor/api/editor)
    * [Tiptap RAG Example](https://tiptap.dev/docs/examples/advanced/retrieval-augmented-generation-rag)
  + ***Styling***: TailwindCSS & Shadcn/ui
    * Tiptap is a headless library requiring custom styles
- **Database**: SurrealDB
- **RAG**: 
  - ***LLM*** - Ollama models such as: 
    - Mistral (7B) | Mistral NeMo (12B) | Llama 3.1 (8B) | Llama 3.2 (1B/3B) | Gemma 3 (1B/4B/12B)
  - ***Embedding Model*** - Ollama models such as:
    - Nomic Embed Text v2
    - ***Note***: Research other **open-source** **text embedding** models
  - ***Retriever*** - Chroma vector database + FAISS
    - ***Note***: Consider retrieval methods such as Dense and Hybrid Retrieval 
  - ***Framework*** - LangChain and LlamaIndex
    - ***Note***: LangChain has a JavaScript API. Look into LlamaIndex as well

#### Framework - Tauri

Backend logic written using Rust that will manage all user projects and documents, as well as RAG logic and connection to services such as Google Drive, OneDrive, NextCloud, etc.

#### Frontend - React

Frontend UI letting users manage projects, documents, AI assistant, and various settings. Also provides an editor to write and edit documents. 

##### Editor - Tiptap

An open-source, headless *What You See Is What You Get* (WYSIWYG) editor that is highly extendable with open-source as well as custom extensions. Its headless nature means it doesn't come with any build in styles, making it highly customizable.

#### Styling - TailwindCSS & Shadcn/ui

...

#### Database - SurrealDB

...

#### LLM -

...

## Roadmap

A sequential roadmap for building Scriptoria with each milestone building on the last. 

#### Phase 1: Project Initialization and Basic UI

**Goal**: Set up project structure and build the application shell.
**Tasks**: 

- [x] Initialize Tauri project
- [ ] Set up **React + TypeScript + TailwindCSS** frontend with Vite
  - ***Note***: React + TypeScript included in template. Add TailwindCSS
- [ ] Design main layout (editor, sidebar, navbar, etc.)
- [ ] Integrate a **rich-text or markdown editor** (e.g., Tiptap, Lexical, ProseMirror, etc.)
  - ***Note***: Chosen editor is Tiptap. Refer to [Tiptap React Starter Guide](https://tiptap.dev/docs/editor/getting-started/install/react) for more information.
- [ ] Add basic navigation for documents and notes
- [ ] Set up IPC communication between React frontend and Rust backend

***Deliverable***: A functioning cross-platform desktop app with a working text editor and UI.

#### Phase 2: File Management System

**Goal**: Implement local file operations (e.g., read/write, open, save, rename, etc.).
**Tasks**: 

- [ ] Enable opening, saving, and renaming local files
- [ ] Persist document metadata (e.g., tags, chapter/scene/outline)
- [ ] Implement auto-save functionality
- [ ] Organize local files into a visible document tree (sidebar)

***Deliverable***: Users can open, edit, and manage stories and notes as local files.

#### Phase 3: Local LLM Integration

**Goal**: Integrate a local LLM to generate or edit text.
**Tasks**: 

- [ ] Integrate `llama-cpp-rs` or `candle` to run a local quantized LLM
- [ ] Add IPC command to send a prompt from frontend to backend and receive a response
- [ ] Build a basic AI prompt UI: "Continue writing", "Rewrite", "Summarize", "Brainstorm", etc.
- [ ] Display results inline or in a sidebar panel

***Deliverable***: An offline LLM that assists with basic writing tasks.

#### Phase 4: RAG Setup (Document Indexing and Embedding)

**Goal**: Use RAG to improve AI responses using user's notes and writing.
**Tasks**: 

- [ ] Chunk and embed documents (e.g., per paragraph or section)
- [ ] Store embeddings and metadata in a local vector store or DB
- [ ] Implement cosine similarity or use a vector database (e.g., Chroma, FAISS, etc.)
- [ ] Use top-k similar chunks to construct contextual prompts for the LLM

***Deliverable***: AI assistant can reference user writing to generate context-aware suggestions.

#### Phase 5: Privacy and User Control Features

**Goal**: Ensure user data is protected and clearly controlled.
**Tasks**: 

- [ ] Create a local-only mode toggle that prevents cloud communication (should be default)
- [ ] Add a "What data is used?" log or dashboard
  - ***Note***: Research AI disclaimers and common data protection information
- [ ] Allows users to remove indexed data or reset AI memory
- [ ] Bundle all components to run entirely offline

***Deliverable***: Users have full control of their data and model behavior.

#### Phase 6: Cloud Drive Integration

**Goal**: Allow users to sync writing and notes with cloud storage (Google Drive, OneDrive, Dropbox, Nextcloud, etc.).
**Tasks**:

- [ ] Set up OAuth2 authentication for chosen providers (start with Google Drive)
- [ ] Let users browse, open, and save documents from the could locally 
- [ ] Preserve file permissions and sync local version with cloud version only on user action

***Deliverable***: Optional sync with external storage that respects privacy defaults.

#### Phase 7: Writing Style Adaption

**Goal**: Make the AI reflect the user's writing style more closely.
**Tasks**:

- [ ] Analyze sentence structure, word choice, and tone from user's files
- [ ] Automatically adjust AI prompt templates accordingly
- [ ] Optionally allow user-initiated fine-tuning on local content (if hardware allows)

***Deliverable***: Personalized AI writing aligned with the user's style.

#### Phase 8: UX Polish and Utility Features

**Goal**: Improve the overall user experience with helpful tools and refinement.
**Tasks**:

- [ ] Implement distraction-free writing mode, dark mode, and editor customization
- [ ] Add word/character count, reading time estimation, and outline tools
- [ ] Implement split screen to reference notes while writing

***Deliverable***: A polished, writer-friendly UX that invites daily use.

#### Phase 9: App Packaging and Deployment

**Goal**: Prepare for cross-platform release.
**Tasks**:

- [ ] Build native installers for Windows, MacOS, and Linux via Tauri
- [ ] Set up update mechanism (e.g., GitHub Releases or self-hosted)
  - ***Note***: Refer to [Distribute | Tauri](https://tauri.app/distribute/) and [Updater | Tauri](https://v2.tauri.app/plugin/updater/) for more info
- [ ] Write documentation for local installation, usage, and data control
- [ ] Perform security audit (especially around file handling and LLM inference)

***Deliverable***: A cross-platform, downloadable app ready for release.
