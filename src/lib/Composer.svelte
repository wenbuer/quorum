<script lang="ts">
  import {
    addPlan,
    clock,
    plans,
    projects,
    roles,
    todayKey,
    ui,
    view,
    type Domain,
    type Plan,
    type Route,
  } from "./store.svelte";

  let {
    domain,
    onopen,
    onswitch,
  }: { domain: Domain; onopen: (route: Route) => void; onswitch: (tab: Domain) => void } = $props();

  const MAX_LEN = 200;

  type Chip = { kind: "project" | "role" | "action"; label: string };

  type CommandDef = { id: string; note: string; params: string[] };

  /// A row shows `label` when the id is only a key -- plan ids (p-3) are not names.
  type Offer = { id: string; note: string; label?: string; run: () => void };

  // First mode: no project chosen, so the general set applies -- creating roles included.
  const commands: CommandDef[] = [
    { id: "/新建计划", note: "创建一条计划", params: ["内容"] },
    { id: "/查看计划", note: "打开已有计划", params: [] },
    { id: "/新建角色", note: "创建一个角色", params: ["名称"] },
    { id: "/编辑角色", note: "修改已有角色", params: [] },
    { id: "/新建项目", note: "创建一个项目", params: ["名称", "路径"] },
    { id: "/编辑项目", note: "修改已有项目", params: [] },
    { id: "/执行角色", note: "选定执行的角色", params: [] },
  ];

  // Second mode: a project is chosen, so the chain may only act on that project.
  const projectCommands: CommandDef[] = [
    { id: "/交给角色", note: "选择执行的角色", params: [] },
    { id: "/添加路径", note: "把这个项目的路径加到角色上", params: ["路径"] },
  ];

  let text = $state("");
  let chips = $state<Chip[]>([]);
  let stage = $state<null | "role" | "plan">(null);
  let active = $state(0);
  let closed = $state(false);
  let pending = $state<{ id: string; params: string[]; filled: string[] } | null>(null);
  let draft = $state<
    { title: string; clock: string; daily: boolean; role: string; project: string; action: string } | null
  >(null);
  let inputEl = $state<HTMLInputElement | null>(null);

  // The panel is summoned to be typed into; landing on a form that ignores the keyboard
  // makes the hotkey pointless.
  $effect(() => {
    const id = setTimeout(() => inputEl?.focus(), 80);
    return () => clearTimeout(id);
  });

  // The box means different things per tab, so a half-built command does not travel with it.
  $effect(() => {
    domain;
    draft = null;
    chips = [];
    stage = null;
    pending = null;
    text = "";
  });

  const project = $derived(chips.find((c) => c.kind === "project"));
  const roleChip = $derived(chips.find((c) => c.kind === "role"));
  const actionChip = $derived(chips.find((c) => c.kind === "action"));

  const placeholder = $derived.by(() => {
    if (pending) return "填写" + pending.params[pending.filled.length];
    if (domain === "role") return "输入 / 新建角色，或 /编辑角色";
    if (domain === "project") return "输入 / 新建项目，或 /编辑项目";
    if (domain === "settings") return "输入 / 看命令";
    if (!project) return "写下计划，或输入 @ 选项目、/ 选命令";
    if (!roleChip) return "输入 / 交给角色 或 /添加路径";
    if (!actionChip) return "输入 / 添加路径，或直接执行";
    return "补一句说明，然后执行";
  });

  function reset() {
    text = "";
    closed = true;
    stage = null;
    pending = null;
    inputEl?.focus();
  }

  /// A command with argument slots collects them one chip at a time, then hands them to the
  /// page that owns the work.
  function finish(id: string, filled: string[]) {
    if (id === "/新建计划") {
      // Every slot is optional, so an empty one still has to land somewhere real: the create
      // form, blank. A filled one only pre-fills it -- nothing is written until 创建计划.
      ui.planDraft = planSeed(filled[0] ?? "");
      onopen({ name: "plan-new" });
    } else if (id === "/新建角色") {
      onswitch("role");
      ui.drawer = { kind: "role", name: filled[0] };
    } else if (id === "/新建项目") {
      ui.projectDraft = { name: filled[0] ?? "", path: filled[1] ?? "" };
      onopen({ name: "project-new" });
    } else if (id === "/添加路径") {
      add("action", filled[0] ?? "项目路径");
    }
    pending = null;
  }

  function add(kind: Chip["kind"], label: string) {
    chips = [...chips.filter((c) => c.kind !== kind), { kind, label }];
  }

  function run(id: string) {
    const def = [...commands, ...projectCommands].find((c) => c.id === id);
    if (def && def.params.length > 0) {
      pending = { id, params: def.params, filled: [] };
      text = "";
      closed = false;
      inputEl?.focus();
      return;
    }
    if (id === "/查看计划") {
      stage = "plan";
      text = "";
      closed = false;
      inputEl?.focus();
      return;
    }
    if (id === "/执行角色" || id === "/交给角色") {
      stage = "role";
      text = "";
      closed = false;
      inputEl?.focus();
      return;
    }
    if (id === "/新建角色") {
      onswitch("role");
      ui.drawer = { kind: "role" };
      reset();
      return;
    }
    if (id === "/新建项目") {
      onopen({ name: "project-new" });
      reset();
      return;
    }
    if (id === "/编辑角色") {
      text = "/编辑角色";
      closed = false;
      inputEl?.focus();
      return;
    }
    if (id === "/编辑项目") {
      text = "/编辑项目";
      closed = false;
      inputEl?.focus();
      return;
    }
    if (id === "/添加路径") {
      add("action", "项目路径");
      reset();
      return;
    }
    reset();
  }

  const offers = $derived.by((): Offer[] => {
    if (closed) return [];
    const value = text.trimStart();

    if (stage === "plan") {
      return [...plans]
        .sort((a, b) => a.day.localeCompare(b.day) || a.start - b.start)
        .filter((p) => p.title.includes(value))
        .map((p) => ({
          id: p.id,
          label: p.title,
          note:
            (p.day === todayKey ? "今天" : p.day.slice(5).replace("-", "/")) +
            " " + clock(p.start) + (p.done ? " · 已完成" : ""),
          run: () => {
            onopen({ name: "plan", id: p.id });
            reset();
          },
        }));
    }

    if (stage === "role") {
      return roles
        .filter((r) => r.name.includes(value))
        .map((r) => ({
          id: r.name,
          note: "交给这个角色",
          run: () => {
            add("role", r.name);
            reset();
          },
        }));
    }

    if (value.startsWith("@")) {
      // @ opens project mode and nothing else; inside a / chain it does not exist.
      if (project) return [];
      const query = value.slice(1).trim();
      return projects
        .filter((p) => p.name.includes(query))
        .map((p) => ({
          id: "@" + p.name,
          note: "项目",
          run: () => {
            add("project", p.name);
            chips = chips.filter((c) => c.kind === "project" || c.kind === "role");
            reset();
          },
        }));
    }

    if (value.startsWith("/")) {
      const head = value.split(/\s/)[0];

      // Must come before the plain command list, or it is never reached.
      const editingRole = value.match(/^\/编辑角色\s*(.*)$/);
      if (editingRole) {
        const query = editingRole[1].trim();
        return roles
          .filter((r) => r.name.includes(query))
          .map((r) => ({
            id: r.name,
            note: "编辑这个角色",
            run: () => {
              onswitch("role");
              ui.drawer = { kind: "role", id: r.id };
              reset();
            },
          }));
      }

      const editingProject = value.match(/^\/编辑项目\s*(.*)$/);
      if (editingProject) {
        const query = editingProject[1].trim();
        return projects
          .filter((p) => p.name.includes(query))
          .map((p) => ({
            id: p.name,
            note: "编辑这个项目",
            run: () => {
              onopen({ name: "project", id: p.id });
              reset();
            },
          }));
      }

      const list = project ? projectCommands : commands;
      return list
        .filter((c) => c.id.startsWith(head))
        .map((c) => ({ id: c.id, note: c.note, run: () => run(c.id) }));
    }

    return [];
  });

  const canSubmit = $derived(
    pending ? true : domain === "plan" && (text.trim().length > 0 || chips.length > 0),
  );

  function readPlan(input: string) {
    const time = input.match(/(\d{1,2})\s*[:：点]\s*(\d{1,2})?/);
    const daily = /每天|每日/.test(input);
    const clock = time ? time[1].padStart(2, "0") + ":" + (time[2] ?? "00").padStart(2, "0") : "";
    const body = input
      .replace(/每天|每日/g, " ")
      .replace(/\d{1,2}\s*[:：点]\s*\d{0,2}\s*分?/g, " ")
      .replace(/\s+/g, " ")
      .trim();
    const fallback = [actionChip?.label, project?.label].filter(Boolean).join(" · ");
    return { title: body || fallback || "新计划", clock, daily };
  }

  function submit() {
    if (!canSubmit) return;
    if (pending) {
      const value = text.trim();
      text = "";
      closed = true;
      // Params are optional: an empty answer ends the chain instead of parking here.
      if (!value) finish(pending.id, pending.filled);
      else {
        const filled = [...pending.filled, value];
        if (filled.length >= pending.params.length) finish(pending.id, filled);
        else pending = { ...pending, filled };
      }
      inputEl?.focus();
      return;
    }
    closed = true;
    openDraft(text.trim());
  }

  /// What was typed, shaped for the create form's one-shot seed. Only what was actually
  /// understood goes in; the rest of the form keeps its defaults.
  function planSeed(body: string): Partial<Omit<Plan, "id">> | null {
    const text = body.trim();
    if (!text) return null;
    const read = readPlan(text);
    const [h, m] = read.clock ? read.clock.split(":").map(Number) : [NaN, NaN];
    return {
      title: read.title,
      ...(Number.isFinite(h) ? { start: h + m / 60, end: h + m / 60 + 0.5 } : {}),
      repeat: read.daily ? ("daily" as const) : ("once" as const),
      ...(roleChip ? { role: roleChip.label } : {}),
    };
  }

  /// The card is the confirmation step: nothing is written until the user accepts it.
  function openDraft(body: string) {
    draft = {
      ...readPlan(body),
      role: roleChip?.label ?? "编排",
      project: project?.label ?? "",
      action: actionChip?.label ?? "",
    };
  }

  function accept() {
    if (!draft) return;
    const [h, m] = draft.clock ? draft.clock.split(":") : [];
    const start = h ? Number(h) + Number(m) / 60 : 12;
    addPlan({
      day: view.day,
      start,
      end: start + 0.5,
      title: draft.title,
      note: [draft.project, draft.action].filter(Boolean).join(" · "),
      role: draft.role,
      level: "L1",
      repeat: draft.daily ? "daily" : "once",
      done: false,
    });
    draft = null;
    text = "";
    chips = [];
    // Created from another tab, the new plan would otherwise land where nobody is looking.
    onswitch("plan");
    inputEl?.focus();
  }

  /// Escape peels one layer at a time -- menu, then the last capsule -- and only lets the
  /// panel close once the box is empty again.
  function stepBack(event: KeyboardEvent) {
    if (closed === false && offers.length > 0) {
      event.preventDefault();
      event.stopPropagation();
      closed = true;
      return;
    }
    if (pending) {
      event.preventDefault();
      event.stopPropagation();
      pending = pending.filled.length > 0 ? { ...pending, filled: pending.filled.slice(0, -1) } : null;
      text = "";
      inputEl?.focus();
      return;
    }
    if (chips.length > 0) {
      event.preventDefault();
      event.stopPropagation();
      chips = chips.slice(0, -1);
    }
  }

  function onKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      stepBack(event);
      return;
    }
    if (event.key === "Backspace" && text === "" && chips.length > 0) {
      event.preventDefault();
      chips = chips.slice(0, -1);
      return;
    }
    if (offers.length > 0) {
      if (event.key === "ArrowDown") {
        event.preventDefault();
        active = (active + 1) % offers.length;
        return;
      }
      if (event.key === "ArrowUp") {
        event.preventDefault();
        active = (active - 1 + offers.length) % offers.length;
        return;
      }
      if (event.key === "Enter") {
        event.preventDefault();
        offers[active]?.run();
        return;
      }
    }
    if (event.key === "Enter") submit();
  }
