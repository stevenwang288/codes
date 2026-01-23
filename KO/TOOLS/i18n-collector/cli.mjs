#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import childProcess from 'node:child_process';

function fail(msg) {
  console.error(msg);
  process.exit(1);
}

function parseArgs(argv) {
  const args = { _: [] };
  for (let i = 0; i < argv.length; i++) {
    const cur = argv[i];
    if (!cur.startsWith('--')) {
      args._.push(cur);
      continue;
    }
    const key = cur.slice(2);
    const next = argv[i + 1];
    if (next == null || next.startsWith('--')) {
      args[key] = true;
      continue;
    }
    args[key] = next;
    i++;
  }
  return args;
}

function readJson(filePath) {
  const raw = fs.readFileSync(filePath, 'utf8');
  try {
    return JSON.parse(raw);
  } catch (e) {
    fail(`Invalid JSON: ${filePath}: ${e}`);
  }
}

function writeJson(filePath, obj) {
  const dir = path.dirname(filePath);
  fs.mkdirSync(dir, { recursive: true });
  fs.writeFileSync(filePath, JSON.stringify(obj, null, 2) + '\n', 'utf8');
}

function* readJsonLines(filePath) {
  if (!fs.existsSync(filePath)) {
    return;
  }
  const raw = fs.readFileSync(filePath, 'utf8');
  const lines = raw.split(/\r?\n/);
  for (const line of lines) {
    const trimmed = line.trim();
    if (!trimmed) continue;
    try {
      yield JSON.parse(trimmed);
    } catch {
      // Ignore malformed line; this is a collector file.
    }
  }
}

function normalizeRecord(rec) {
  const key = typeof rec?.key === 'string' ? rec.key : null;
  if (!key) return null;
  const missingIn = typeof rec?.missing_in === 'string' ? rec.missing_in : 'unknown';
  const app = typeof rec?.app === 'string' ? rec.app : null;
  const fallbackText = typeof rec?.fallback_text === 'string' ? rec.fallback_text : null;
  const event = typeof rec?.event === 'string' ? rec.event : null;
  const locale = typeof rec?.locale === 'string' ? rec.locale : null;
  return { key, missingIn, app, fallbackText, event, locale };
}

function safeRegex(pattern) {
  try {
    return new RegExp(pattern);
  } catch {
    return null;
  }
}

function loadTypeRules(rulesPath) {
  if (!rulesPath) return null;
  const raw = readJson(rulesPath);
  const rules = Array.isArray(raw?.rules) ? raw.rules : null;
  if (!rules) {
    fail(`Invalid rules file (expected { rules: [...] }): ${rulesPath}`);
  }
  const compiled = [];
  for (const rule of rules) {
    const type = typeof rule?.type === 'string' ? rule.type : null;
    if (!type) continue;
    const keyPrefixes = Array.isArray(rule?.key_prefixes)
      ? rule.key_prefixes.filter(x => typeof x === 'string' && x.trim())
      : [];
    const appRegex = typeof rule?.app_regex === 'string' ? safeRegex(rule.app_regex) : null;
    compiled.push({ type, keyPrefixes, appRegex });
  }
  return compiled;
}

function detectTypeWithRules({ key, app }, compiledRules) {
  if (!compiledRules) return null;
  for (const rule of compiledRules) {
    if (rule.keyPrefixes.length > 0) {
      if (rule.keyPrefixes.some(p => key.startsWith(p))) return rule.type;
    }
    if (rule.appRegex && typeof app === 'string') {
      if (rule.appRegex.test(app)) return rule.type;
    }
  }
  return null;
}

function detectTypeFallback({ key, app }) {
  if (key.startsWith('tui.')) return 'tui';
  if (key.startsWith('cli.')) return 'cli';
  if (key.startsWith('docs.') || key.startsWith('site.') || key.startsWith('web.')) return 'site';
  if (typeof app === 'string') {
    const a = app.toLowerCase();
    if (a.includes('tui')) return 'tui';
    if (a.includes('cli')) return 'cli';
  }
  return 'unknown';
}

function detectType(rec, compiledRules) {
  const fromRules = detectTypeWithRules(rec, compiledRules);
  return fromRules || detectTypeFallback(rec);
}

function inc(map, key, n = 1) {
  map.set(key, (map.get(key) || 0) + n);
}

function topEntries(map, top) {
  const arr = Array.from(map.entries()).sort((a, b) => b[1] - a[1]);
  return typeof top === 'number' && Number.isFinite(top) && top > 0 ? arr.slice(0, top) : arr;
}

