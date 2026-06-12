<script lang="ts">

  import { api } from "$lib/api";

  import type { MaterialAiLab } from "$lib/types";



  let {

    aiLab,

    compact = false,

    embed = false,

    labCompleted = false,

    materialId = "",

    studentMode = false,

    onLabComplete,

  }: {

    aiLab: MaterialAiLab;

    compact?: boolean;

    embed?: boolean;

    labCompleted?: boolean;

    materialId?: string;

    studentMode?: boolean;

    onLabComplete?: () => void;

  } = $props();



  let completing = $state(false);

  let localCompleted = $state(false);

  const completed = $derived(labCompleted || localCompleted);

  let showEmbed = $state(false);



  async function markComplete() {

    if (!materialId || completing || completed) return;

    completing = true;

    try {

      await api.markMaterialLabComplete(materialId);

      localCompleted = true;

      onLabComplete?.();

    } catch {

      /* parent can show error */

    } finally {

      completing = false;

    }

  }

</script>



<div class="ai-lab" class:compact>

  <div class="ai-lab-head">

    <span class="ai-lab-badge">ARTIZAI {aiLab.lab.name}</span>

    {#if completed}

      <span class="done-badge">Lab complete</span>

    {/if}

    {#if !compact}

      <span class="ai-lab-tools">{aiLab.lab.tools.join(" · ")}</span>

    {/if}

  </div>

  {#if !compact}

    <p class="ai-lab-desc">{aiLab.lab.description}</p>

  {/if}

  <div class="ai-lab-actions">

    <a class="btn-ai-lab" href={aiLab.url} target="_blank" rel="noopener">Open AI Lab</a>

    {#if embed && aiLab.embed_url}

      <button type="button" class="btn-embed" onclick={() => (showEmbed = !showEmbed)}>

        {showEmbed ? "Hide embedded lab" : "Embed lab here"}

      </button>

    {/if}

    {#if studentMode && materialId && !completed}

      <button type="button" class="btn-complete" disabled={completing} onclick={markComplete}>

        {completing ? "Saving…" : "Mark lab complete"}

      </button>

    {/if}

    {#if !compact}

      <div class="prompts">

        {#each aiLab.activities as activity}

          <span class="prompt">{activity}</span>

        {/each}

      </div>

    {/if}

  </div>

  {#if showEmbed && aiLab.embed_url}

    <iframe

      class="lab-embed"

      src={aiLab.embed_url}

      title="ARTIZAI lab"

      loading="lazy"

      sandbox="allow-scripts allow-same-origin allow-forms allow-popups"

    ></iframe>

  {/if}

</div>



<style>

  .ai-lab {

    margin-top: 0.65rem;

    padding: 0.75rem;

    border-radius: 8px;

    border: 1px solid color-mix(in srgb, #7c3aed 35%, var(--border));

    background: color-mix(in srgb, #7c3aed 8%, var(--card, #fff));

  }



  .ai-lab.compact {

    padding: 0.55rem 0.65rem;

  }



  .ai-lab-head {

    display: flex;

    flex-wrap: wrap;

    gap: 0.35rem 0.75rem;

    align-items: center;

    margin-bottom: 0.35rem;

  }



  .ai-lab-badge {

    display: inline-block;

    background: #5b21b6;

    color: #fff;

    font-size: 0.72rem;

    font-weight: 700;

    letter-spacing: 0.03em;

    padding: 0.15rem 0.45rem;

    border-radius: 4px;

  }



  .done-badge {

    background: #166534;

    color: #dcfce7;

    font-size: 0.72rem;

    font-weight: 600;

    padding: 0.15rem 0.45rem;

    border-radius: 4px;

  }



  .ai-lab-tools,

  .ai-lab-desc {

    color: var(--muted);

    font-size: 0.85rem;

    margin: 0;

  }



  .ai-lab-actions {

    display: flex;

    flex-wrap: wrap;

    gap: 0.35rem;

    align-items: center;

  }



  .btn-ai-lab,

  .btn-embed,

  .btn-complete {

    display: inline-block;

    border: none;

    cursor: pointer;

    padding: 0.35rem 0.75rem;

    border-radius: 6px;

    font-size: 0.85rem;

    font-weight: 600;

    text-decoration: none;

  }



  .btn-ai-lab {

    background: #7c3aed;

    color: #fff;

  }



  .btn-embed {

    background: #334155;

    color: #fff;

  }



  .btn-complete {

    background: #0f766e;

    color: #fff;

  }



  .lab-embed {

    width: 100%;

    height: min(420px, 70vh);

    margin-top: 0.65rem;

    border: 1px solid var(--border);

    border-radius: 8px;

    background: #fff;

  }



  .prompts {

    display: flex;

    flex-wrap: wrap;

    gap: 0.35rem;

    margin-top: 0.5rem;

    width: 100%;

  }



  .prompt {

    background: #1e293b;

    color: #cbd5e1;

    font-size: 0.78rem;

    padding: 0.2rem 0.45rem;

    border-radius: 999px;

  }

</style>


