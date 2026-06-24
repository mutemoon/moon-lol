// Thin REST client for the server-side endpoints introduced in Phase 3–5:
// rooms, matches, rank, essence, subscriptions, community, admin and agent snapshots.
//
// These endpoints are not part of the local-only IBackendClient surface — they
// always talk to lol_web_server (HTTPS). The auth token is shared with
// WebBackendClient via the same localStorage key.

const BASE_URL = (import.meta as any).env?.VITE_BASE_URL || "http://localhost:3000";
const TOKEN_KEY = "moon_lol_auth_token";

function token(): string | null {
  return typeof localStorage !== "undefined" ? localStorage.getItem(TOKEN_KEY) : null;
}

async function ensureToken(): Promise<string> {
  let t = token();
  if (t) return t;

  // Mirror WebBackendClient.ensureAuth() — auto-login/register with a dev account
  const phone = "13800000000";
  const password = "admin_password";
  try {
    const res = await fetch(`${BASE_URL}/api/auth/login`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ phone, password }),
    });
    if (res.ok) {
      const data = await res.json();
      t = data.data.token as string;
      localStorage.setItem(TOKEN_KEY, t);
      return t;
    }
  } catch {
    /* fallthrough to register */
  }
  const reg = await fetch(`${BASE_URL}/api/auth/register`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ phone, password, code: "111111" }),
  });
  if (!reg.ok) throw new Error(`Auth bootstrap failed: ${reg.status}`);
  const data = await reg.json();
  t = data.data.token as string;
  localStorage.setItem(TOKEN_KEY, t);
  return t;
}

async function call<T = any>(path: string, init: RequestInit = {}): Promise<T> {
  const t = await ensureToken();
  const headers = new Headers(init.headers);
  headers.set("Authorization", `Bearer ${t}`);
  if (init.body && !headers.has("Content-Type")) headers.set("Content-Type", "application/json");

  const res = await fetch(`${BASE_URL}${path}`, { ...init, headers });
  const body = await res.text();
  let parsed: any = null;
  try {
    parsed = body ? JSON.parse(body) : null;
  } catch {
    /* non-JSON */
  }
  if (!res.ok) {
    const msg = parsed?.error?.message || `HTTP ${res.status}`;
    throw new Error(msg);
  }
  return parsed?.data as T;
}

// ── Types ──

export type Visibility = "private" | "friends" | "public";
export type Team = "order" | "chaos";

export interface Agent {
  id: string;
  owner_user_id: number;
  name: string;
  champion: string;
  agent_config_id: string;
  spawn_preset_id: string | null;
  visibility: Visibility;
  forked_from_agent_id: string | null;
  created_at: string;
  updated_at: string;
}

export interface AgentSnapshot {
  id: string;
  agent_id: string;
  version: number;
  config_freeze: Record<string, any>;
  created_at: string;
}

export interface RoomConstraints {
  max_members: number;
  max_agents_per_member: number;
  team_strategy: "single" | "free";
  lobby_visible: boolean;
  reveal_prompts: boolean;
}

export interface Room {
  id: string;
  name: string;
  owner_user_id: number;
  constraints: RoomConstraints;
  invite_code: string;
  member_count: number;
  status: "pending" | "running" | "finished";
  created_at: string;
}

export interface RoomAgentSlot {
  id: string;
  room_id: string;
  member_user_id: number;
  agent_id: string;
  team: Team;
}

export interface Match {
  id: string;
  mode: string;
  status: "pending" | "running" | "finished" | "aborted";
  owner_user_id: number | null;
  room_id: string | null;
  ws_port: number | null;
  created_at: string;
  finished_at: string | null;
}

export interface MatchEvent {
  id: string;
  match_id: string;
  seq: number;
  payload: Record<string, any>;
  recorded_at: string;
}

export interface RankQueueEntry {
  user_id: number;
  agent_id: string;
  agent_snapshot_id: string;
  mode: string;
  rating: number;
  enqueued_at: string;
}

export interface EloRating {
  agent_id: string;
  agent_name: string;
  mode: string;
  rating: number;
  games_played: number;
  wins: number;
  losses: number;
  daily_delta: number;
}

export interface Season {
  id: string;
  mode: string;
  starts_at: string;
  ends_at: string | null;
}

export interface EssenceTransaction {
  id: string;
  user_id: number;
  amount: number;
  reason: string;
  created_at: string;
}

export interface BillingPlan {
  id: string;
  name: string;
  monthly_essence: number;
  agent_limit: number;
  price_cents: number;
}

export interface AdminMetrics {
  running_matches: number;
  total_memory_mb: number;
  avg_match_memory_mb: number;
  cpu_usage_percent: number;
}

// ── Agents (extended: list with filters, visibility, snapshots) ──

