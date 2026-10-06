export type Level = "L1" | "L2" | "L3";
export type Repeat = "once" | "daily";
export type Domain = "plan" | "role" | "project" | "settings";

/** Plans own the schedule and the notification: title, note, when, how loud. */
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
  | { name: "settings" };

let seq = 0;
const nextId = (prefix: string) => prefix + "-" + ++seq;

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

const on = (offset: number) => {
  const date = new Date();
  date.setDate(date.getDate() + offset);
  return keyOf(date);
};

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

export const settings = $state({ floatingIcon: true, hotkey: "", capsule: "#e7f7fa" });

export function domainOf(route: Route): Domain {
  switch (route.name) {
    case "planner":
    case "plan":
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
}>({ drawer: null, projectDraft: null });

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

export const roles = $state<Role[]>([
  { id: "planner", name: "编排", grants: withOn(["calendar", "plans"]) },
  { id: "sentinel", name: "哨兵", grants: withOn(["files"]) },
  { id: "archivist", name: "归档", grants: withOn(["files", "plans"]) },
]);

export const projects = $state<Project[]>([
  { id: "aail", name: "AAIL", note: "桌面助手本体", paths: ["G:\\Projects\\aail"] },
  { id: "thesis", name: "毕业论文", note: "", paths: [] },
  { id: "blog", name: "博客重构", note: "", paths: ["G:\\blog"] },
]);

// Demo data: spread today's plans over whatever is left of the day, so the capsule always
// has something real to count down to instead of a hardcoded morning that has already gone.
const hoursNow = new Date().getHours() + new Date().getMinutes() / 60;
const room = Math.max(2, 22.5 - hoursNow);
const at = (share: number) => Math.round((hoursNow + room * share) * 100) / 100;
const soon = at(0.15);
const mid = at(0.45);
const late = at(0.8);
const earlier = Math.max(0.3, Math.round((hoursNow - 1.6) * 100) / 100);

export const plans = $state<Plan[]>([
  { id: nextId("p"), day: on(0), start: earlier, end: earlier + 0.6, title: "开盘前复盘", note: "", role: "编排", level: "L1", repeat: "daily", done: true },
  { id: nextId("p"), day: on(0), start: soon, end: soon + 0.7, title: "API 方案评审", note: "过一遍接口草案", role: "编排", level: "L3", repeat: "once", done: false },
  { id: nextId("p"), day: on(0), start: mid, end: mid + 0.9, title: "竞品拆解 · 复盘", note: "", role: "归档", level: "L2", repeat: "once", done: false },
  { id: nextId("p"), day: on(0), start: late, end: late + 0.8, title: "周报提纲", note: "", role: "编排", level: "L1", repeat: "daily", done: false },
  { id: nextId("p"), day: on(3), start: 10, end: 10.5, title: "季度对账", note: "", role: "归档", level: "L2", repeat: "once", done: false },
  { id: nextId("p"), day: on(5), start: 14, end: 14.5, title: "供应商沟通", note: "", role: "哨兵", level: "L1", repeat: "once", done: false },
]);

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
