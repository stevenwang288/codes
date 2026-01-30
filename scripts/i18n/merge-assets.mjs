import fs from "node:fs";
import path from "node:path";

function usage() {
  // eslint-disable-next-line no-console
  console.error(
    "Usage: node scripts/i18n/merge-assets.mjs --base <en.json> --existing <zh.json> [--out <zh.json>]"
  );
  process.exit(2);
}

function parseArgs(argv) {
  const args = { base: null, existing: null, out: null };
  for (let i = 2; i < argv.length; i++) {
    const a = argv[i];
    if (a === "--base") args.base = argv[++i] ?? null;
    else if (a === "--existing") args.existing = argv[++i] ?? null;
    else if (a === "--out") args.out = argv[++i] ?? null;
    else usage();
  }
  if (!args.base || !args.existing) usage();
  if (!args.out) args.out = args.existing;
  return args;
}

function readJson(filePath) {
  const raw = fs.readFileSync(filePath, "utf8");
  return JSON.parse(raw);
}

function stableStringify(obj) {
  const keys = Object.keys(obj).sort((a, b) => a.localeCompare(b));
  const out = {};
  for (const k of keys) out[k] = obj[k];
  return JSON.stringify(out, null, 2) + "\n";
}

const { base, existing, out } = parseArgs(process.argv);
const basePath = path.resolve(base);
const existingPath = path.resolve(existing);
const outPath = path.resolve(out);

const baseJson = readJson(basePath);
const existingJson = readJson(existingPath);

const merged = {};
for (const key of Object.keys(baseJson)) {
  if (Object.prototype.hasOwnProperty.call(existingJson, key)) {
    merged[key] = existingJson[key];
  } else {
    merged[key] = baseJson[key];
  }
}

fs.mkdirSync(path.dirname(outPath), { recursive: true });
fs.writeFileSync(outPath, stableStringify(merged), "utf8");