function cmdExtract(args) {
  const logPath = args.log;
  const enPath = args.en;
  const outPath = args.out;
  if (!logPath) fail('extract requires --log <path>');
  if (!enPath) fail('extract requires --en <path-to-en.json>');
  if (!outPath) fail('extract requires --out <path>');

  const en = readJson(enPath);
  if (typeof en !== 'object' || en == null || Array.isArray(en)) {
    fail(`Expected flat JSON object: ${enPath}`);
  }

  const todo = {};
  for (const rec of readJsonLines(logPath)) {
    const n = normalizeRecord(rec);
    if (!n) continue;
    if (n.missingIn !== 'zh-CN') continue;
    const base = en[n.key];
    const fallback = n.fallbackText;
    const text = typeof base === 'string' ? base : (typeof fallback === 'string' ? fallback : null);
    if (typeof text === 'string' && !(n.key in todo)) {
      todo[n.key] = text;
    }
  }

  writeJson(outPath, todo);
  console.log(`extract: wrote ${Object.keys(todo).length} keys -> ${outPath}`);
}

function buildTodoFromLog({ logPath, enPath, zhPath }) {
  const en = readJson(enPath);
  if (typeof en !== 'object' || en == null || Array.isArray(en)) {
    fail(`Expected flat JSON object: ${enPath}`);
  }

  const zh = readJson(zhPath);
  if (typeof zh !== 'object' || zh == null || Array.isArray(zh)) {
    fail(`Expected flat JSON object: ${zhPath}`);
  }

  const todo = {};
  for (const rec of readJsonLines(logPath)) {
    const n = normalizeRecord(rec);
    if (!n) continue;
    // Only treat explicit missing events as translation tasks.
    if (n.event && n.event !== 'missing') continue;
    if (n.missingIn !== 'zh-CN') continue;
    if (typeof zh[n.key] === 'string' && zh[n.key].trim()) continue;

    const base = en[n.key];
    const fallback = n.fallbackText;
    const text = typeof base === 'string' ? base : (typeof fallback === 'string' ? fallback : null);
    if (typeof text === 'string') {
      todo[n.key] = text;
    }
  }

  const sorted = {};
  for (const key of Object.keys(todo).sort()) {
    sorted[key] = todo[key];
  }
  return sorted;
}

function cmdStats(args) {
  const logPath = args.log;
  const enPath = args.en;
  const zhPath = args.zh;
  const json = args.json === true;
  const groupBy = args['group-by'] || 'type';
  const top = args.top ? Number(args.top) : 20;
  const rulesPath = args.rules || null;

  if (!logPath) fail('stats requires --log <path>');
  if (!enPath) fail('stats requires --en <path-to-en.json>');
  if (!zhPath) fail('stats requires --zh <path-to-zh-CN.json>');

  const compiledRules = loadTypeRules(rulesPath);

  let total = 0;
  const missingZh = new Set();
  const missingEn = new Set();
  const seenKeys = new Set();
  const byType = new Map();
  const byApp = new Map();
  for (const rec of readJsonLines(logPath)) {
    total++;
    const n = normalizeRecord(rec);
    if (!n) continue;
    const t = detectType(n, compiledRules);
    inc(byType, t);
    inc(byApp, n.app || 'unknown');
    if (n.event === 'seen') {
      seenKeys.add(n.key);
      continue;
    }
    if (n.missingIn === 'zh-CN') missingZh.add(n.key);
    if (n.missingIn === 'en') missingEn.add(n.key);
  }

  const pending = buildTodoFromLog({ logPath, enPath, zhPath });
  const payload = {
    log: logPath,
    rules: rulesPath,
    total_records: total,
    unique_missing_zh_cn: missingZh.size,
    unique_missing_en: missingEn.size,
    unique_seen: seenKeys.size,
    pending_zh_cn: Object.keys(pending).length,
    by_type: Object.fromEntries(byType.entries()),
    by_app: Object.fromEntries(topEntries(byApp, top)),
  };

  if (json) {
    console.log(JSON.stringify(payload, null, 2));
    return;
  }

  console.log('i18n-collector stats');
  console.log(`  log: ${payload.log}`);
  if (payload.rules) console.log(`  rules: ${payload.rules}`);
  console.log(`  total records: ${payload.total_records}`);
  console.log(`  unique missing zh-CN: ${payload.unique_missing_zh_cn}`);
  console.log(`  unique missing en: ${payload.unique_missing_en}`);
  console.log(`  unique seen keys: ${payload.unique_seen}`);
  console.log(`  pending zh-CN (needs translation): ${payload.pending_zh_cn}`);

  if (groupBy === 'none') return;
  if (groupBy === 'type') {
    console.log('  by type:');
    for (const [k, v] of topEntries(byType, 0)) {
      console.log(`    ${k}: ${v}`);
    }
    return;
  }
  if (groupBy === 'app') {
    console.log(`  by app (top ${top}):`);
    for (const [k, v] of topEntries(byApp, top)) {
      console.log(`    ${k}: ${v}`);
    }
    return;
  }
  fail(`Unknown --group-by: ${groupBy} (expected type|app|none)`);
}

