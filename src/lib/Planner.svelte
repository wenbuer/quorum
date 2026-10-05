<script lang="ts">
  import { slide } from "svelte/transition";
  import { keyOf, parseKey, plans, removePlan, todayKey, toggleDone, view, WEEKDAYS } from "./store.svelte";

  let { onopen }: { onopen: (id: string) => void } = $props();

  let calOpen = $state(true);

  let cursor = $state(new Date(new Date().getFullYear(), new Date().getMonth(), 1));

  // Six rows always: switching months must not make the grid jump.
  const cells = $derived.by(() => {
    const first = new Date(cursor.getFullYear(), cursor.getMonth(), 1);
    const start = new Date(first);
    start.setDate(1 - ((first.getDay() + 6) % 7));
    return Array.from({ length: 42 }, (_, i) => {
      const date = new Date(start);
      date.setDate(start.getDate() + i);
      const key = keyOf(date);
      return {
        key,
        date: date.getDate(),
        out: date.getMonth() !== cursor.getMonth(),
        today: key === todayKey,
        count: plans.filter((p) => p.day === key).length,
      };
    });
  });

  const month = $derived(cursor.getFullYear() + " 年 " + (cursor.getMonth() + 1) + " 月");

  function shiftMonth(by: number) {
    cursor = new Date(cursor.getFullYear(), cursor.getMonth() + by, 1);
  }

  function toToday() {
    const now = new Date();
    cursor = new Date(now.getFullYear(), now.getMonth(), 1);
    view.day = todayKey;
  }

  const selDate = $derived(parseKey(view.day));
  const selLabel = $derived(
    view.day === todayKey
      ? "今日"
      : selDate.getMonth() + 1 + " 月 " + selDate.getDate() + " 日 " + WEEKDAYS[(selDate.getDay() + 6) % 7],
  );

  const weekCount = $derived.by(() => {
    const monday = new Date(selDate);
    monday.setDate(selDate.getDate() - ((selDate.getDay() + 6) % 7));
    return plans.filter((p) => {
      const gap = (parseKey(p.day).getTime() - monday.getTime()) / 86400000;
      return gap >= 0 && gap < 7;
    }).length;
  });

  const todays = $derived([...plans.filter((p) => p.day === view.day)].sort((a, b) => a.start - b.start));
  const open = $derived(todays.filter((p) => !p.done));
  const done = $derived(todays.filter((p) => p.done));

  function clock(hour: number) {
    const h = Math.floor(hour);
    const m = Math.round((hour - h) * 60);
    return String(h).padStart(2, "0") + ":" + String(m).padStart(2, "0");
  }
</script>