</script>

<div class="composer">
  <div class="row">
    <button
      class="lead"
      aria-label="新建计划"
      onmousedown={(e) => e.preventDefault()}
      onclick={() => run("/新建计划")}
    >
      <svg viewBox="0 0 16 16" width="15" height="15" fill="none" stroke="currentColor" stroke-width="1.5">
        <path d="M8 3.2v9.6M3.2 8h9.6" stroke-linecap="round" />
      </svg>
    </button>

    {#if pending}
      <span class="cmd">{pending.id}</span>
      {#each pending.filled as value, i (i)}
        <span class="fill">
          <span class="pname tone{i % 3}">{pending.params[i]}</span>
          <span class="pval">{value}</span>
        </span>
      {/each}
    {/if}

    {#each chips as chip, i (chip.kind)}
      <span class="chip {chip.kind}">
        {chip.label}
        <button
          class="unpick"
          aria-label="移除"
          onmousedown={(e) => e.preventDefault()}
          onclick={() => (chips = chips.filter((_, at) => at !== i))}
        >
          <svg viewBox="0 0 16 16" width="10" height="10" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M4.5 4.5l7 7M11.5 4.5l-7 7" stroke-linecap="round" />
          </svg>
        </button>
      </span>
    {/each}

    <input
      bind:this={inputEl}
      bind:value={text}
      maxlength={MAX_LEN}
      spellcheck="false"
      {placeholder}
      oninput={() => { closed = false; active = 0; }}
      onkeydown={onKeydown}
      onfocus={() => (closed = false)}
      onblur={() => (closed = true)}
    />
    <button class="send" onclick={submit} disabled={!canSubmit}>
      {pending || chips.length > 0 ? "执行" : "解析"}
    </button>
  </div>

  {#if offers.length > 0}
    <ul class="offers">
      {#each offers as offer, i (offer.id)}
        <li>
          <button
            class="offer"
            class:active={i === active}
            onmousedown={(e) => e.preventDefault()}
            onclick={() => offer.run()}
            onmouseenter={() => (active = i)}
          >
            <span class="oid">{offer.label ?? offer.id}</span>
            <span class="onote">{offer.note}</span>
          </button>
        </li>
      {/each}
    </ul>
  {/if}

  {#if draft}
    <div class="draft">
      <span class="dlabel">将创建计划</span>
      <span class="dtitle">{draft.title}</span>
      <span class="dmeta num">{draft.daily ? "每天 " : ""}{draft.clock || "时间待定"}</span>
      <span class="dmeta">{draft.project || "未选项目"}</span>
      <span class="dmeta">{draft.action || "未选动作"}</span>
      <span class="dmeta">{draft.role}</span>
      <span class="grow"></span>
      <button class="dact" onclick={() => (draft = null)}>取消</button>
      <button class="dact primary" onclick={accept}>创建计划</button>
    </div>
  {/if}
</div>

<style>
  .composer {
    position: relative;
    padding: 10px 12px 8px;
    border-bottom: 1px solid var(--line);
  }

  .row {
    display: flex;
    align-items: center;
    gap: 6px;
    height: 38px;
    padding: 0 6px 0 10px;
    border: 1px solid var(--line-strong);
    border-radius: 10px;
    background: var(--sunken);
  }

  .row:focus-within {
    border-color: var(--brand);
    box-shadow: 0 0 0 3px var(--brand-dim);
    background: var(--bg);
  }

  .lead {
    display: grid;
    place-items: center;
    color: var(--brand);
    flex: none;
    border: 0;
    background: none;
    padding: 2px;
    border-radius: 6px;
    cursor: pointer;
  }

  .lead:hover {
    background: var(--brand-dim);
  }

  .chip {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    height: 24px;
    padding: 0 4px 0 9px;
    border-radius: 12px;
    font-size: 11.5px;
    font-weight: 600;
    white-space: nowrap;
    flex: none;
  }

  .chip.project {
    background: var(--brand);
    color: #fff;
  }

  .chip.role {
    background: var(--brand-dim);
    color: var(--brand);
  }

  .chip.action {
    background: var(--pressed);
    color: var(--text-2);
  }

  .cmd {
    display: inline-flex;
    align-items: center;
    height: 24px;
    padding: 0 10px;
    border-radius: 12px;
    background: var(--brand);
    color: #fff;
    font-size: 11.5px;
    font-weight: 600;
    white-space: nowrap;
    flex: none;
  }

  .fill {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 24px;
    padding: 0 9px 0 3px;
    border: 1px solid var(--line-strong);
    border-radius: 12px;
    background: var(--bg);
    max-width: 240px;
    flex: none;
  }

  .pname {
    display: inline-flex;
    align-items: center;
    height: 18px;
    padding: 0 7px;
    border-radius: 9px;
    font-size: 10.5px;
    font-weight: 600;
    flex: none;
  }

  .tone0 {
    background: var(--brand-dim);
    color: var(--brand);
  }

  .tone1 {
    background: var(--green-dim);
    color: var(--green);
  }

  .tone2 {
    background: var(--orange-dim);
    color: var(--orange);
  }

  .pval {
    font-size: 11.5px;
    color: var(--text-2);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .unpick {
    display: grid;
    place-items: center;
    width: 16px;
    height: 16px;
    padding: 0;
    border: 0;
    border-radius: 50%;
    background: none;
    color: inherit;
    opacity: 0.7;
    cursor: pointer;
  }

  .unpick:hover {
    opacity: 1;
    background: rgba(255, 255, 255, 0.25);
  }

  input {
    flex: 1;
    min-width: 70px;
    height: 100%;
    border: 0;
    background: none;
    color: var(--text);
    font: inherit;
    font-size: 12.5px;
    outline: none;
  }

  input::placeholder {
    color: var(--text-4);
  }

  .send {
    height: 26px;
    padding: 0 12px;
    border: 0;
    border-radius: 7px;
    background: var(--brand);
    color: #fff;
    font: inherit;
    font-size: 11.5px;
    font-weight: 600;
    cursor: pointer;
    flex: none;
  }

  .send:disabled {
    background: var(--pressed);
    color: var(--text-4);
    cursor: default;
  }

  .offers {
    position: absolute;
    left: 18px;
    right: 18px;
    top: 52px;
    z-index: 20;
    margin: 0;
    padding: 4px;
    list-style: none;
    border: 1px solid var(--line-strong);
    border-radius: 10px;
    background: var(--bg);
    box-shadow: 0 12px 28px rgba(31, 35, 41, 0.14);
  }

  .offer {
    display: flex;
    align-items: baseline;
    gap: 10px;
    width: 100%;
    padding: 7px 10px;
    border: 0;
    border-radius: 7px;
    background: none;
    color: inherit;
    font: inherit;
    text-align: left;
    cursor: pointer;
  }

  .offer.active {
    background: var(--brand-dim);
  }

  .oid {
    font-size: 12.5px;
    font-weight: 600;
    color: var(--brand);
    flex: none;
  }

  .onote {
    font-size: 11px;
    color: var(--text-3);
  }

  .draft {
    display: flex;
    align-items: center;
    gap: 9px;
    margin-top: 8px;
    padding: 7px 8px 7px 12px;
    border: 1px solid rgba(43, 108, 246, 0.25);
    border-radius: 9px;
    background: var(--brand-dim);
  }

  .dlabel {
    font-size: 10px;
    font-weight: 600;
    color: var(--brand);
    flex: none;
  }

  .dtitle {
    font-size: 12px;
    color: var(--text);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .dmeta {
    font-size: 11px;
    color: var(--text-2);
    flex: none;
  }

  .grow {
    flex: 1;
  }

  .dact {
    height: 24px;
    padding: 0 10px;
    border: 1px solid var(--line-strong);
    border-radius: 6px;
    background: var(--bg);
    color: var(--text);
    font: inherit;
    font-size: 11px;
    cursor: pointer;
    flex: none;
  }

  .dact.primary {
    border-color: var(--brand);
    background: var(--brand);
    color: #fff;
    font-weight: 600;
  }
</style>
