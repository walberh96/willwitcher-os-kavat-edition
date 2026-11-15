// The notes' panel: the notes, clippings and album sites that mention the site of the tab
// in view. It follows the tab: another tab, or another page in it, and it asks again. A
// note opens in the editor.
import { theme } from './theme.js';

const KINDS = { note: 'Note', clip: 'Clipping', album: 'Album' };
let address = '';

async function draw() {
  const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
  const now = (tab && tab.url) || '';
  if (now === address) {
    return;
  }
  address = now;
  let host = '';
  try {
    host = /^https?:/.test(now) ? new URL(now).hostname : '';
  } catch (error) {
    host = '';
  }
  document.getElementById('site').textContent = host || 'Not a site';
  const answer = host ? await chrome.runtime.sendMessage({ ask: 'about', url: now }) : { notes: [] };
  const notes = (answer && answer.notes) || [];
  document.getElementById('notes').replaceChildren(...notes.map((note) => {
    const item = document.createElement('li');
    const kind = document.createElement('span');
    kind.className = 'kind muted';
    kind.textContent = KINDS[note.kind] || note.kind;
    const title = document.createElement('span');
    title.textContent = note.title;
    item.append(title, kind);
    item.title = 'Open in the editor';
    item.addEventListener('click', () => chrome.runtime.sendMessage({ ask: 'note', file: note.file }));
    return item;
  }));
  document.getElementById('none').hidden = notes.length > 0 || !host;
}

function button(id, command) {
  document.getElementById(id).addEventListener('click', async () => {
    const result = await chrome.runtime.sendMessage({ run: command });
    document.getElementById('said').textContent = (result && (result.failed || result.said)) || '';
    address = '';
    draw();
  });
}

button('clip', 'save to notes');
button('album', 'add to album');
button('later', 'read later');
chrome.tabs.onActivated.addListener(draw);
chrome.tabs.onUpdated.addListener((id, change) => change.url && draw());
theme();
draw();
