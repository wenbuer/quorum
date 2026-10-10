<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import Capsule from "./lib/Capsule.svelte";
  import Panel from "./lib/Panel.svelte";
  import Popup from "./lib/Popup.svelte";
  import {
    adoptIds,
    domainOf,
    plans,
    projects,
    roles,
    sameRoute,
    settings,
    ui,
    type Domain,
    type Plan,
    type Project,
    type Role,
    type Route,
  } from "./lib/store.svelte";

  type Mode = "capsule" | "edge" | "panel";

  // Both windows run this same bundle; Tauri tells them apart by window label.
  function currentLabel() {
    try {
      return getCurrentWindow().label;
    } catch {
      return "";
    }
  }

  // Hash flags let a plain browser render one state at a time; the Tauri webview has no hash.
  const flags = new Set(
    (typeof location === "undefined" ? "" : location.hash.slice(1)).split("+").filter(Boolean),
  );
  const isPopup = currentLabel() === "popup" || flags.has("popup");
  const wantsPanel = flags.has("panel") || flags.has("roles");

  let mode = $state<Mode>(wantsPanel ? "panel" : flags.has("edge") ? "edge" : "capsule");
  let collapseTimer: ReturnType<typeof setTimeout> | undefined;

  // Each tab keeps its own trail, so back never crosses into the other tab and running the
  // trail dry lands on that tab's own root.
  const initialTab: Domain = flags.has("roles") ? "role" : "plan";
  let tab = $state<Domain>(initialTab);
  let hist = $state<Record<Domain, Route[]>>({
    plan: [{ name: "planner" }],
    role: [{ name: "roles" }],
    project: [{ name: "projects" }],
    settings: [{ name: "settings" }],
  });

  const trail = $derived(hist[tab]);
  const route = $derived(trail[trail.length - 1]);
  const canBack = $derived(trail.length > 1);

  function setMode(next: Mode) {
    if (mode === next) return;
    mode = next;
    invoke("set_mode", { mode: next }).catch((e) => console.error("set_mode failed", e));
  }

  function open(next: Route) {
    const domain = domainOf(next);
    tab = domain;
    const here = hist[domain];
    if (sameRoute(here[here.length - 1], next)) return;
    hist[domain] = [...here, next];
  }

  function back() {
    const here = hist[tab];
    if (here.length > 1) hist[tab] = here.slice(0, -1);
  }

  /// Put the capsule back where it was; the kernel picks capsule or icon from its own state.
  function restore() {
    invoke("set_mode", { mode: "restore" }).catch((e) => console.error("restore failed", e));
  }

  function onEnter() {
    clearTimeout(collapseTimer);
    if (mode === "edge") setMode("capsule");
  }

  function onLeave() {
    if (mode !== "capsule" || !settings.floatingIcon) return;
    clearTimeout(collapseTimer);
    collapseTimer = setTimeout(() => setMode("edge"), 320);
  }

  function onKeydown(event: KeyboardEvent) {
    // Escape never answers a strong reminder: that card is meant to stay until it is
    // actually dealt with, and a stray keypress should not count as dealing with it.
    if (event.key !== "Escape" || isPopup) return;
    if (ui.drawer) ui.drawer = null;
    else restore();
  }

  // The icon can be switched off, and the kernel has to know -- not just the UI.
  $effect(() => {
    if (isPopup) return;
    invoke("set_floating_icon", { enabled: settings.floatingIcon }).catch(() => {});
  });

  // Settings live in a JSON file the kernel owns; the frontend owns the shape. The hotkey
  // is deliberately not read back here: the kernel already registered it, and what it
  // actually holds is the truth (a taken shortcut falls back to another one).
  let restored = $state(false);
  let saveTimer: ReturnType<typeof setTimeout> | undefined;

  $effect(() => {
    if (isPopup) return;
    invoke<Record<string, unknown>>("load_settings")
      .then((saved) => {
        if (typeof saved.capsule === "string") settings.capsule = saved.capsule;
        if (typeof saved.floatingIcon === "boolean") settings.floatingIcon = saved.floatingIcon;
        if (saved.llm && typeof saved.llm === "object") Object.assign(settings.llm, saved.llm);
      })
      .catch(() => {})
      .finally(() => (restored = true));
  });

  $effect(() => {
    const snapshot = {
      hotkey: settings.hotkey,
      capsule: settings.capsule,
      floatingIcon: settings.floatingIcon,
      llm: { ...settings.llm },
    };
    // Never write before the read has landed, or the defaults would overwrite the file.
    if (isPopup || !restored) return;
    clearTimeout(saveTimer);
    saveTimer = setTimeout(() => invoke("save_settings", { settings: snapshot }).catch(() => {}), 400);
  });

  // Reminders are the kernel's clock, not this one's: it keeps ticking with the panel put
  // away and the capsule parked, where a hidden webview's timers get throttled. All it needs
  // from here is the list, as absolute instants -- it has no business knowing a timezone.
  function dueAt(plan: Plan) {
    const [y, m, d] = plan.day.split("-").map(Number);
    const hour = Math.floor(plan.start);
    return new Date(y, m - 1, d, hour, Math.round((plan.start - hour) * 60)).getTime();
  }

  let minute = $state(0);
  $effect(() => {
    const id = setInterval(() => (minute = Math.floor(Date.now() / 60000)), 30000);
    return () => clearInterval(id);
  });

  const schedule = $derived.by(() => {
    // Reading the minute is what lets a daily plan roll over to tomorrow by itself.
    void minute;
    const now = Date.now();
    return plans
      .filter((p) => !p.done)
      .map((p) => {
        let due = dueAt(p);
        if (p.repeat === "daily" && due <= now) due += 86_400_000;
        return { id: p.id, dueAt: due, title: p.title || "未命名计划", level: p.level };
      });
  });

  $effect(() => {
    if (isPopup) return;
    invoke("set_schedule", { items: schedule }).catch(() => {});
  });

  // Plans, roles and projects get the same treatment one file over. They used to be seeds
  // that re-anchored to "today" on every launch; now an empty list is a real, saved state.
  let dataReady = $state(false);
  let dataTimer: ReturnType<typeof setTimeout> | undefined;

  $effect(() => {
    if (isPopup) return;
    invoke<{ plans?: Plan[]; roles?: Role[]; projects?: Project[] }>("load_data")
      .then((saved) => {
        if (Array.isArray(saved.plans)) plans.splice(0, plans.length, ...saved.plans);
        if (Array.isArray(saved.roles)) roles.splice(0, roles.length, ...saved.roles);
        if (Array.isArray(saved.projects)) projects.splice(0, projects.length, ...saved.projects);
        // Ids restart at 1 each launch, so the counter has to clear whatever came back.
        adoptIds(saved.plans ?? [], saved.roles ?? [], saved.projects ?? []);
      })
      .catch(() => {})
      .finally(() => (dataReady = true));
  });

  $effect(() => {
    // Taking the snapshot is also what subscribes this effect to edits buried inside a plan,
    // not just to the arrays being replaced.
    const snapshot = $state.snapshot({ plans, roles, projects });
    if (isPopup || !dataReady) return;
    clearTimeout(dataTimer);
    dataTimer = setTimeout(() => invoke("save_data", { data: snapshot }).catch(() => {}), 400);
  });

  $effect(() => {
    if (isPopup) return;
    // The startup event races the listener, so ask for it once as well.
    invoke<string>("active_hotkey")
      .then((held) => (settings.hotkey = held))
      .catch(() => {});

    const stop: Array<() => void> = [];
    let disposed = false;

    listen<string>("quorum://mode", (event) => {
      mode = event.payload as Mode;
    })
      .then((un) => (disposed ? un() : stop.push(un)))
      .catch((e) => console.error("mode listener failed", e));

    listen<string>("quorum://hotkey", (event) => {
      settings.hotkey = event.payload ?? "";
    })
      .then((un) => (disposed ? un() : stop.push(un)))
      .catch(() => {});

    // The tray item flips the same preference the settings page owns.
    listen<boolean>("quorum://floating", (event) => {
      settings.floatingIcon = event.payload ?? true;
    })
      .then((un) => (disposed ? un() : stop.push(un)))
      .catch(() => {});

    listen<{ id: string; action: string }>("quorum://remind-answered", (event) => {
      const answer = event.payload;
      const plan = answer && plans.find((p) => p.id === answer.id);
      if (!plan) return;
      if (answer.action === "done") plan.done = true;
      // Snoozing moves the plan itself, so the new time is the one the list and the capsule
      // show -- and the kernel re-reads it as a fresh, unfired slot.
      if (answer.action === "snooze") {
        plan.start += 10 / 60;
        plan.end += 10 / 60;
      }
    })
      .then((un) => (disposed ? un() : stop.push(un)))
      .catch(() => {});

    listen("quorum://focus-lost", () => {
      if (mode === "panel") restore();
    })
      .then((un) => (disposed ? un() : stop.push(un)))
      .catch((e) => console.error("focus listener failed", e));

    return () => {
      disposed = true;
      stop.forEach((un) => un());
    };
  });
