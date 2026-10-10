<script lang="ts">
  import { slide } from "svelte/transition";
  import {
    clock,
    keyOf,
    parseKey,
    plans,
    removePlan,
    todayKey,
    toggleDone,
    view,
    WEEKDAYS,
    type Plan,
  } from "./store.svelte";

  let { onopen, oncreate }: { onopen: (id: string) => void; oncreate: () => void } = $props();

  // Collapsed by default: one week is what a day planner actually needs on screen.
  let calOpen = $state(false);

  let cursor = $state(new Date(new Date().getFullYear(), new Date().getMonth(), 1));

  function dayCell(date: Date, out: boolean) {
    const key = keyOf(date);
    return {
      key,
      date: date.getDate(),
      out,
      today: key === todayKey,
      count: plans.filter((p) => p.day === key).length,
    };
  }

  // Collapsed it is the single week around the selected day; expanded, six rows always, so
  // switching months must not make the grid jump.
  const cells = $derived.by(() => {
    if (!calOpen) {
      const anchor = parseKey(view.day);
      const monday = new Date(anchor);
      monday.setDate(anchor.getDate() - ((anchor.getDay() + 6) % 7));
      return Array.from({ length: 7 }, (_, i) => {
        const date = new Date(monday);
        date.setDate(monday.getDate() + i);
        return dayCell(date, false);
      });
    }
    const first = new Date(cursor.getFullYear(), cursor.getMonth(), 1);
    const start = new Date(first);
    start.setDate(1 - ((first.getDay() + 6) % 7));
    return Array.from({ length: 42 }, (_, i) => {
      const date = new Date(start);
      date.setDate(start.getDate() + i);
      return dayCell(date, date.getMonth() !== cursor.getMonth());
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

  // "即将到来" answers "what is coming up" without clicking a calendar day at a time.
  let scope = $state<"day" | "upcoming">("day");

  const upcoming = $derived(
    plans.filter((p) => !p.done).sort((a, b) => a.day.localeCompare(b.day) || a.start - b.start),
  );

  function dayLabel(key: string) {
    if (key === todayKey) return "今天";
    const date = parseKey(key);
    return `${date.getMonth() + 1} 月 ${date.getDate()} 日 ${WEEKDAYS[(date.getDay() + 6) % 7]}`;
  }

  const groups = $derived.by(() => {
    if (scope === "day") {
      return [
        { key: "待执行", items: open },
        { key: "已完成", items: done },
      ];
    }
    const byDay = new Map<string, Plan[]>();
    for (const plan of upcoming) byDay.set(plan.day, [...(byDay.get(plan.day) ?? []), plan]);
    return [...byDay.entries()].map(([day, items]) => ({ key: dayLabel(day), items }));
  });

  // Deleting takes a second click, and the arm expires on its own so a stray click never
  // leaves a row one keystroke from gone.
  let armed = $state<string | null>(null);
  let armTimer: ReturnType<typeof setTimeout> | undefined;

  function armDelete(id: string) {
    clearTimeout(armTimer);
    if (armed === id) {
      armed = null;
      removePlan(id);
      return;
    }
    armed = id;
    armTimer = setTimeout(() => (armed = null), 4000);
  }

  $effect(() => () => clearTimeout(armTimer));

</script>

<section class="planner">
  <div class="head">
    {#if scope === "day"}
      <button class="twist" class:closed={!calOpen} onclick={() => (calOpen = !calOpen)}>
        <svg viewBox="0 0 16 16" width="12" height="12" fill="none" stroke="currentColor" stroke-width="1.8">
          <path d="M4 6.5L8 10.5l4-4" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
        {calOpen ? "收起日历" : "展开完整日历"}
      </button>
      <span class="strong">{selLabel}</span>
      <span class="count num">{todays.length} 项</span>
      <span class="sep"></span>
      <span class="week-num num">本周 {weekCount} 项</span>
    {:else}
      <span class="strong">即将到来</span>
      <span class="count num">{upcoming.length} 项待执行</span>
    {/if}
    <span class="grow"></span>
    <button
      class="scope"
      class:on={scope === "upcoming"}
      onclick={() => (scope = scope === "day" ? "upcoming" : "day")}
    >
      {scope === "day" ? "即将到来" : "回到当天"}
    </button>
    <button class="new" onclick={oncreate}>
      <svg viewBox="0 0 16 16" width="12" height="12" fill="none" stroke="currentColor" stroke-width="1.7">
        <path d="M8 3.6v8.8M3.6 8h8.8" stroke-linecap="round" />
      </svg>
      新建计划
    </button>
  </div>

  {#if scope === "day" && calOpen}
    <div class="cbar" transition:slide={{ duration: 180 }}>
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
  {/if}

  {#if scope === "day"}
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
  {/if}

  <div class="list">
    {#if scope === "day" && todays.length === 0}
      <div class="blank">
        <p class="b1">{plans.length === 0 ? "还没有任何计划" : "这一天没有安排"}</p>
        <p class="b2">在上面输入框写一句，比如「写周报 9:30」，回车就能建；或者点右上角「新建计划」。</p>
        <button class="new" onclick={oncreate}>新建一条计划</button>
      </div>
    {:else if scope === "upcoming" && upcoming.length === 0}
      <div class="blank">
        <p class="b1">没有待执行的计划</p>
        <p class="b2">都办完了，或者还没开始写。</p>
      </div>
    {:else}
      {#each groups as group (group.key)}
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
          <button
            class="del"
            class:armed={armed === plan.id}
            onclick={() => armDelete(plan.id)}
            aria-label={armed === plan.id ? "再点一次确认删除" : "删除"}
          >
            {#if armed === plan.id}
              <svg viewBox="0 0 16 16" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M3.5 8.4l3 3 6-6.6" stroke-linecap="round" stroke-linejoin="round" />
              </svg>
            {:else}
              <svg viewBox="0 0 16 16" width="12" height="12" fill="none" stroke="currentColor" stroke-width="1.7">
                <path d="M4.5 4.5l7 7M11.5 4.5l-7 7" stroke-linecap="round" />
              </svg>
            {/if}
          </button>
        </div>
      {/each}
        {#if group.items.length === 0}<p class="empty">暂无</p>{/if}
      {/each}
    {/if}
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
    display: flex;
    align-items: center;
    gap: 4px;
    height: 22px;
    padding: 0 6px 0 4px;
    border: 0;
    border-radius: 5px;
    background: none;
    color: var(--text-3);
    font: inherit;
    font-size: 11px;
    cursor: pointer;
  }

  /* Only the chevron turns; the label next to it has to stay upright. */
  .twist svg {
    flex: none;
    transition: transform 0.15s ease;
  }

  .twist.closed svg {
    transform: rotate(-90deg);
  }

  .twist:hover {
    background: var(--hover);
    color: var(--text);
  }

  .new {
    display: flex;
    align-items: center;
    gap: 4px;
    height: 24px;
    padding: 0 10px;
    border: 1px solid var(--line-strong);
    border-radius: var(--r-tag);
    background: var(--bg);
    color: var(--text-2);
    font: inherit;
    font-size: 11px;
    cursor: pointer;
  }

  .new:hover {
    border-color: var(--brand);
    color: var(--brand);
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

  /* An armed row says so even after the pointer leaves it. */
  .del.armed {
    opacity: 1;
    background: var(--red-dim);
    color: var(--red);
  }

  .scope {
    height: 24px;
    padding: 0 10px;
    border: 1px solid var(--line-strong);
    border-radius: var(--r-tag);
    background: var(--bg);
    color: var(--text-2);
    font: inherit;
    font-size: 11px;
    cursor: pointer;
  }

  .scope:hover {
    border-color: var(--brand);
    color: var(--brand);
  }

  .scope.on {
    border-color: transparent;
    background: var(--brand-dim);
    color: var(--brand);
    font-weight: 600;
  }

  .blank {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    padding: 34px 24px;
    text-align: center;
  }

  .b1 {
    margin: 0;
    font-size: 12.5px;
    font-weight: 600;
    color: var(--text-2);
  }

  /* First run has to say what the box above is for; two "暂无" do not. */
  .b2 {
    margin: 0;
    max-width: 380px;
    font-size: 11.5px;
    line-height: 1.7;
    color: var(--text-3);
  }
</style>
