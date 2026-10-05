<script lang="ts">
  import { plans, removePlan, toggleDone, type Level } from "./store.svelte";

  let { id, onback }: { id: string; onback: () => void } = $props();

  const plan = $derived(plans.find((p) => p.id === id));

  const levels: { id: Level; label: string; hint: string }[] = [
    { id: "L1", label: "静默", hint: "只在贴边条上显示" },
    { id: "L2", label: "系统通知", hint: "进 Windows 通知中心" },
    { id: "L3", label: "强提醒", hint: "停在最前面，等你回应" },
  ];

  function clock(hour: number) {
    const h = Math.floor(hour);
    const m = Math.round((hour - h) * 60);
    return String(h).padStart(2, "0") + ":" + String(m).padStart(2, "0");
  }
</script>

{#if plan}
  <article class="detail">
    <label class="field">
      <span class="lb">标题</span>
      <input bind:value={plan.title} spellcheck="false" placeholder="这条计划叫什么" />
    </label>

    <label class="field">
      <span class="lb">说明</span>
      <input bind:value={plan.note} spellcheck="false" placeholder="这条计划要做什么" />
    </label>

    <div class="row">
      <span class="lb">时间</span>
      <span class="value num">{clock(plan.start)} – {clock(plan.end)}</span>
    </div>

    <div class="row">
      <span class="lb">触发</span>
      <div class="chips">
        <button class="chip" class:on={plan.repeat === "once"} onclick={() => (plan.repeat = "once")}>一次</button>
        <button class="chip" class:on={plan.repeat === "daily"} onclick={() => (plan.repeat = "daily")}>每天</button>
      </div>
    </div>

    <div class="row">
      <span class="lb">通知</span>
      <div class="chips">
        {#each levels as level (level.id)}
          <button
            class="chip"
            class:on={plan.level === level.id}
            title={level.hint}
            onclick={() => (plan.level = level.id)}
          >
            {level.label}
          </button>
        {/each}
      </div>
    </div>

    <div class="row">
      <span class="lb">角色</span>
      <span class="value">{plan.role}</span>
    </div>

    <div class="actions">
      <button class="btn" onclick={() => toggleDone(plan.id)}>
        {plan.done ? "标记为待执行" : "标记为已完成"}
      </button>
      <button
        class="btn danger"
        onclick={() => {
          removePlan(plan.id);
          onback();
        }}
      >
        删除计划
      </button>
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

  .value {
    font-size: 12.5px;
    color: var(--text);
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

  .chip:hover {
    background: var(--hover);
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

  .btn.danger {
    border-color: rgba(240, 74, 74, 0.35);
    color: var(--red);
  }

  .btn.danger:hover {
    background: var(--red-dim);
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
