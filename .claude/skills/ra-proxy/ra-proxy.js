#!/usr/bin/env node
// ra-proxy: an LSP proxy between Claude Code (client) and rust-analyzer (server).
//
// Why it exists: Claude Code's LSP client, when connected directly to
// rust-analyzer on a heavy-proc-macro (bevy) workspace, drives ra into a 96%
// CPU death-loop during cache priming. A minimal client that declares normal
// (IDE-like) capabilities and stays SILENT during priming lets ra finish
// indexing cleanly (proven by /tmp/ra-proxy-trial.js). This proxy replicates
// that: it presents IDE-like capabilities to ra, gates Claude's requests
// until priming completes, and logs EVERY message so we can see exactly what
// Claude sends.
//
// Message log: ~/.claude/skills/ra-proxy/proxy.log
//   [ts] DIR  METHOD  {json}
//   DIR: C->R (claude to ra) | R->C (ra to claude) | PROXY (internal note)

"use strict";

const { spawn } = require("child_process");
const fs = require("fs");
const path = require("path");

// Locate the project root from this script's location (walk up to Cargo.toml),
// so the plugin is portable and not hardcoded to an absolute path.
function findRoot(start) {
  let dir = start;
  for (let i = 0; i < 8; i++) {
    if (fs.existsSync(path.join(dir, "Cargo.toml"))) return dir;
    const parent = path.dirname(dir);
    if (parent === dir) break;
    dir = parent;
  }
  return start;
}
const PLUGIN_DIR = __dirname;
const ROOT = findRoot(PLUGIN_DIR);
// rust-analyzer binary: prefer RA_BIN env, then ~/.cargo/bin/rust-analyzer (rustup shim).
const RA_BIN = process.env.RA_BIN || path.join(process.env.HOME || "/Users/zhr", ".cargo/bin/rust-analyzer");
const LOG = path.join(PLUGIN_DIR, "proxy.log");

const logStream = fs.createWriteStream(LOG, { flags: "w" });
function log(dir, method, obj) {
  const ts = new Date().toISOString().slice(11, 23);
  let s;
  try { s = JSON.stringify(obj); } catch { s = String(obj); }
  if (s && s.length > 800) s = s.slice(0, 800) + "...<truncated>";
  logStream.write(`${ts} ${dir} ${method || ""} ${s || ""}\n`);
}
log("PROXY", "startup", { ra: RA_BIN, root: ROOT });

// --- LSP framing helpers (work on a Buffer-accumulating stream) ---
function makeReader(onMessage) {
  let buf = Buffer.alloc(0);
  return (chunk) => {
    buf = Buffer.concat([buf, chunk]);
    while (true) {
      const headerEnd = buf.indexOf("\r\n\r\n");
      if (headerEnd === -1) break;
      const header = buf.slice(0, headerEnd).toString();
      const m = /Content-Length:\s*(\d+)/i.exec(header);
      if (!m) break;
      const len = parseInt(m[1], 10);
      const bodyStart = headerEnd + 4;
      if (buf.length < bodyStart + len) break;
      const body = buf.slice(bodyStart, bodyStart + len);
      buf = buf.slice(bodyStart + len);
      let msg;
      try { msg = JSON.parse(body.toString()); } catch { continue; }
      onMessage(msg);
    }
  };
}
function writeMsg(stream, msg) {
  const json = JSON.stringify(msg);
  stream.write(`Content-Length: ${Buffer.byteLength(json)}\r\n\r\n${json}`);
}

// --- spawn rust-analyzer ---
const ra = spawn(RA_BIN, [], { stdio: ["pipe", "pipe", "pipe"], cwd: ROOT });
ra.stderr.on("data", (c) => log("R-ERR", "stderr", c.toString().trim()));
ra.on("exit", (code, sig) => {
  log("PROXY", "ra-exit", { code, sig });
  process.exit(1);
});

// --- state ---
let claudeInit = null;          // initialize params from Claude
let raReady = false;            // has ra returned initialize result?
let primed = false;             // cachePriming complete?
const buffered = [];            // Claude messages held until primed
const idMap = new Map();        // claude req id -> ra req id (client->server requests)
const revMap = new Map();       // ra req id -> claude req id (for matching ra's responses)
// server->client requests (e.g. client/registerCapability): ra req id -> claude req id.
// ra sends a request with its own id; we forward it to Claude with a fresh id and
// remember the mapping so Claude's response can be remapped back to ra's id.
const srvReqRev = new Map();    // claude id (we assigned) -> ra id (ra assigned)
const srvReqFwd = new Map();    // ra id -> claude id
let nextClaudeId = 1;           // ids we assign to server->client requests forwarded to Claude
let nextRaId = 1;