function buildSchema() {
  return {
    type: 'object',
    properties: {
      translations: {
        type: 'object',
        additionalProperties: { type: 'string' },
      },
    },
    required: ['translations'],
    additionalProperties: false,
  };
}

function cmdTranslate(args) {
  const inPath = args.in;
  const outPath = args.out;
  const runner = args.runner || 'coder';
  const model = args.model || 'code-gpt-5.2-codex';
  const style = args.style || 'zh-only';
  const maxSeconds = args['max-seconds'] ? Number(args['max-seconds']) : 600;

  if (!inPath) fail('translate requires --in <todo.json>');
  if (!outPath) fail('translate requires --out <translated.json>');
  if (!Number.isFinite(maxSeconds) || maxSeconds <= 0) fail('translate --max-seconds must be a positive number');

  const todo = readJson(inPath);
  if (typeof todo !== 'object' || todo == null || Array.isArray(todo)) {
    fail(`Expected flat JSON object: ${inPath}`);
  }

  const schemaPath = args.schema || path.join(path.dirname(outPath), 'i18n-translate.schema.json');
  const rawOutPath = path.join(path.dirname(outPath), 'i18n-translate.raw.json');
  writeJson(schemaPath, buildSchema());

  const keys = Object.keys(todo);
  const prompt = [
    'You are a localization assistant.',
    'Task: translate the provided map of i18n keys -> English strings into Simplified Chinese (zh-CN).',
    'Output MUST be valid JSON that matches the provided JSON schema (object with {"translations":{...}}).',
    '',
    'Rules (critical):',
    '- Do NOT translate command names, subcommands, flags, environment variable names, file paths, URLs, model names, tool names, error codes, JSON/protocol field names.',
    '- Preserve any placeholders exactly, including braces segments like {name} or {path}. Do not add/remove placeholders.',
    '- Preserve inline code in backticks `like_this` without translating the code tokens.',
    '',
    'Style preference:',
    style === 'bilingual-tui'
      ? '- Default: Chinese only. For keys starting with "tui.", you MAY use bilingual form "中文（English）" when the English is short and space likely allows; otherwise Chinese only.'
      : '- Chinese only.',
    '',
    'Return translations for ALL keys. If you are unsure, still provide a reasonable Chinese translation.',
    '',
    'Input JSON (key -> English):',
    JSON.stringify(todo),
    '',
  ].join('\n');

  const execArgs = [
    'exec',
    '-',
    '--model',
    model,
    '--output-schema',
    schemaPath,
    '--output-last-message',
    rawOutPath,
    '--max-seconds',
    String(maxSeconds),
  ];

  const res = childProcess.spawnSync(runner, execArgs, {
    input: prompt,
    encoding: 'utf8',
    stdio: ['pipe', 'inherit', 'inherit'],
    env: { ...process.env },
  });
  if (res.error) {
    fail(`Failed to run ${runner}: ${res.error}`);
  }
  if (res.status !== 0) {
    fail(`${runner} exited with code ${res.status}`);
  }

  const raw = readJson(rawOutPath);
  const translations = raw?.translations;
  if (typeof translations !== 'object' || translations == null || Array.isArray(translations)) {
    fail(`Translator output missing translations object: ${rawOutPath}`);
  }

  writeJson(outPath, translations);
  console.log(`translate: wrote ${Object.keys(translations).length} keys -> ${outPath}`);
}

function cmdApply(args) {
  const inPath = args.in;
  const zhPath = args.zh;
  if (!inPath) fail('apply requires --in <translated-json>');
  if (!zhPath) fail('apply requires --zh <path-to-zh-CN.json>');

  const patch = readJson(inPath);
  if (typeof patch !== 'object' || patch == null || Array.isArray(patch)) {
    fail(`Expected flat JSON object: ${inPath}`);
  }

  const zh = readJson(zhPath);
  if (typeof zh !== 'object' || zh == null || Array.isArray(zh)) {
    fail(`Expected flat JSON object: ${zhPath}`);
  }

  let applied = 0;
  for (const [k, v] of Object.entries(patch)) {
    if (typeof v !== 'string') continue;
    if (!v.trim()) continue;
    if (zh[k] === v) continue;
    zh[k] = v;
    applied++;
  }

  writeJson(zhPath, zh);
  console.log(`apply: updated ${applied} keys -> ${zhPath}`);
}

