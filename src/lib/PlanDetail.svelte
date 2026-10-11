<script lang="ts">
  import { untrack } from "svelte";
  import ConfirmButton from "./ConfirmButton.svelte";
  import {
    addPlan,
    blankPlan,
    clock,
    plans,
    removePlan,
    roles,
    toggleDone,
    ui,
    view,
    type Level,
    type Plan,
    type Repeat,
  } from "./store.svelte";

  let { id, onback }: { id?: string; onback: () => void } = $props();

  const existing = $derived(id ? plans.find((p) => p.id === id) : undefined);

  /// One object backs both jobs. For a plan that already exists this is the store's own
  /// object, so every keystroke is live; for a new one it is a local draft that nothing
  /// writes until 创建计划. A command can hand over a partial draft exactly once.
  let draft = $state<Omit<Plan, "id"> | null>(null);

  $effect(() => {
    if (id) {
      draft = existing ?? null;
      return;
    }
    const day = untrack(() => view.day);
    const seeded = untrack(() => ui.planDraft);
    draft = { ...blankPlan(day), ...(seeded ?? {}) };
    if (seeded) ui.planDraft = null;
  });

  const levels: { id: Level; label: string; hint: string; ready: boolean }[] = [
    { id: "L1", label: "静默", hint: "只在贴边胶囊和日历里出现", ready: true },
    { id: "L2", label: "系统通知", hint: "到点弹一条 Windows 通知", ready: true },
    { id: "L3", label: "强提醒", hint: "到点弹一张卡片，停在你回应为止", ready: true },
  ];

  const repeats: { id: Repeat; label: string; ready: boolean }[] = [
    { id: "once", label: "一次", ready: true },
    { id: "daily", label: "每天", ready: true },
  ];

  /// Hours are decimals in the store (9.5) and "09:30" in a time input.
  function toHours(text: string) {
    const [h, m] = text.split(":").map(Number);
    return (h || 0) + (m || 0) / 60;
  }

  /// Moving the start carries the end along, so the length just picked survives the edit.
  function setStart(text: string) {
    if (!draft) return;
    const span = draft.end - draft.start;
    draft.start = toHours(text);
    draft.end = draft.start + span;
  }

  const minutes = $derived(draft ? Math.round((draft.end - draft.start) * 60) : 30);

  function setMinutes(text: string) {
    if (!draft) return;
    const value = Number(text);
    // An emptied box must not snap the length to something arbitrary while it is being typed.
    if (!Number.isFinite(value) || value <= 0) return;
    draft.end = draft.start + value / 60;
  }

  function create() {
    if (!draft) return;
    addPlan({ ...draft });
    onback();
  }
</script>

