<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  let appVersion = "0.1.0";
  let tauriVersion = "";

  onMount(async () => {
    try {
      // Get Tauri version (example of IPC call)
      tauriVersion = await invoke<string>("tauri", {
        __tauriModule: "App",
        message: { cmd: "version" },
      }).catch(() => "Tauri 2.x");
    } catch (error) {
      console.error("Failed to get Tauri version:", error);
      tauriVersion = "Unknown";
    }
  });
</script>

<main>
  <div class="container">
    <h1>Scriptoria</h1>
    <p class="subtitle">AI-Enhanced Creative Writing Studio</p>

    <div class="info-card">
      <h2>Phase 1: Chunk 1 Complete! 🎉</h2>
      <p>Project infrastructure is set up and running.</p>

      <div class="version-info">
        <div class="version-item">
          <span class="label">App Version:</span>
          <span class="value">{appVersion}</span>
        </div>
        <div class="version-item">
          <span class="label">Tauri Version:</span>
          <span class="value">{tauriVersion}</span>
        </div>
      </div>
    </div>

    <div class="next-steps">
      <h3>Next Steps:</h3>
      <ul>
        <li>
          <strong>Chunk 0:</strong> Database schema & encryption foundation
        </li>
        <li><strong>Chunk 2:</strong> Rich text editor (Tiptap)</li>
        <li><strong>Chunk 3:</strong> Document management backend</li>
      </ul>
    </div>

    <footer>
      <p>Local-first • Privacy-respecting • AI-powered</p>
    </footer>
  </div>
</main>

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    font-family:
      -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Oxygen, Ubuntu,
      Cantarell, sans-serif;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    color: #333;
  }

  main {
    display: flex;
    justify-content: center;
    align-items: center;
    min-height: 100vh;
    padding: 2rem;
  }

  .container {
    background: white;
    border-radius: 16px;
    padding: 3rem;
    max-width: 600px;
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
  }

  h1 {
    font-size: 3rem;
    margin: 0 0 0.5rem 0;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
  }

  .subtitle {
    font-size: 1.2rem;
    color: #666;
    margin: 0 0 2rem 0;
  }

  .info-card {
    background: #f8f9fa;
    border-radius: 12px;
    padding: 1.5rem;
    margin-bottom: 2rem;
  }

  .info-card h2 {
    margin: 0 0 1rem 0;
    font-size: 1.5rem;
    color: #667eea;
  }

  .info-card p {
    margin: 0 0 1rem 0;
    color: #666;
  }

  .version-info {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .version-item {
    display: flex;
    justify-content: space-between;
    padding: 0.5rem;
    background: white;
    border-radius: 6px;
  }

  .label {
    font-weight: 600;
    color: #555;
  }

  .value {
    font-family: "Courier New", monospace;
    color: #667eea;
  }

  .next-steps {
    background: #fff9e6;
    border-left: 4px solid #ffc107;
    border-radius: 8px;
    padding: 1.5rem;
    margin-bottom: 2rem;
  }

  .next-steps h3 {
    margin: 0 0 1rem 0;
    color: #ff9800;
  }

  .next-steps ul {
    margin: 0;
    padding-left: 1.5rem;
  }

  .next-steps li {
    margin-bottom: 0.5rem;
    color: #666;
  }

  .next-steps strong {
    color: #ff9800;
  }

  footer {
    text-align: center;
    padding-top: 2rem;
    border-top: 1px solid #e0e0e0;
  }

  footer p {
    margin: 0;
    color: #999;
    font-size: 0.9rem;
  }
</style>