<section class="planner">
  <div class="head">
    <button class="twist" class:closed={!calOpen} onclick={() => (calOpen = !calOpen)} aria-label="展开日历">
      <svg viewBox="0 0 16 16" width="12" height="12" fill="none" stroke="currentColor" stroke-width="1.8">
        <path d="M4 6.5L8 10.5l4-4" stroke-linecap="round" stroke-linejoin="round" />
      </svg>
    </button>
    <span class="strong">{selLabel}</span>
    <span class="count num">{todays.length} 项</span>
    <span class="sep"></span>
    <span class="week-num num">本周 {weekCount} 项</span>
  </div>

  {#if calOpen}
    <div class="cal" transition:slide={{ duration: 180 }}>
      <div class="cbar">
        <button class="prev" onclick={() => shiftMonth(-1)} aria-label="上个月">
          <svg viewBox="0 0 16 16" width="12" height="12" fill="none" stroke="currentColor" stroke-width="1.8">
            <path d="M10 3.5L5.5 8l4.5 4.5" stroke-linecap="round" stroke-linejoin="round" />
          </svg>
        </button>
        <span class="ctitle num">{month}</span>
        <button class="next" onclick={() => shiftMonth(1)} aria-label="下个月">
          <svg viewBox="0 0 16 16" width="12" height="12" fill="none" stroke="currentColor" stroke-width="1.8">
            <path d="M6 3.5L10.5 8 6 12.5" stroke-linecap="round" stroke-linejoin="round" />
          </svg>
        </button>
        <span class="grow"></span>
        <button class="back-today" onclick={toToday}>回到今天</button>
      </div>

      <div class="wd">
        {#each WEEKDAYS as name (name)}<span>{name}</span>{/each}
      </div>

      <div class="cgrid">
        {#each cells as cell (cell.key)}
          <button
            class="cell"
            class:out={cell.out}
            class:today={cell.today}
            class:on={view.day === cell.key}
            onclick={() => (view.day = cell.key)}
          >
            <span class="cn num">{cell.date}</span>
            <span class="dots">
              {#each Array(Math.min(cell.count, 3)) as _, i (i)}<i></i>{/each}
            </span>
          </button>
        {/each}
      </div>
    </div>
  {/if}

  <div class="list">
    {#each [{ key: "待执行", items: open }, { key: "已完成", items: done }] as group (group.key)}
      <p class="group">{group.key} · {group.items.length}</p>
      {#each group.items as plan (plan.id)}
        <div class="line">
          <span class="clock num">{clock(plan.start)}</span>
          <button class="tick-box" class:on={plan.done} onclick={() => toggleDone(plan.id)} aria-label="完成">
            {#if plan.done}
              <svg viewBox="0 0 16 16" width="10" height="10" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M3.5 8.4l3 3 6-6.6" stroke-linecap="round" stroke-linejoin="round" />
              </svg>
            {/if}
          </button>
          <button class="title" onclick={() => onopen(plan.id)}>{plan.title}</button>
          <span class="who">{plan.role}</span>
          <span class="lvl {plan.level.toLowerCase()} num">{plan.level}</span>
          <button class="del" onclick={() => removePlan(plan.id)} aria-label="删除">
            <svg viewBox="0 0 16 16" width="12" height="12" fill="none" stroke="currentColor" stroke-width="1.7">
              <path d="M4.5 4.5l7 7M11.5 4.5l-7 7" stroke-linecap="round" />
            </svg>
          </button>
        </div>
      {/each}
      {#if group.items.length === 0}<p class="empty">暂无</p>{/if}
    {/each}
  </div>
</section>

<style>
  .planner {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
  }

  .head {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 14px 8px;
  }

  .twist {
    display: grid;
    place-items: center;
    width: 20px;
    height: 20px;
    padding: 0;
    border: 0;
    border-radius: 5px;
    background: none;
    color: var(--text-3);
    cursor: pointer;
    transition: transform 0.15s ease;
  }

  .twist.closed {
    transform: rotate(-90deg);
  }

  .twist:hover {
    background: var(--hover);
    color: var(--text);
  }

  .strong {
    font-size: 12.5px;
    font-weight: 600;
    color: var(--text);
  }

  .count,
  .week-num {
    font-size: 11px;
    color: var(--text-3);
  }

  .sep {
    width: 1px;
    height: 10px;
    background: var(--line-strong);
  }

  .grow {
    flex: 1;
  }

  .cbar {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 0 14px 6px;
  }

  .ctitle {
    font-size: 12px;
    font-weight: 600;
    color: var(--text);
  }

  .prev,
  .next {
    display: grid;
    place-items: center;
    width: 20px;
    height: 20px;
    padding: 0;
    border: 0;
    border-radius: 5px;
    background: none;
    color: var(--text-3);
    cursor: pointer;
  }

  .prev:hover,
  .next:hover {
    background: var(--hover);
    color: var(--text);
  }

  .back-today {
    padding: 3px 9px;
    border: 1px solid var(--line-strong);
    border-radius: var(--r-tag);
    background: var(--bg);
    color: var(--text-2);
    font: inherit;
    font-size: 10.5px;
    cursor: pointer;
  }

  .back-today:hover {
    border-color: var(--brand);
    color: var(--brand);
  }

  .wd {
    display: grid;
    grid-template-columns: repeat(7, 1fr);
    padding: 0 14px 2px;
  }

  .wd span {
    text-align: center;
    font-size: 10px;
    color: var(--text-4);
  }

  .cgrid {
    display: grid;
    grid-template-columns: repeat(7, 1fr);
    gap: 2px;
    padding: 0 14px 10px;
  }

  .cell {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 2px;
    height: 36px;
    padding: 0;
    border: 0;
    border-radius: var(--r-btn);
    background: none;
    color: inherit;
    font: inherit;
    cursor: pointer;
  }

  .cell:hover {
    background: var(--hover);
  }

  .cn {
    display: grid;
    place-items: center;
    width: 24px;
    height: 24px;
    border-radius: 50%;
    font-size: 12px;
    color: var(--text);
  }

  .cell.out .cn {
    color: var(--text-4);
  }

  .cell.today .cn {
    color: var(--brand);
    font-weight: 700;
    box-shadow: inset 0 0 0 1px rgba(43, 108, 246, 0.35);
  }

  .cell.on .cn {
    background: var(--brand);
    color: #fff;
    font-weight: 600;
    box-shadow: none;
  }

  .dots {
    display: flex;
    gap: 3px;
    height: 4px;
  }

  .dots i {
    width: 4px;
    height: 4px;
    border-radius: 50%;
    background: var(--text-4);
  }

  .cell.today .dots i {
    background: var(--brand);
  }

  .list {
    flex: 1;
    min-height: 0;
    padding: 6px 16px 14px;
    overflow-y: auto;
  }

  .line {
    display: flex;
    align-items: center;
    gap: 10px;
    height: 40px;
    padding: 0 6px;
    border-bottom: 1px solid var(--line);
  }

  .line:hover {
    background: var(--sunken);
  }

  .line:hover .del {
    opacity: 1;
  }

  .clock {
    width: 42px;
    flex: none;
    font-size: 10.5px;
    color: var(--text-3);
  }

  .tick-box {
    display: grid;
    place-items: center;
    width: 16px;
    height: 16px;
    padding: 0;
    border: 1px solid var(--line-strong);
    border-radius: 5px;
    background: var(--bg);
    color: var(--green);
    cursor: pointer;
    flex: none;
  }

  .tick-box.on {
    border-color: var(--green);
    background: var(--green-dim);
  }

  .title {
    flex: 1;
    min-width: 0;
    padding: 0;
    border: 0;
    background: none;
    color: var(--text);
    font: inherit;
    font-size: 12.5px;
    text-align: left;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    cursor: pointer;
  }

  .title:hover {
    color: var(--brand);
  }

  .who {
    font-size: 11px;
    color: var(--text-3);
    flex: none;
  }

  .lvl {
    padding: 1px 6px;
    border-radius: var(--r-tag);
    font-size: 9.5px;
    flex: none;
  }

  .lvl.l1 { color: var(--text-3); background: var(--pressed); }
  .lvl.l2 { color: var(--brand); background: var(--brand-dim); }
  .lvl.l3 { color: var(--orange); background: var(--orange-dim); }

  .group {
    margin: 12px 6px 4px;
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.06em;
    color: var(--text-4);
  }

  .empty {
    margin: 18px 0 0;
    text-align: center;
    font-size: 11.5px;
    color: var(--text-4);
  }

  .del {
    display: grid;
    place-items: center;
    width: 20px;
    height: 20px;
    padding: 0;
    border: 0;
    border-radius: 5px;
    background: none;
    color: var(--text-4);
    opacity: 0;
    cursor: pointer;
    flex: none;
  }

  .del:hover {
    background: var(--red-dim);
    color: var(--red);
  }
</style>