</script>

<svelte:window onkeydown={onKeydown} />

{#if isPopup}
  <Popup />
{:else}
  <div
    class="root"
    data-mode={mode}
    style="--capsule:{settings.capsule}"
    onmouseenter={onEnter}
    onmouseleave={onLeave}
    role="presentation"
  >
    <div class="pane" class:away={mode !== "panel"}>
      <Panel
        {route}
        {tab}
        {canBack}
        active={mode === "panel"}
        onopen={open}
        onswitch={(next) => (tab = next)}
        {back}
      />
    </div>

    {#if mode !== "panel"}
      <Capsule
        edge={mode === "edge"}
        onopen={(next) => {
          if (next) open(next);
          setMode("panel");
        }}
      />
    {/if}
  </div>
{/if}

<style>
  .root {
    height: 100%;
    display: flex;
  }

  /* Hidden, not unmounted: putting the window away must not throw away a half-written
     line in the composer, and losing focus is how the panel gets put away. */
  .pane {
    display: flex;
    flex: 1;
    min-width: 0;
    min-height: 0;
  }

  .pane.away {
    display: none;
  }

  /* Floating: a real capsule -- both ends round, clear of any edge.
     No box-shadow: the window is exactly the size of the pill, so a shadow can only paint
     inside the four corners -- which on a transparent window reads as a grey haze. */
  .root[data-mode="capsule"] {
    align-items: stretch;
    border-radius: var(--r-capsule);
    border: 1px solid var(--line-strong);
    background: var(--capsule);
  }

  /* Parked: the same pill, docked against the edge. A percentage radius would turn a
     148x44 box into an ellipse and spill the contents outside it. */
  .root[data-mode="edge"] {
    align-items: stretch;
    border-radius: var(--r-capsule);
    border: 1px solid var(--line-strong);
    background: var(--capsule);
  }

  /* Same reason as the capsule: the window is the panel, so its shadow can only land in
     the four corner notches. */
  .root[data-mode="panel"] {
    border-radius: var(--r-panel);
    border: 1px solid var(--line-strong);
    background: var(--bg);
    overflow: hidden;
  }
</style>
