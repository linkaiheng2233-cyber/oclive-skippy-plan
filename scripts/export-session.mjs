// Export a DSH session into a readable Markdown transcript.
//
// A DSH session lives in ~/.dsh/sessions/<workspace-key>/<session-id>/session.v4.jsonl.zstd.
// That file is a CONCATENATION of zstd frames (one per append), so a normal
// single-shot decompress only yields the first frame (the session header).
// This script walks every frame, parses the JSONL events, and flattens
// user/assistant turns into Markdown. Tool calls are counted, not dumped.
//
// Usage:
//   node scripts/export-session.mjs <session.v4.jsonl.zstd> <output.md> [--redact <file>]
//
// --redact <file>: one literal string per line; occurrences are replaced with
// [已脱敏]. Used for Wi-Fi SSID / host identifiers before publishing.

import { readFileSync, writeFileSync } from 'node:fs';
import zlib from 'node:zlib';

const [input, output, flag, redactFile] = process.argv.slice(2);
if (!input || !output) {
  console.error('usage: node scripts/export-session.mjs <input.zstd> <output.md> [--redact <file>]');
  process.exit(2);
}

const redactions = [];
if (flag === '--redact' && redactFile) {
  for (const line of readFileSync(redactFile, 'utf8').split(/\r?\n/)) {
    const value = line.trim();
    if (value) redactions.push(value);
  }
}
const redact = (text) => redactions.reduce((acc, value) => acc.split(value).join('[已脱敏]'), text);

// --- decode every concatenated zstd frame ---------------------------------
const buffer = readFileSync(input);
const FRAME_MAGIC = [0x28, 0xb5, 0x2f, 0xfd];
const offsets = [];
for (let i = 0; i + 3 < buffer.length; i += 1) {
  if (
    buffer[i] === FRAME_MAGIC[0] &&
    buffer[i + 1] === FRAME_MAGIC[1] &&
    buffer[i + 2] === FRAME_MAGIC[2] &&
    buffer[i + 3] === FRAME_MAGIC[3]
  ) {
    offsets.push(i);
    i += 3;
  }
}
let decoded = Buffer.alloc(0);
let failedFrames = 0;
for (let k = 0; k < offsets.length; k += 1) {
  const start = offsets[k];
  const end = k + 1 < offsets.length ? offsets[k + 1] : buffer.length;
  try {
    decoded = Buffer.concat([decoded, zlib.zstdDecompressSync(buffer.subarray(start, end))]);
  } catch {
    failedFrames += 1;
  }
}

const events = [];
for (const line of decoded.toString('utf8').split(/\r?\n/)) {
  const text = line.trim();
  if (!text) continue;
  try {
    events.push(JSON.parse(text));
  } catch {
    // Skip unparseable lines; the transcript stays useful.
  }
}

// --- flatten turns ---------------------------------------------------------
const turns = [];
const toolCounts = new Map();

function textOf(content) {
  if (typeof content === 'string') return content;
  if (!Array.isArray(content)) return '';
  return content
    .filter((item) => item && item.type === 'text' && typeof item.text === 'string')
    .map((item) => item.text)
    .join('\n');
}

for (const event of events) {
  if (event?.type === 'user/message') {
    const text = textOf(event.data?.content);
    if (text.trim()) turns.push({ role: 'user', seq: event.seq ?? 0, text: text.trim() });
    continue;
  }
  if (event?.type === 'assistant/message') {
    const text = textOf(event.data?.message?.content);
    if (text.trim()) turns.push({ role: 'assistant', seq: event.seq ?? 0, text: text.trim() });
    continue;
  }
  if (event?.type === 'tool/call') {
    const name = event.data?.name ?? 'tool';
    toolCounts.set(name, (toolCounts.get(name) ?? 0) + 1);
  }
}

// --- write markdown --------------------------------------------------------
const startedAt = events.find((event) => event.type === 'session')?.createdAt;
const lines = [];
lines.push('# 会话记录');
lines.push('');
lines.push(`- 导出时间：${new Date().toISOString()}`);
if (startedAt) lines.push(`- 会话开始：${new Date(startedAt).toISOString()}`);
lines.push(`- 事件数：${events.length}（压缩帧 ${offsets.length}，解码失败 ${failedFrames}）`);
lines.push(`- 轮次：用户 ${turns.filter((t) => t.role === 'user').length} / 助手 ${turns.filter((t) => t.role === 'assistant').length}`);
if (toolCounts.size > 0) {
  const summary = [...toolCounts.entries()]
    .sort((a, b) => b[1] - a[1])
    .map(([name, count]) => `${name}×${count}`)
    .join('、');
  lines.push(`- 工具调用：${summary}`);
}
if (redactions.length > 0) lines.push(`- 已脱敏：${redactions.length} 项`);
lines.push('');
lines.push('> 本文件是**对话的可读摘录**：只保留用户与助手的文字，工具调用的具体参数' +
  '不计入（数量见上）。完整原始事件不入库，避免公开仓库泄露本机路径与网络标识。');
lines.push('');
lines.push('---');
lines.push('');

for (const turn of turns) {
  lines.push(turn.role === 'user' ? '### 用户' : '### 助手');
  lines.push('');
  lines.push(redact(turn.text));
  lines.push('');
}

writeFileSync(output, lines.join('\n'), 'utf8');
console.log(
  `export-session: ${turns.length} turns, ${events.length} events, ` +
    `${(Buffer.byteLength(lines.join('\n')) / 1024).toFixed(0)} KiB -> ${output}`,
);