{#if draft}
  <article class="detail">
    <label class="field">
      <span class="lb">标题</span>
      <input bind:value={draft.title} spellcheck="false" placeholder="这条计划叫什么" />
    </label>

    <label class="field">
      <span class="lb">说明</span>
      <input bind:value={draft.note} spellcheck="false" placeholder="这条计划要做什么" />
    </label>

    <div class="row">
      <span class="lb">日期</span>
      <input class="tin" type="date" bind:value={draft.day} />
    </div>

    <div class="row">
      <span class="lb">开始</span>
      <input
        class="tin"
        type="time"
        value={clock(draft.start)}
        oninput={(e) => setStart(e.currentTarget.value)}
      />
      <span class="unit">持续</span>
      <input
        class="tin num"
        type="number"
        min="5"
        step="5"
        value={minutes}
        oninput={(e) => setMinutes(e.currentTarget.value)}
      />
      <span class="unit">分钟</span>
    </div>

    <div class="row">
      <span class="lb">触发</span>
      <div class="chips">
        {#each repeats as repeat (repeat.id)}
          <button
            class="chip"
            class:on={draft.repeat === repeat.id}
            disabled={!repeat.ready}
            title={repeat.ready ? undefined : "重复要等调度器落地（M1）"}
            onclick={() => draft && (draft.repeat = repeat.id)}
          >
            {repeat.label}
          </button>
        {/each}
      </div>
    </div>

    <div class="row">
      <span class="lb">通知</span>
      <div class="chips">
        {#each levels as level (level.id)}
          <button
            class="chip"
            class:on={draft.level === level.id}
            disabled={!level.ready}
            title={level.hint}
            onclick={() => draft && (draft.level = level.id)}
          >
            {level.label}
          </button>
        {/each}
      </div>
    </div>

    <div class="row">
      <span class="lb">角色</span>
      <select class="tin" bind:value={draft.role}>
        {#each roles as role (role.id)}
          <option value={role.name}>{role.name}</option>
        {/each}
      </select>
    </div>

    <div class="actions">
      {#if existing}
        <button class="btn" onclick={() => toggleDone(existing.id)}>
          {existing.done ? "标记为待执行" : "标记为已完成"}
        </button>
        <ConfirmButton
          label="删除计划"
          onconfirm={() => {
            removePlan(existing.id);
            onback();
          }}
        />
      {:else}
        <button class="btn" onclick={onback}>取消</button>
        <button class="btn primary" onclick={create}>创建计划</button>
      {/if}
    </div>
  </article>
{:else}
  <div class="missing">
    <p>这条计划已经不在了。</p>
    <button class="btn" onclick={onback}>返回</button>
  </div>
{/if}

<style>
  .detail {
    display: flex;
    flex-direction: column;
    height: 100%;
    padding: 16px 20px 14px;
    overflow-y: auto;
  }

  .field {
    display: block;
    margin-bottom: 14px;
  }

  .lb {
    display: block;
    margin-bottom: 6px;
    font-size: 10.5px;
    font-weight: 600;
    color: var(--text-3);
  }

  .field input {
    width: 100%;
    height: 32px;
    padding: 0 10px;
    border: 1px solid var(--line-strong);
    border-radius: var(--r-btn);
    background: var(--bg);
    color: var(--text);
    font: inherit;
    font-size: 12.5px;
    outline: none;
  }

  .field input:focus {
    border-color: var(--brand);
    box-shadow: 0 0 0 3px var(--brand-dim);
  }

  .row {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-bottom: 14px;
  }

  .row .lb {
    width: 34px;
    margin: 0;
    flex: none;
  }

  /* The native pickers are the whole reason these fields are not free text. */
  .tin {
    height: 30px;
    padding: 0 8px;
    border: 1px solid var(--line-strong);
    border-radius: var(--r-btn);
    background: var(--bg);
    color: var(--text);
    font: inherit;
    font-size: 12px;
    outline: none;
  }

  .tin:focus {
    border-color: var(--brand);
    box-shadow: 0 0 0 3px var(--brand-dim);
  }

  .tin.num {
    width: 74px;
  }

  .unit {
    font-size: 11.5px;
    color: var(--text-3);
  }

  .chips {
    display: flex;
    gap: 6px;
  }

  .chip {
    padding: 5px 12px;
    border: 1px solid var(--line-strong);
    border-radius: 14px;
    background: var(--bg);
    color: var(--text-2);
    font: inherit;
    font-size: 11.5px;
    cursor: pointer;
  }

  .chip:hover:not(:disabled) {
    background: var(--hover);
  }

  .chip:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }

  .chip.on {
    border-color: rgba(43, 108, 246, 0.35);
    background: var(--brand-dim);
    color: var(--brand);
    font-weight: 600;
  }

  .actions {
    display: flex;
    gap: 8px;
    margin-top: auto;
    padding-top: 14px;
  }

  .btn {
    height: 32px;
    padding: 0 16px;
    border: 1px solid var(--line-strong);
    border-radius: var(--r-btn);
    background: var(--bg);
    color: var(--text);
    font: inherit;
    font-size: 12.5px;
    cursor: pointer;
  }

  .btn:hover {
    background: var(--hover);
  }

  .btn.primary {
    border-color: transparent;
    background: var(--brand);
    color: #fff;
    font-weight: 600;
  }

  .btn.primary:hover {
    background: var(--brand);
    filter: brightness(1.06);
  }

  .missing {
    display: grid;
    place-content: center;
    gap: 12px;
    height: 100%;
    color: var(--text-3);
    font-size: 12.5px;
    text-align: center;
  }
</style>
