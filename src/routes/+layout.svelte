<script lang="ts">
  import "../app.css";
  import { goto, invalidateAll } from "$app/navigation";
  import { page } from "$app/stores";
  import { onMount } from "svelte";
  import { auth } from "$lib/stores/auth";
  import { preferences } from "$lib/stores/preferences";
  import { tenancy } from "$lib/stores/tenancy";
  import { navLabel, t } from "$lib/i18n";
  import type { User } from "$lib/types";
  import type { TenancyContext, UiPreferences } from "$lib/types";

  let user = $state<User | null>(null);
  let prefs = $state<UiPreferences | null>(null);
  let tenancyCtx = $state<TenancyContext | null>(null);
  let ready = $state(false);

  const allLinks = [
    { href: "/", labelKey: "dashboard" as const, roles: ["admin", "teacher", "student", "parent"] },
    { href: "/my-courses", labelKey: "myCourses" as const, roles: ["student"] },
    { href: "/courses", labelKey: "courses" as const, roles: ["admin", "teacher"] },
    { href: "/gradebook", labelKey: "gradebook" as const, roles: ["admin", "teacher"] },
    { href: "/quizzes", labelKey: "quizzes" as const, roles: ["admin", "teacher"] },
    { href: "/submissions", labelKey: "submissions" as const, roles: ["admin", "teacher"] },
    { href: "/schedule", labelKey: "schedule" as const, roles: ["admin", "teacher"] },
    { href: "/announcements", labelKey: "announcements" as const, roles: ["admin", "teacher"] },
    { href: "/groups", labelKey: "groups" as const, roles: ["admin", "teacher"] },
    { href: "/forums", labelKey: "forums" as const, roles: ["admin", "teacher", "student"] },
    { href: "/hub", labelKey: "hub" as const, roles: ["admin", "teacher"] },
    { href: "/sessions", labelKey: "sessions" as const, roles: ["admin", "teacher"] },
    { href: "/certificates", labelKey: "certificates" as const, roles: ["admin", "teacher"] },
    { href: "/parent", labelKey: "parent" as const, roles: ["admin", "parent"] },
    { href: "/users", labelKey: "users" as const, roles: ["admin"] },
    { href: "/fees", labelKey: "fees" as const, roles: ["admin"] },
    { href: "/settings", labelKey: "settings" as const, roles: ["admin", "teacher"] },
  ];

  let links = $derived(
    user ? allLinks.filter((link) => link.roles.includes(user!.role)) : [],
  );

  onMount(() => {
    let unsubUser = auth.subscribe((u) => {
      user = u;
    });
    let unsubPrefs = preferences.subscribe((p) => {
      prefs = p;
    });
    let unsubTenancy = tenancy.subscribe((ctx) => {
      tenancyCtx = ctx;
    });
    Promise.all([auth.init(), preferences.init()]).then(() => {
      ready = true;
    });
    return () => {
      unsubUser();
      unsubPrefs();
      unsubTenancy();
    };
  });

  $effect(() => {
    if (!ready) return;
    const path = $page.url.pathname;
    if (!user && path !== "/login") {
      goto("/login");
    }
    if (user && path === "/login") {
      goto(user.role === "student" ? "/my-courses" : "/");
    }
  });

  async function signOut() {
    await auth.logout();
    goto("/login");
  }

  async function switchSchool(event: Event) {
    const select = event.target as HTMLSelectElement;
    await tenancy.setActiveSchool(select.value);
    await invalidateAll();
  }
</script>

{#if !ready}
  <div class="boot">{t("loading")}</div>
{:else if user}
  <div class="app-shell">
    <aside class="sidebar">
      <div class="brand">
        <div class="brand-mark">CM</div>
        <div>
          <h1>{prefs?.school_name || t("appName")}</h1>
          <p>{user.name} · {user.role}</p>
        </div>
      </div>
      {#if tenancyCtx && tenancyCtx.schools.length > 0}
        <label class="school-switch">
          <span>{t("tenancy.activeSchool")}</span>
          <select
            value={tenancyCtx.active_school_id}
            onchange={switchSchool}
            aria-label={t("tenancy.activeSchool")}
          >
            {#each tenancyCtx.schools as school}
              <option value={school.id}>{school.name}</option>
            {/each}
          </select>
        </label>
      {/if}
      <nav class="nav">
        {#each links as link}
          <a href={link.href} class:active={$page.url.pathname === link.href}>
            {navLabel(link.labelKey)}
          </a>
        {/each}
      </nav>
      <button class="btn btn-secondary sign-out" onclick={signOut}>{t("signOut")}</button>
    </aside>
    <main class="content">
      <slot />
    </main>
  </div>
{:else}
  <slot />
{/if}

<style>
  .boot {
    min-height: 100vh;
    display: grid;
    place-items: center;
    color: var(--muted);
  }

  .sign-out {
    margin-top: auto;
    width: 100%;
  }

  .school-switch {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    margin: 0.75rem 0 1rem;
    font-size: 0.8rem;
    color: var(--muted);
  }

  .school-switch select {
    width: 100%;
    padding: 0.4rem 0.5rem;
    border-radius: 6px;
    border: 1px solid var(--border);
    background: var(--surface);
    color: inherit;
  }
</style>
