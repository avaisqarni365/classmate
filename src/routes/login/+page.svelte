<script lang="ts">
  import { goto } from "$app/navigation";
  import { auth } from "$lib/stores/auth";

  let email = $state("admin@classmate.local");
  let password = $state("admin123");
  let error = $state("");
  let loading = $state(false);

  async function submit(event: Event) {
    event.preventDefault();
    loading = true;
    error = "";
    try {
      await auth.login(email, password);
      goto("/");
    } catch (e) {
      error = e instanceof Error ? e.message : "Login failed";
    } finally {
      loading = false;
    }
  }
</script>

<div class="login-page">
  <div class="login-card card">
    <div class="brand" style="margin-bottom: 1rem">
      <div class="brand-mark">CM</div>
      <div>
        <h1>ClassMate</h1>
        <p>Sign in to your local workspace</p>
      </div>
    </div>

    <form class="form-grid" onsubmit={submit}>
      <label>
        Email
        <input type="email" bind:value={email} required autocomplete="username" />
      </label>
      <label>
        Password
        <input
          type="password"
          bind:value={password}
          required
          autocomplete="current-password"
        />
      </label>
      {#if error}
        <p class="error">{error}</p>
      {/if}
      <button class="btn btn-primary" type="submit" disabled={loading}>
        {loading ? "Signing in..." : "Sign in"}
      </button>
    </form>

    <p class="hint">Demo: admin@classmate.local / admin123</p>

    <p style="margin-top:1rem;font-size:.9rem;text-align:center">
      <a href="/help">Help & setup guide</a>
      ·
      <a href="https://cm.codes-ai.uk/download" target="_blank" rel="noopener noreferrer">Download</a>
    </p>
  </div>
</div>

<style>
  .login-page {
    min-height: 100vh;
    display: grid;
    place-items: center;
    padding: 1rem;
    background: var(--bg);
  }

  .login-card {
    width: min(420px, 100%);
  }

  .hint {
    margin: 1rem 0 0;
    color: var(--muted);
    font-size: 0.85rem;
  }
</style>
