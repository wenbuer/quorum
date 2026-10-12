export type Level = "L1" | "L2" | "L3";
export type Repeat = "once" | "daily";
export type Domain = "plan" | "role" | "project" | "settings";

/// What a plan does when it comes due. `run` is the hook the roles will execute -- nothing
/// runs it yet, but the field has to exist before the task store locks its shape.
export type PlanAction = { kind: "remind" | "run"; prompt: string };

/** Plans own the schedule and the notification: title, note, when, how loud, and what to do. */
export type Plan = {
  id: string;
  day: string;
  start: number;
  end: number;
  title: string;
  note: string;
  role: string;
  level: Level;
  repeat: Repeat;
  done: boolean;
  action: PlanAction;
};

/** A role owns its identity and what it is allowed to touch. */
export type Grant = { id: string; name: string; scope: string; on: boolean };

export type Role = { id: string; name: string; grants: Grant[] };

/** A project owns its files: a name, a note, and the folders it covers. */
export type Project = { id: string; name: string; note: string; paths: string[] };

export type Route =
  | { name: "planner" }
  | { name: "plan"; id: string }
  | { name: "roles" }
  | { name: "projects" }
  | { name: "project"; id: string }
  | { name: "project-new" }
  | { name: "plan-new" }
  | { name: "settings" };

let seq = 0;
const nextId = (prefix: string) => prefix + "-" + ++seq;

/// Ids restart at 1 every launch, so whatever came back from disk has to push the counter
/// past itself first -- otherwise the next new plan is handed an id that is already taken.
export function adoptIds(...groups: { id: string }[][]) {
  for (const group of groups) {
    for (const item of group) {
      const n = Number(item.id.split("-").pop());
      if (Number.isFinite(n) && n > seq) seq = n;
    }
  }
}

export const WEEKDAYS = ["周一", "周二", "周三", "周四", "周五", "周六", "周日"];

/// Dates are keys, not weekday slots: a calendar that can leave this week needs one.
export function keyOf(date: Date) {
  const m = String(date.getMonth() + 1).padStart(2, "0");
  const d = String(date.getDate()).padStart(2, "0");
  return date.getFullYear() + "-" + m + "-" + d;
}

export function parseKey(key: string) {
  const [y, m, d] = key.split("-").map(Number);
  return new Date(y, m - 1, d);
}

export const todayKey = keyOf(new Date());

/// Plans keep time as a decimal hour (9.5); people read "09:30".
export function clock(hour: number) {
  const h = Math.floor(hour);
  const m = Math.round((hour - h) * 60);
  return String(h).padStart(2, "0") + ":" + String(m).padStart(2, "0");
}

export const view = $state({ day: todayKey });

/// Light tints only: the capsule keeps its dark text, so anything that would need a
/// second text colour is out of scope here.
export const CAPSULE_TONES = [
  { name: "天青", value: "#e7f7fa" },
  { name: "薄荷", value: "#e4f6ed" },
  { name: "丁香", value: "#eae9fb" },
  { name: "米杏", value: "#fcf0e2" },
  { name: "藕荷", value: "#fbe9f0" },
  { name: "雾灰", value: "#eef0f4" },
];

export type Llm = {
  provider: string;
  baseUrl: string;
  model: string;
  apiKey: string;
  /// Off means every note is parsed by the offline regex, which costs nothing and needs no
  /// network. On is the default: the field only exists so a metered key can be spared.
  parseNotes: boolean;
  /// The last word the endpoint gave us: "" never asked, "ok" answered, "failed" did not.
  /// Written by the connection test and by every note the parser sends, so the panel's pill
  /// stays honest without polling anything.
  status: "ok" | "failed" | "";
};

/// OpenAI-compatible endpoints only: one request shape covers the cloud ones and the
/// local runtimes, so nothing here is provider-specific yet.
export const LLM_PRESETS = [
  { id: "openai", name: "OpenAI", baseUrl: "https://api.openai.com/v1", model: "gpt-4o-mini" },
  { id: "deepseek", name: "DeepSeek", baseUrl: "https://api.deepseek.com/v1", model: "deepseek-chat" },
  {
    id: "qwen",
    name: "通义千问",
    baseUrl: "https://dashscope.aliyuncs.com/compatible-mode/v1",
    model: "qwen-plus",
  },
  { id: "ollama", name: "Ollama 本机", baseUrl: "http://127.0.0.1:11434/v1", model: "qwen2.5:7b" },
  { id: "custom", name: "自定义", baseUrl: "", model: "" },
];

export const settings = $state({
  floatingIcon: true,
  hotkey: "",
  capsule: "#e7f7fa",
  llm: {
    provider: "openai",
    baseUrl: "https://api.openai.com/v1",
    model: "gpt-4o-mini",
    apiKey: "",
    parseNotes: true,
    status: "",
  } as Llm,
});

