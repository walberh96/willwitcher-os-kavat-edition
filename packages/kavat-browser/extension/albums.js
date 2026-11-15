// The albums' page: the albums by name, and the one that is chosen as a grid of thumbnails.
// The album in the address, albums.html#name, is the one shown.
import { albums, grid } from './grid.js';
import { theme } from './theme.js';

const names = document.getElementById('names');
const sites = document.getElementById('sites');
let chosen = decodeURIComponent(location.hash.slice(1));
let shown = [];

async function draw() {
  const all = await albums();
  if (!all.some((album) => album.name === chosen)) {
    chosen = all.length ? all[0].name : '';
  }
  names.replaceChildren(...all.map((album) => {
    const link = document.createElement('a');
    link.href = `#${encodeURIComponent(album.name)}`;
    link.className = album.name === chosen ? 'on' : '';
    link.textContent = `${album.name} `;
    const count = document.createElement('span');
    count.className = 'muted';
    count.textContent = album.count;
    link.append(count);
    return link;
  }));
  shown = chosen ? await grid(sites, chosen, true) : [];
  if (!chosen) {
    sites.replaceChildren();
  }
  document.getElementById('empty').hidden = shown.length > 0;
  document.getElementById('all').hidden = shown.length === 0;
  const start = document.getElementById('start');
  start.hidden = !chosen;
  start.textContent = (await onStart()) === chosen ? 'On the start page ✓' : 'Show on the start page';
}

document.getElementById('all').addEventListener('click', () => {
  for (const site of shown) {
    chrome.tabs.create({ url: site.url, active: false });
  }
});
// the album the start page shows: a line of the browser's settings file, which the host keeps
async function onStart() {
  const answer = await chrome.runtime.sendMessage({ ask: 'settings' });
  return (answer && answer.settings && answer.settings.start_album) || '';
}
document.getElementById('start').addEventListener('click', async () => {
  await chrome.runtime.sendMessage({ ask: 'start-album', name: (await onStart()) === chosen ? '' : chosen });
  draw();
});
window.addEventListener('hashchange', () => {
  chosen = decodeURIComponent(location.hash.slice(1));
  draw();
});
theme();
draw();
