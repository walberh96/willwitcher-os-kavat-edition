// The start page: the clock, the search, the sites most gone to, and an album when one was
// chosen for it. Nothing on it comes from the network: the sites are the browser's own
// list, their icons the ones it keeps, and the album a folder of the notes.
import { grid } from './grid.js';
import { theme } from './theme.js';

function tick() {
  const now = new Date();
  document.getElementById('clock').textContent = now.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', hour12: false });
  document.getElementById('day').textContent = now.toLocaleDateString('en', { weekday: 'long', day: 'numeric', month: 'long' });
}

document.getElementById('search').addEventListener('submit', (event) => {
  event.preventDefault();
  const words = document.getElementById('words').value.trim();
  if (!words) {
    return;
  }
  // an address goes where it says; anything else goes to the search the browser is set to
  if (/^[a-z][a-z0-9+.-]*:\/\//i.test(words)) {
    location.href = words;
  } else if (/^[^\s/]+\.[a-z]{2,}(\/\S*)?$/i.test(words) || /^localhost(:\d+)?(\/\S*)?$/.test(words)) {
    location.href = `https://${words}`;
  } else {
    chrome.search.query({ text: words, disposition: 'CURRENT_TAB' });
  }
});

async function sites() {
  const found = (await chrome.topSites.get()).slice(0, 8);
  document.getElementById('sites').replaceChildren(...found.map((site) => {
    const link = document.createElement('a');
    link.href = site.url;
    const icon = document.createElement('img');
    icon.src = `chrome-extension://${chrome.runtime.id}/_favicon/?pageUrl=${encodeURIComponent(site.url)}&size=32`;
    icon.alt = '';
    const name = document.createElement('span');
    name.textContent = site.title || new URL(site.url).hostname;
    link.append(icon, name);
    return link;
  }));
}

async function mode() {
  const answer = await chrome.runtime.sendMessage({ ask: 'mode' });
  const badge = document.getElementById('mode');
  if (answer && answer.mode && answer.mode !== 'coder') {
    badge.textContent = `${answer.mode[0].toUpperCase()}${answer.mode.slice(1)} mode`;
    badge.hidden = false;
  }
}

// the album chosen for the start page, on the albums' page or in the browser's settings file
async function album() {
  const answer = await chrome.runtime.sendMessage({ ask: 'settings' });
  const name = answer && answer.settings && answer.settings.start_album;
  if (name) {
    document.body.classList.add('with-album');
    grid(document.getElementById('album'), name);
  }
}
theme();
album();
tick();
setInterval(tick, 5000);
sites();
mode();