// forward a Claude request to ra with a fresh id, mapping back on reply
function forwardRequest(msg) {
  const raId = nextRaId++;
  idMap.set(msg.id, raId);
  revMap.set(raId, msg.id);
  const out = { ...msg, id: raId };
  log("C->R", msg.method, msg.params ?? null);
  writeMsg(ra.stdin, out);
}
function forwardNotification(msg) {
  log("C->R", msg.method, msg.params ?? null);
  writeMsg(ra.stdin, msg);
}
function forwardResponse(msg) {
  // ra->claude response: remap id
  const claudeId = revMap.get(msg.id);
  if (claudeId === undefined) {
    log("PROXY", "unmapped-response", { id: msg.id });
    return;
  }
  revMap.delete(msg.id);
  idMap.delete(claudeId);
  const out = { ...msg, id: claudeId };
  log("R->C", "response", msg.result ?? msg.error ?? null);
  writeMsg(process.stdout, out);
}

// --- ra -> proxy ---
ra.stdout.on("data", makeReader((msg) => {
  if (msg.id != null && (msg.result !== undefined || msg.error !== undefined)) {
    // it's a RESPONSE (to a client->server request we forwarded)
    if (!raReady && claudeInit) {
      // this is the initialize response
      raReady = true;
      log("PROXY", "ra-initialized-reply", null);
      // send initialized notification
      writeMsg(ra.stdin, { jsonrpc: "2.0", method: "initialized", params: {} });
      log("C->R", "initialized", {});
      // NOW: tell Claude its initialize succeeded (pass through ra's capabilities)
      const initResult = msg.result;
      const claudeInitResult = {
        jsonrpc: "2.0",
        id: claudeInit.id,
        result: initResult,
      };
      log("R->C", "initialize-response", initResult);
      writeMsg(process.stdout, claudeInitResult);
      return;
    }
    forwardResponse(msg);
    return;
  }
  // SERVER->CLIENT REQUEST (ra asking Claude to do something, e.g.
  // client/registerCapability): has id + method, no result/error.
  // Forward to Claude with a fresh id and remember the mapping so we can
  // route Claude's response back to ra's original id.
  if (msg.id != null && msg.method) {
    const claudeId = nextClaudeId++;
    srvReqFwd.set(msg.id, claudeId);
    srvReqRev.set(claudeId, msg.id);
    const out = { ...msg, id: claudeId };
    log("R->C", "request " + msg.method, msg.params ?? null);
    writeMsg(process.stdout, out);
    return;
  }
  // it's a notification
  // detect priming completion via rust-analyzer serverStatus (secondary signal)
  if (msg.method === "$/rust-analyzer/serverStatus" && msg.params) {
    log("R->C", "serverStatus", msg.params);
    if (msg.params.quiescent) flushBuffered("serverStatus.quiescent");
    writeMsg(process.stdout, msg);
    return;
  }
  // other notifications (diagnostics, etc.) -> pass through
  log("R->C", msg.method, msg.params ?? null);
  writeMsg(process.stdout, msg);
}));

