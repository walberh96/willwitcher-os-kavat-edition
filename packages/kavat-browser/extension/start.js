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

// What the page shows behind it: the theme's colour, the desktop's background or a picture
// of yours. The picture is kept by the browser, so a new tab shows it at once; the system
// is asked whether it is still the one, and only then for the picture itself.
function wear(picture) {
  document.body.classList.toggle('pictured', Boolean(picture));
  document.body.style.backgroundImage = picture ? `linear-gradient(var(--scrim), var(--scrim)), url("${picture}")` : '';
}

async function background(settings) {
  const kept = await chrome.storage.local.get(['backgroundStamp', 'backgroundPicture']);
  const stamp = (settings && settings.start_background_stamp) || '';
  if (stamp && stamp === kept.backgroundStamp) {
    return;
  }
  if (!stamp) {
    wear('');
    await chrome.storage.local.remove(['backgroundStamp', 'backgroundPicture']);
    return;
  }
  const answer = await chrome.runtime.sendMessage({ ask: 'start-background' });
  if (answer && answer.picture) {
    wear(answer.picture);
    await chrome.storage.local.set({ backgroundStamp: answer.stamp, backgroundPicture: answer.picture });
  }
}

// a picture of yours is made no wider than a screen here, and handed to the system to keep
function shrunk(file) {
  return new Promise((done, failed) => {
    const picture = new Image();
    picture.onload = () => {
      const scale = Math.min(1, 1920 / picture.naturalWidth);
      const canvas = document.createElement('canvas');
      canvas.width = Math.round(picture.naturalWidth * scale);
      canvas.height = Math.round(picture.naturalHeight * scale);
      canvas.getContext('2d').drawImage(picture, 0, 0, canvas.width, canvas.height);
      URL.revokeObjectURL(picture.src);
      done(canvas.toDataURL('image/jpeg', 0.82));
    };
    picture.onerror = failed;
    picture.src = URL.createObjectURL(file);
  });
}

async function choose(message) {
  const answer = await chrome.runtime.sendMessage({ ask: 'start-background-set', ...message });
  if (answer && answer.settings) {
    await background(answer.settings);
  }
}

const menu = document.getElementById('background-menu');
document.getElementById('background-button').addEventListener('click', () => { menu.hidden = !menu.hidden; });
menu.addEventListener('click', (event) => {
  const value = event.target.dataset.value;
  menu.hidden = true;
  if (value === 'picture') {
    document.getElementById('background-file').click();
  } else if (value) {
    choose({ value });
  }
});
document.getElementById('background-file').addEventListener('change', async (event) => {
  const file = event.target.files[0];
  if (file) {
    choose({ picture: await shrunk(file) });
  }
});

// the settings of the start page: the album chosen for it, on the albums' page or in the
// browser's settings file, and its background
async function settings() {
  const answer = await chrome.runtime.sendMessage({ ask: 'settings' });
  const name = answer && answer.settings && answer.settings.start_album;
  if (name) {
    document.body.classList.add('with-album');
    grid(document.getElementById('album'), name);
  }
  background(answer && answer.settings);
}
// the picture kept from last time shows before anything is asked
chrome.storage.local.get(['backgroundPicture']).then((kept) => kept.backgroundPicture && wear(kept.backgroundPicture));
theme();
settings();
tick();
setInterval(tick, 5000);
sites();
mode();
