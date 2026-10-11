import { settings, type Level, type Plan, type PlanAction, type Repeat } from "./store.svelte";

/** What the parser is allowed to fill in. It only overrides the fields it is sure about. */
export type NoteDraft = Partial<Omit<Plan, "id">>;

/// The model is here to turn a note into fields, so the prompt is a contract, not a
/// conversation. Everything it gets wrong is caught below and the regex result stands.
const SYSTEM = `你是"智群"的计划解析器。把用户随手写的一句话变成一条结构化计划。

只输出一个 JSON 对象，不要解释，不要 markdown 代码块，不要多余字段：
{
  "title": "去掉时间词后的标题，不超过 20 字",
  "day": "YYYY-MM-DD 格式的日期；没提到日期就 null",
  "start": "0 到 24 之间的小数小时，9.5 表示 09:30；没提到时间就 null",
  "minutes": "预计花多少分钟，整数，默认 30",
  "level": "提醒力度。L1 静默是默认；只有用户明确说'提醒我''必须''重要'才用 L2 系统通知或 L3 强提醒",
  "repeat": "once 或 daily。出现'每天''每日'才用 daily",
  "role": "角色名，必须是下面列出的之一",
  "actionKind": "remind 或 run。到点只要提醒人一眼用 remind；到点应该由角色动手做一件事（写、查、整理、发）用 run",
  "actionPrompt": "actionKind 是 run 时给角色的一句明确指令，否则空字符串"
}`;

const LEVELS = ["L1", "L2", "L3"];

export function llmReady() {
  return settings.llm.parseNotes && settings.llm.baseUrl.trim().length > 0;
}

/** The model is asked for bare JSON, but a fenced block is a common enough answer that
 *  refusing it would only cost a fallback. Take the outermost braces and try. */
function extractJson(raw: string): unknown {
  const start = raw.indexOf("{");
  const end = raw.lastIndexOf("}");
  if (start < 0 || end <= start) return null;
  try {
    return JSON.parse(raw.slice(start, end + 1));
  } catch {
    return null;
  }
}

function number(raw: unknown, lo: number, hi: number): number | null {
  if (raw === null || raw === undefined || raw === "") return null;
  const value = Number(raw);
  return Number.isFinite(value) ? Math.min(hi, Math.max(lo, value)) : null;
}

function toDraft(value: unknown, roleNames: string[]): NoteDraft | null {
  if (!value || typeof value !== "object") return null;
  const raw = value as Record<string, unknown>;
  const draft: NoteDraft = {};

  if (typeof raw.title === "string" && raw.title.trim()) draft.title = raw.title.trim().slice(0, 60);
  if (typeof raw.day === "string" && /^\d{4}-\d{2}-\d{2}$/.test(raw.day)) draft.day = raw.day;

  const minutes = number(raw.minutes, 5, 480) ?? 30;
  const start = number(raw.start, 0, 23.99);
  if (start !== null) {
    draft.start = start;
    draft.end = start + minutes / 60;
  }

  if (typeof raw.level === "string" && LEVELS.includes(raw.level)) draft.level = raw.level as Level;
  if (raw.repeat === "daily" || raw.repeat === "once") draft.repeat = raw.repeat as Repeat;
  if (typeof raw.role === "string" && roleNames.includes(raw.role)) draft.role = raw.role;

  const prompt = typeof raw.actionPrompt === "string" ? raw.actionPrompt.trim() : "";
  const action: PlanAction =
    raw.actionKind === "run" && prompt ? { kind: "run", prompt } : { kind: "remind", prompt: "" };
  draft.action = action;

  // A reply that carries neither a title nor a time has nothing worth putting on the card.
  return draft.title || draft.start !== undefined ? draft : null;
}

/// Returns null whenever the model cannot be reached or answers something unusable -- the
/// caller keeps whatever the offline regex produced, so a bad network is not a broken feature.
export async function parseNote(
  text: string,
  roleNames: string[],
  today: string,
): Promise<NoteDraft | null> {
  const base = settings.llm.baseUrl.replace(/\/+$/, "");
  if (!base || !text.trim()) return null;

  const res = await fetch(base + "/chat/completions", {
    method: "POST",
    headers: {
      "content-type": "application/json",
      ...(settings.llm.apiKey ? { authorization: "Bearer " + settings.llm.apiKey } : {}),
    },
    body: JSON.stringify({
      model: settings.llm.model,
      temperature: 0,
      messages: [
        { role: "system", content: SYSTEM },
        {
          role: "user",
          content: `今天是 ${today}。可用角色：${roleNames.join("、") || "编排"}。\n用户写的是：${text}`,
        },
      ],
    }),
    signal: AbortSignal.timeout(15000),
  });
  if (!res.ok) throw new Error("llm " + res.status);

  const payload = await res.json();
  const answer = payload?.choices?.[0]?.message?.content;
  if (typeof answer !== "string") return null;
  return toDraft(extractJson(answer), roleNames);
}