function cmdSync(args) {
  const logPath = args.log;
  const enPath = args.en;
  const zhPath = args.zh;
  const runner = args.runner || 'coder';
  const model = args.model || 'code-gpt-5.2-codex';
  const style = args.style || 'zh-only';
  const maxSeconds = args['max-seconds'] ? Number(args['max-seconds']) : 600;
  const watch = args.watch === true;
  const once = args.once === true;
  const debounceMs = args['debounce-ms'] ? Number(args['debounce-ms']) : 750;

  if (!logPath) fail('sync requires --log <path>');
  if (!enPath) fail('sync requires --en <path-to-en.json>');
  if (!zhPath) fail('sync requires --zh <path-to-zh-CN.json>');
  if (!Number.isFinite(maxSeconds) || maxSeconds <= 0) fail('sync --max-seconds must be a positive number');
  if (args['debounce-ms'] && (!Number.isFinite(debounceMs) || debounceMs < 0)) {
    fail('sync --debounce-ms must be >= 0');
  }

  const outDir = args['out-dir'] || path.dirname(logPath);
  const todoPath = path.join(outDir, 'i18n-todo.zh-CN.json');
  const patchPath = path.join(outDir, 'i18n-zh-CN.patch.json');

  function syncOnce() {
    if (!fs.existsSync(logPath)) {
      console.log(`sync: log not found yet -> ${logPath}`);
      return;
    }

    const todo = buildTodoFromLog({ logPath, enPath, zhPath });
    const todoCount = Object.keys(todo).length;
    if (todoCount === 0) {
      console.log('sync: no pending zh-CN keys');
      return;
    }

    writeJson(todoPath, todo);
    console.log(`sync: extracted ${todoCount} keys -> ${todoPath}`);

    cmdTranslate({
      in: todoPath,
      out: patchPath,
      runner,
      model,
      style,
      'max-seconds': String(maxSeconds),
    });

    cmdApply({ in: patchPath, zh: zhPath });
  }

  if (once || !watch) {
    syncOnce();
    return;
  }

  console.log(`sync: watching ${logPath}`);
  let timer = null;
  const schedule = () => {
    if (timer) clearTimeout(timer);
    timer = setTimeout(() => {
      timer = null;
      syncOnce();
    }, debounceMs);
  };

  const dir = path.dirname(logPath);
  try {
    fs.mkdirSync(dir, { recursive: true });
  } catch (e) {
    fail(`sync: failed to prepare watch directory ${dir}: ${e}`);
  }
  try {
    const watcher = fs.watch(dir, { persistent: true }, (_event, filename) => {
      if (filename && filename !== path.basename(logPath)) return;
      schedule();
    });

    // Run once on startup so an existing log is processed immediately.
    schedule();

    process.on('SIGINT', () => {
      watcher.close();
      process.exit(0);
    });
  } catch (e) {
    fail(`sync: failed to watch directory ${dir}: ${e}`);
  }
}

function cmdHelp() {
  console.log(`i18n-collector

Commands:
  extract --log <jsonl> --en <en.json> --out <todo.json>
  stats   --log <jsonl> --en <en.json> --zh <zh-CN.json> [--json] [--group-by type|app|none] [--top 20] [--rules <rules.json>]
  translate --in <todo.json> --out <translated.json> [--runner coder] [--model code-gpt-5.2-codex] [--style zh-only|bilingual-tui]
  apply   --in <translated.json> --zh <zh-CN.json>
  sync    --log <jsonl> --en <en.json> --zh <zh-CN.json> [--watch] [--once] [--out-dir <dir>] [--runner coder] [--model code-gpt-5.2-codex] [--style zh-only|bilingual-tui]
`);
}

function main() {
  const argv = process.argv.slice(2);
  const cmd = argv[0];
  const args = parseArgs(argv.slice(1));
  switch (cmd) {
    case 'extract':
      cmdExtract(args);
      break;
    case 'stats':
      cmdStats(args);
      break;
    case 'translate':
      cmdTranslate(args);
      break;
    case 'apply':
      cmdApply(args);
      break;
    case 'sync':
      cmdSync(args);
      break;
    case 'help':
    case undefined:
      cmdHelp();
      break;
    default:
      fail(`Unknown command: ${cmd}`);
  }
}

main();