// --- claude -> proxy ---
process.stdin.on("data", makeReader((msg) => {
  // intercept initialize: hold it, we drive ra's initialize ourselves
  if (msg.method === "initialize") {
    claudeInit = msg;
    log("C->R", "initialize(intercepted)", msg.params);
    // send our OWN initialize to ra with IDE-like capabilities + cachePriming
    const raInit = {
      jsonrpc: "2.0",
      id: nextRaId++,
      method: "initialize",
      params: {
        processId: process.pid,
        clientInfo: { name: "ra-proxy", version: "0.1.0" },
        rootUri: "file://" + ROOT,
        rootPath: ROOT,
        workspaceFolders: [{ uri: "file://" + ROOT, name: "moon-lol" }],
        capabilities: {
          workspace: {
            configuration: true,
            workspaceFolders: true,
            didChangeWatchedFiles: { dynamicRegistration: true },
            symbol: { dynamicRegistration: false },
            applyEdit: true,
            workspaceEdit: { documentChanges: true },
          },
          textDocument: {
            synchronization: { didSave: true, willSave: false, willSaveWaitUntil: false, dynamicRegistration: false },
            hover: { contentFormat: ["markdown", "plaintext"] },
            definition: { linkSupport: true },
            references: {},
            documentSymbol: { hierarchicalDocumentSymbolSupport: true },
            publishDiagnostics: { relatedInformation: true, tagSupport: { valueSet: [1, 2] } },
            signatureHelp: { signatureInformation: { documentationFormat: ["markdown", "plaintext"] } },
            completion: { completionItem: { snippetSupport: true } },
          },
          general: { positionEncodings: ["utf-16"] },
        },
        initializationOptions: {
          checkOnSave: false,
          diagnostics: { enable: false },
          cachePriming: { enable: true, numThreads: "physical" },
          procMacro: { enable: true, attributes: { enable: true } },
          cargo: { buildScripts: { enable: true } },
        },
      },
    };
    revMap.set(raInit.id, "__init__");
    writeMsg(ra.stdin, raInit);
    return;
  }

  if (msg.method === "initialized") {
    // Claude's initialized — we already sent ours; absorb it
    log("C->R", "initialized(absorbed)", {});
    return;
  }

  // shutdown / exit
  if (msg.method === "shutdown") {
    log("C->R", "shutdown", null);
    writeMsg(ra.stdin, msg);
    return;
  }
  if (msg.method === "exit") {
    log("C->R", "exit", null);
    writeMsg(ra.stdin, msg);
    setTimeout(() => process.exit(0), 300);
    return;
  }

  // CLAUDE RESPONSE to a server->client request (e.g. registerCapability reply):
  // has id but NO method, and result/error present. Route back to ra's original id.
  // Never buffer these, never forward as a request.
  if (msg.id != null && !msg.method && (msg.result !== undefined || msg.error !== undefined)) {
    const raId = srvReqRev.get(msg.id);
    if (raId === undefined) {
      log("PROXY", "unmapped-srv-response", { id: msg.id });
      return;
    }
    srvReqRev.delete(msg.id);
    srvReqFwd.delete(raId);
    const out = { ...msg, id: raId };
    log("C->R", "response", msg.result ?? msg.error ?? null);
    writeMsg(ra.stdin, out);
    return;
  }

  // everything else: buffer until primed, then forward
  if (!primed) {
    log("PROXY", "buffer", { method: msg.method, id: msg.id });
    buffered.push(msg);
    return;
  }
  if (msg.id != null) forwardRequest(msg);
  else if (msg.method) forwardNotification(msg);
}));

// --- priming completion detection ---
// ra doesn't reliably send $/rust-analyzer/serverStatus by default, so the
// PRIMARY signal is CPU/RSS polling: once ra's CPU stays low and RSS stable,
// cache priming is done. serverStatus (if sent) is a secondary trigger.
function flushBuffered(reason) {
  if (primed) return;
  primed = true;
  log("PROXY", "PRIMING-DONE " + reason, { buffered: buffered.length });
  for (const m of buffered) {
    if (m.id != null) forwardRequest(m);
    else if (m.method) forwardNotification(m);
  }
  buffered.length = 0;
}

let lowCpuStreak = 0;
let lastRss = 0;
function startCpuProbe() {
  const iv = setInterval(() => {
    if (primed) { clearInterval(iv); return; }
    const r = require("child_process").spawnSync(
      "ps", ["-o", "%cpu=,rss=", "-p", String(ra.pid)],
      { encoding: "utf8" }
    );
    const out = (r.stdout || "").trim();
    if (!out) return;
    const [cpuStr, rssStr] = out.split(/\s+/);
    const cpu = parseFloat(cpuStr);
    const rss = parseInt(rssStr, 10);
    const rssDelta = lastRss ? Math.abs(rss - lastRss) : 0;
    lastRss = rss;
    // priming done when: CPU < 8% AND RSS barely changing, for 3 consecutive samples
    // (require RSS >= 1GB so we don't fire on the very first low-CPU blip at startup)
    if (cpu < 8 && rss >= 1_000_000 && rssDelta < 100_000) {
      lowCpuStreak++;
      log("PROXY", "cpu-probe", { cpu, rss, rssDelta, streak: lowCpuStreak });
      if (lowCpuStreak >= 3) {
        clearInterval(iv);
        flushBuffered("cpu-probe (CPU<8% & RSS stable x3)");
      }
    } else {
      lowCpuStreak = 0;
      log("PROXY", "cpu-probe", { cpu, rss, rssDelta, streak: 0 });
    }
  }, 4000);
  // hard fallback: never block longer than 6 min
  setTimeout(() => flushBuffered("hard-timeout-6min"), 6 * 60 * 1000);
}

process.stdin.resume();
log("PROXY", "proxy-ready", { note: "waiting for Claude initialize" });

// kick off CPU probe shortly after we expect initialize to have happened
setTimeout(() => { if (raReady) startCpuProbe(); }, 5000);