export const agentsApi = {
  list: () => call<Agent[]>("/api/agents"),
  get: (id: string) => call<Agent>(`/api/agents/${id}`),
  updateVisibility: (id: string, visibility: Visibility) =>
    call<void>(`/api/agents/${id}/visibility`, {
      method: "PATCH",
      body: JSON.stringify({ visibility }),
    }),
  publishSnapshot: (id: string) =>
    call<AgentSnapshot>(`/api/agents/${id}/publish`, { method: "POST" }),
  listSnapshots: (id: string) => call<AgentSnapshot[]>(`/api/agents/${id}/snapshots`),
  browseCommunity: (sort: "recent" | "popular" | "elo" = "recent", limit = 50) =>
    call<Agent[]>(`/api/agents/community?sort=${sort}&limit=${limit}`),
  fork: (id: string, newName?: string) =>
    call<Agent>(`/api/agents/${id}/fork`, {
      method: "POST",
      body: JSON.stringify({ new_name: newName ?? null }),
    }),
};

// ── Rooms ──

export const roomsApi = {
  listMine: () => call<Room[]>("/api/rooms"),
  listLobby: () => call<Room[]>("/api/rooms/lobby"),
  create: (name: string, constraints: RoomConstraints) =>
    call<Room>("/api/rooms", {
      method: "POST",
      body: JSON.stringify({ name, constraints }),
    }),
  joinByCode: (code: string) =>
    call<Room>("/api/rooms/join-by-code", {
      method: "POST",
      body: JSON.stringify({ code }),
    }),
  get: (id: string) => call<Room>(`/api/rooms/${id}`),
  dissolve: (id: string) => call<void>(`/api/rooms/${id}`, { method: "DELETE" }),
  updateConstraints: (id: string, c: RoomConstraints) =>
    call<void>(`/api/rooms/${id}`, { method: "PATCH", body: JSON.stringify(c) }),
  join: (id: string) => call<void>(`/api/rooms/${id}/join`, { method: "POST" }),
  leave: (id: string) => call<void>(`/api/rooms/${id}/leave`, { method: "POST" }),
  listSlots: (id: string) => call<RoomAgentSlot[]>(`/api/rooms/${id}/agents`),
  addSlot: (id: string, agent_id: string, team: Team) =>
    call<RoomAgentSlot>(`/api/rooms/${id}/agents`, {
      method: "POST",
      body: JSON.stringify({ agent_id, team }),
    }),
  removeSlot: (id: string, slotId: string) =>
    call<void>(`/api/rooms/${id}/agents/${slotId}`, { method: "DELETE" }),
  start: (id: string) =>
    call<{ match_id: string; ws_port: number }>(`/api/rooms/${id}/start`, {
      method: "POST",
    }),
};

// ── Matches ──

export const matchesApi = {
  listMine: () => call<Match[]>("/api/matches"),
  listByStatus: (status: string) => call<Match[]>(`/api/matches?status=${status}`),
  get: (id: string) => call<Match>(`/api/matches/${id}`),
  getEvents: (id: string, fromSeq = 0, limit = 200) =>
    call<MatchEvent[]>(`/api/matches/${id}/events?from_seq=${fromSeq}&limit=${limit}`),
  stop: (id: string) => call<void>(`/api/matches/${id}/stop`, { method: "POST" }),
};

// ── Rank ──

export const rankApi = {
  enqueue: (agent_id: string, agent_snapshot_id: string, mode: string) =>
    call<RankQueueEntry>("/api/rank/queue", {
      method: "POST",
      body: JSON.stringify({ agent_id, agent_snapshot_id, mode }),
    }),
  status: () => call<RankQueueEntry[]>("/api/rank/queue/status"),
  leaderboard: (mode = "top_solo", limit = 50) =>
    call<EloRating[]>(`/api/rank/leaderboard?mode=${mode}&limit=${limit}`),
  currentSeason: () => call<Season>("/api/rank/seasons/current"),
};

// ── Essence & Subscriptions ──

export const essenceApi = {
  balance: () => call<number>("/api/essence/balance"),
  checkIn: () =>
    call<{ already_checked_in: boolean; granted: number; balance: number }>(
      "/api/essence/check-in",
      { method: "POST" }
    ),
  transactions: (limit = 50, offset = 0) =>
    call<EssenceTransaction[]>(`/api/essence/transactions?limit=${limit}&offset=${offset}`),
};

export const subscriptionsApi = {
  current: () => call<BillingPlan>("/api/subscriptions"),
  subscribe: (plan_id: string) =>
    call<any>("/api/subscriptions", { method: "POST", body: JSON.stringify({ plan_id }) }),
  listPlans: () => call<BillingPlan[]>("/api/billing/plans"),
};

// ── Admin ──

export const adminApi = {
  metrics: () => call<AdminMetrics>("/api/admin/metrics"),
  listRunning: () => call<Match[]>("/api/admin/matches/running"),
  forceAbort: (id: string) =>
    call<void>(`/api/admin/matches/${id}/abort`, { method: "POST" }),
};

export const BASE = BASE_URL;
