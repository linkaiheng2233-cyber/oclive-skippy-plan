// Minimal Markdown → single-file HTML converter for handoff documents.
//
// Why not a library: this repository's Node environment has no `marked`/`pandoc`,
// and the only need is a readable, phone-friendly export of a handoff document
// that the owner can forward through WeChat. The subset supported here is
// deliberately small: headings, paragraphs, lists, fenced code, tables, bold,
// inline code and links.
//
// Usage:
//   node scripts/md-to-html.mjs <input.md> <output.html> [--title "标题"]

import { readFileSync, writeFileSync } from 'node:fs';

const [input, output, titleFlag, titleValue] = process.argv.slice(2);
if (!input || !output) {
  console.error('usage: node scripts/md-to-html.mjs <input.md> <output.html> [--title "标题"]');
  process.exit(2);
}

const escapeHtml = (text) =>
  text.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');

function inline(text) {
  // 先把 Markdown 的 <url> 自动链接改写成 [url](url)，避免被当作普通尖括号转义。
  const normalized = text.replace(/<(https?:\/\/[^>\s]+)>/g, '[$1]($1)');
  let html = escapeHtml(normalized);
  html = html.replace(/`([^`]+)`/g, '<code>$1</code>');
  html = html.replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>');
  html = html.replace(/\[([^\]]+)\]\(([^)]+)\)/g, '<a href="$2">$1</a>');
  html = html.replace(/<([^>\s]+@[^>\s]+)>/g, '<a href="mailto:$1">$1</a>');
  html = html.replace(/(^|[^"=])(https?:\/\/[^\s<]+)/g, '$1<a href="$2">$2</a>');
  return html;
}

const lines = readFileSync(input, 'utf8').split(/\r?\n/);
const body = [];
let index = 0;
let paragraph = [];
let listType = null;
let inCode = false;
let codeLines = [];
let tableRows = [];

function flushParagraph() {
  if (paragraph.length > 0) {
    body.push(`<p>${paragraph.join('<br>')}</p>`);
    paragraph = [];
  }
}

function flushList() {
  if (listType) {
    body.push(`</${listType}>`);
    listType = null;
  }
}

function flushTable() {
  if (tableRows.length === 0) return;
  const rows = tableRows
    .filter((row) => !/^\|[\s:|-]+\|$/.test(row))
    .map((row) => row.replace(/^\||\|$/g, '').split('|').map((cell) => cell.trim()));
  if (rows.length === 0) {
    tableRows = [];
    return;
  }
  const [head, ...rest] = rows;
  const html = [];
  html.push('<table><thead><tr>');
  for (const cell of head) html.push(`<th>${inline(cell)}</th>`);
  html.push('</tr></thead><tbody>');
  for (const row of rest) {
    html.push('<tr>');
    for (const cell of row) html.push(`<td>${inline(cell)}</td>`);
    html.push('</tr>');
  }
  html.push('</tbody></table>');
  body.push(html.join(''));
  tableRows = [];
}

while (index < lines.length) {
  const line = lines[index];

  if (line.startsWith('```')) {
    if (!inCode) {
      flushParagraph(); flushList(); flushTable();
      inCode = true;
      codeLines = [];
    } else {
      body.push(`<pre><code>${escapeHtml(codeLines.join('\n'))}</code></pre>`);
      inCode = false;
    }
    index += 1;
    continue;
  }
  if (inCode) {
    codeLines.push(line);
    index += 1;
    continue;
  }

  if (/^\|.*\|$/.test(line.trim())) {
    flushParagraph(); flushList();
    tableRows.push(line.trim());
    index += 1;
    continue;
  }
  flushTable();

  if (line.trim() === '') {
    flushParagraph(); flushList();
    index += 1;
    continue;
  }

  const heading = /^(#{1,4})\s+(.*)$/.exec(line);
  if (heading) {
    flushParagraph(); flushList();
    const level = heading[1].length;
    body.push(`<h${level}>${inline(heading[2])}</h${level}>`);
    index += 1;
    continue;
  }

  if (line.trim() === '---') {
    flushParagraph(); flushList();
    body.push('<hr>');
    index += 1;
    continue;
  }

  const bullet = /^\s*[-*]\s+(.*)$/.exec(line);
  if (bullet) {
    flushParagraph();
    if (listType !== 'ul') {
      flushList();
      body.push('<ul>');
      listType = 'ul';
    }
    body.push(`<li>${inline(bullet[1])}</li>`);
    index += 1;
    continue;
  }

  const numbered = /^\s*\d+\.\s+(.*)$/.exec(line);
  if (numbered) {
    flushParagraph();
    if (listType !== 'ol') {
      flushList();
      body.push('<ol>');
      listType = 'ol';
    }
    body.push(`<li>${inline(numbered[1])}</li>`);
    index += 1;
    continue;
  }

  if (line.startsWith('>')) {
    flushParagraph(); flushList();
    body.push(`<blockquote>${inline(line.replace(/^>\s?/, ''))}</blockquote>`);
    index += 1;
    continue;
  }

  paragraph.push(inline(line));
  index += 1;
}

flushParagraph(); flushList(); flushTable();

const title = titleValue ?? input.split(/[\\/]/).pop().replace(/\.md$/, '');
const html = `<!doctype html>
<html lang="zh-CN">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>${escapeHtml(title)}</title>
<style>
  :root { color-scheme: light dark; }
  body {
    margin: 0 auto; padding: 24px 18px 64px; max-width: 820px;
    font: 15px/1.7 -apple-system, "Segoe UI", "Microsoft YaHei", system-ui, sans-serif;
    color: #1d1c22; background: #fbfbfd;
  }
  h1 { font-size: 26px; margin: 8px 0 20px; }
  h2 { font-size: 20px; margin: 32px 0 12px; padding-bottom: 6px; border-bottom: 1px solid #e3e0ee; }
  h3 { font-size: 17px; margin: 24px 0 10px; }
  h4 { font-size: 15px; margin: 20px 0 8px; }
  p { margin: 10px 0; }
  a { color: #4a3f8f; }
  code { font: 13px/1.5 ui-monospace, Consolas, monospace; background: #efedf7; padding: 1px 5px; border-radius: 4px; }
  pre { background: #211f2e; color: #f3f0ff; padding: 14px 16px; border-radius: 10px; overflow-x: auto; }
  pre code { background: none; color: inherit; padding: 0; }
  table { width: 100%; border-collapse: collapse; margin: 14px 0; font-size: 14px; }
  th, td { border: 1px solid #e3e0ee; padding: 8px 10px; text-align: left; vertical-align: top; }
  th { background: #f1eefb; }
  blockquote { margin: 14px 0; padding: 10px 14px; border-left: 3px solid #b9aef0; background: #f4f1fd; }
  hr { border: 0; border-top: 1px solid #e3e0ee; margin: 28px 0; }
  ul, ol { padding-left: 22px; }
  li { margin: 4px 0; }
  @media (prefers-color-scheme: dark) {
    body { color: #e8e5f2; background: #17151f; }
    h2 { border-color: #322d44; }
    a { color: #b9aef0; }
    code { background: #2a2638; }
    th, td { border-color: #322d44; }
    th { background: #241f33; }
    blockquote { background: #241f33; }
    hr { border-color: #322d44; }
  }
</style>
</head>
<body>
${body.join('\n')}
</body>
</html>
`;

writeFileSync(output, html, 'utf8');
console.log(`md-to-html: ${(Buffer.byteLength(html) / 1024).toFixed(0)} KiB -> ${output}`);

