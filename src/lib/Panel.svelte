<script lang="ts">
  import Composer from "./Composer.svelte";
  import Mark from "./Mark.svelte";
  import PlanDetail from "./PlanDetail.svelte";
  import Planner from "./Planner.svelte";
  import ProjectPage from "./ProjectPage.svelte";
  import Projects from "./Projects.svelte";
  import Roles from "./Roles.svelte";
  import Settings from "./Settings.svelte";
  import { llmConnected } from "./note";
  import { clock, keyOf, plans, todayKey, type Domain, type Route } from "./store.svelte";

  let {
    route,
    tab,
    canBack,
    active,
    onopen,
    onswitch,
    back,
  }: {
    route: Route;
    tab: Domain;
    canBack: boolean;
    /// False while the window is showing the capsule: the panel stays mounted so a
    /// half-written line survives being put away, but it must not steal focus.
    active: boolean;
    onopen: (route: Route) => void;
    onswitch: (tab: Domain) => void;
    back: () => void;
  } = $props();

  // The status line reports what is actually scheduled, and nothing here fires by itself
  // yet. The minute-ish tick is so "next up" rolls over on its own instead of going stale.
  let tick = $state(Date.now());
  $effect(() => {
    const id = setInterval(() => (tick = Date.now()), 30000);
    return () => clearInterval(id);
  });

  const pending = $derived(plans.filter((p) => !p.done));

  const next = $derived.by(() => {
    const now = new Date(tick);
    const today = keyOf(now);
    const minutes = now.getHours() * 60 + now.getMinutes();
    return [...pending]
      .filter((p) => p.day > today || (p.day === today && p.start * 60 > minutes))
      .sort((a, b) => a.day.localeCompare(b.day) || a.start - b.start)[0];
  });

  const nextLabel = $derived.by(() => {
    if (!next) return pending.length > 0 ? "今天没有别的安排了" : "还没有安排";
    const day = next.day === todayKey ? "今天" : next.day.slice(5).replace("-", "/");
    return `下一项 · ${day} ${clock(next.start)} ${next.title}`;
  });

  const tabs: { id: Domain; label: string; root: Route }[] = [
    { id: "plan", label: "计划", root: { name: "planner" } },
    { id: "role", label: "角色", root: { name: "roles" } },
    { id: "project", label: "项目", root: { name: "projects" } },
    { id: "settings", label: "设置", root: { name: "settings" } },
  ];
</script>

<div class="panel">
  <header class="title" data-tauri-drag-region>
    <Mark size={18} />
    <span class="brand">智群</span>
    {#if !llmConnected()}
      <button
        class="llm-state"
        title="大模型还没连上，便签只能用本地正则解析"
        onclick={() => onopen({ name: "settings" })}
      >
        <span class="sdot" aria-hidden="true"></span>未连接
      </button>
    {/if}
  </header>

  <Composer domain={tab} {active} {onopen} {onswitch} />

  <header class="bar" data-tauri-drag-region>
    <div class="nav">
      <button class="icon" disabled={!canBack} onclick={back} aria-label="后退">
        <svg viewBox="0 0 16 16" width="14" height="14" fill="none" stroke="currentColor" stroke-width="1.7">
          <path d="M10 3.5L5.5 8l4.5 4.5" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
      </button>
    </div>

    <span class="divider"></span>

    <nav class="tabs">
      {#each tabs as item (item.id)}
        <button
          class="tab"
          class:active={tab === item.id}
          onclick={() => {
            onswitch(item.id);
            onopen(item.root);
          }}
        >
          {item.label}
        </button>
      {/each}
    </nav>
  </header>

  <div class="stage">
    {#if route.name === "planner"}
      <Planner onopen={(id) => onopen({ name: "plan", id })} oncreate={() => onopen({ name: "plan-new" })} />
    {:else if route.name === "plan"}
      <PlanDetail id={route.id} onback={back} />
    {:else if route.name === "plan-new"}
      <PlanDetail onback={back} />
    {:else if route.name === "roles"}
      <Roles />
    {:else if route.name === "projects"}
      <Projects onopen={(id) => onopen({ name: "project", id })} oncreate={() => onopen({ name: "project-new" })} />
    {:else if route.name === "project"}
      <ProjectPage id={route.id} ondone={back} />
    {:else if route.name === "project-new"}
      <ProjectPage ondone={back} />
    {:else}
      <Settings />
    {/if}
  </div>

  <footer class="status">
    <span class="dot" aria-hidden="true"></span>
    <span class="clip">{nextLabel}</span>
    <span class="divider"></span>
    <span class="num">待执行 {pending.length} 项</span>
  </footer>
</div>

<style>
  .panel {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    background: var(--bg);
    /* Without this the inner paint squares off the window's rounded corners. */
    border-radius: inherit;
  }

  .title {
    display: flex;
    align-items: center;
    gap: 7px;
    padding: 9px 14px 0;
  }

  /* The panel says out loud why a note came back parsed by the regex, and offers the fix --
     a top-right badge rather than a dialog nobody asked for. */
  .llm-state {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    margin-left: auto;
    height: 20px;
    padding: 0 8px;
    border: 0;
    border-radius: var(--r-tag);
    background: var(--orange-dim);
    color: var(--orange);
    font: inherit;
    font-size: 10px;
    cursor: pointer;
  }

  .llm-state .sdot {
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background: currentColor;
  }

  .brand {
    font-size: 10.5px;
    font-weight: 600;
    letter-spacing: 0.1em;
    color: var(--text-3);
  }

  .bar {
    display: flex;
    align-items: center;
    gap: 8px;
    height: 38px;
    padding: 0 10px 0 8px;
    border-bottom: 1px solid var(--line);
  }

  .nav {
    display: flex;
    gap: 2px;
  }

  .icon {
    display: grid;
    place-items: center;
    width: 26px;
    height: 26px;
    padding: 0;
    border: 0;
    border-radius: var(--r-btn);
    background: none;
    color: var(--text-2);
    cursor: pointer;
  }

  .icon:hover:not(:disabled) {
    background: var(--hover);
    color: var(--text);
  }

  .icon:disabled {
    color: var(--text-4);
    opacity: 0.5;
    cursor: default;
  }

  .divider {
    width: 1px;
    height: 14px;
    background: var(--line-strong);
  }

  .tabs {
    display: flex;
    gap: 2px;
  }

  .tab {
    padding: 5px 11px;
    border: 0;
    border-radius: var(--r-btn);
    background: none;
    color: var(--text-2);
    font: inherit;
    font-size: 12.5px;
    cursor: pointer;
  }

  .tab:hover {
    background: var(--hover);
  }

  .tab.active {
    background: var(--brand-dim);
    color: var(--brand);
    font-weight: 600;
  }

  .stage {
    position: relative;
    flex: 1;
    min-height: 0;
  }

  .status {
    display: flex;
    align-items: center;
    gap: 8px;
    height: 28px;
    padding: 0 14px;
    border-top: 1px solid var(--line);
    background: var(--sunken);
    font-size: 10.5px;
    color: var(--text-3);
  }

  .dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--green);
    flex: none;
  }

  /* A long plan title must not push the count off the end of a 760px panel. */
  .clip {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
  }

  .status .num {
    flex: none;
  }
</style>