export function domainOf(route: Route): Domain {
  switch (route.name) {
    case "planner":
    case "plan":
    case "plan-new":
      return "plan";
    case "projects":
    case "project":
    case "project-new":
      return "project";
    case "settings":
      return "settings";
    default:
      return "role";
  }
}

/** Overlay state that is not a route: the editor slides over the list it belongs to. */
export const ui = $state<{
  drawer: null | { kind: "role"; id?: string; name?: string };
  projectDraft: null | { name: string; path: string };
  /// Same one-shot trick as projectDraft: a command fills it, the create form reads it once.
  planDraft: null | Partial<Omit<Plan, "id">>;
  /// The settings page's 大模型 drawer. It lives here rather than in the page so Escape can
  /// close the drawer instead of the whole panel.
  llmOpen: boolean;
}>({ drawer: null, projectDraft: null, planDraft: null, llmOpen: false });

export function rootOf(domain: Domain): Route {
  switch (domain) {
    case "plan":
      return { name: "planner" };
    case "project":
      return { name: "projects" };
    case "settings":
      return { name: "settings" };
    default:
      return { name: "roles" };
  }
}

export function sameRoute(a: Route, b: Route) {
  if (a.name !== b.name) return false;
  if (a.name === "plan" && b.name === "plan") return a.id === b.id;
  if (a.name === "project" && b.name === "project") return a.id === b.id;
  return true;
}

export function grantsTemplate(): Grant[] {
  return [
    { id: "calendar", name: "日历读取", scope: "只读 · 当前账号", on: false },
    { id: "plans", name: "计划写入", scope: "只在这个角色的文件夹里", on: false },
    { id: "files", name: "文件读写", scope: "只在这个角色的文件夹里", on: false },
    { id: "screen", name: "屏幕截图", scope: "在本机识别文字", on: false },
    { id: "browser", name: "浏览器控制", scope: "另开一个浏览器，不碰你正在用的", on: false },
    { id: "shell", name: "执行脚本", scope: "只能运行角色自带的脚本", on: false },
  ];
}

const withOn = (ids: string[]) =>
  grantsTemplate().map((g) => (ids.includes(g.id) ? { ...g, on: true } : g));

/// These three are filled from data.json at launch and written back when they change; an
/// empty list is a real state, not a missing one. 编排 stays built in because a plan written
/// without picking a role is filed under that name.
export const roles = $state<Role[]>([
  { id: "planner", name: "编排", grants: withOn(["calendar", "plans"]) },
]);

export const projects = $state<Project[]>([]);

export const plans = $state<Plan[]>([]);

/// A plan handed to the create form with nothing filled but the time: the next half hour if
/// the day it lands on is today, otherwise a sober 09:00.
export function blankPlan(day: string): Omit<Plan, "id"> {
  const now = new Date();
  const half = Math.ceil((now.getHours() + now.getMinutes() / 60) * 2) / 2;
  const start = day === todayKey ? half : 9;
  return {
    day,
    start,
    end: start + 0.5,
    title: "",
    note: "",
    role: roles[0]?.name ?? "编排",
    level: "L1",
    repeat: "once",
    done: false,
    action: { kind: "remind", prompt: "" },
  };
}

/// Plans written before `action` existed only ever reminded, so that is what they get.
export function withAction(plan: Plan): Plan {
  return { ...plan, action: plan.action ?? { kind: "remind", prompt: "" } };
}

export function addPlan(plan: Omit<Plan, "id">) {
  const created = { ...plan, id: nextId("p") };
  plans.push(created);
  return created;
}

export function removePlan(id: string) {
  const at = plans.findIndex((p) => p.id === id);
  if (at >= 0) plans.splice(at, 1);
}

export function toggleDone(id: string) {
  const plan = plans.find((p) => p.id === id);
  if (plan) plan.done = !plan.done;
}

export function addRole(name: string, grants: Grant[]) {
  const created: Role = { id: nextId("r"), name, grants };
  roles.push(created);
  return created;
}

export function updateRole(id: string, name: string, grants: Grant[]) {
  const role = roles.find((r) => r.id === id);
  if (!role) return;
  role.name = name;
  role.grants = grants;
}

export function removeRole(id: string) {
  const at = roles.findIndex((r) => r.id === id);
  if (at >= 0) roles.splice(at, 1);
}

export function addProject(name: string, note: string, paths: string[]) {
  const created: Project = { id: nextId("pr"), name, note, paths };
  projects.push(created);
  return created;
}

export function updateProject(id: string, name: string, note: string, paths: string[]) {
  const project = projects.find((p) => p.id === id);
  if (!project) return;
  project.name = name;
  project.note = note;
  project.paths = paths;
}

export function removeProject(id: string) {
  const at = projects.findIndex((p) => p.id === id);
  if (at >= 0) projects.splice(at, 1);
}
