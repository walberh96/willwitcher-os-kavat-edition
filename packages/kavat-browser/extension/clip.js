// A clipping as the console's form asks for it: the page in the notes with a comment and
// tags of yours, and as its picture what the tab shows, a region you drag over the page, or
// the whole page, taken a screen at a time while it is scrolled and put together here.
import { current, selection, shot } from './commands.js';

const SCREENS = 12;         // the most of a page that is taken: this many times what is in view
const WIDE = 1280;          // the whole page is made no wider than the notes keep a picture
const LARGEST = 6000000;    // what the host takes in one message, with room to spare
const BETWEEN = 550;        // the browser gives two pictures of a tab a second, and no more

const wait = (time) => new Promise((done) => setTimeout(done, time));

// Run a function inside the page and wait for what it gives back.
async function inside(tab, func, ...args) {
  const [found] = await chrome.scripting.executeScript({ target: { tabId: tab.id }, func, args });
  return found && found.result;
}

// In the page: a veil over it to drag a rectangle on. It gives the rectangle back, with the
// width of what is in view to measure it against, or nothing when Esc is pressed.
function mark(accent) {
  return new Promise((done) => {
    const veil = document.createElement('div');
    veil.style.cssText = 'all: initial; position: fixed; inset: 0; z-index: 2147483647; cursor: crosshair; background: rgba(0, 0, 0, 0.25);';
    const box = document.createElement('div');
    box.style.cssText = `all: initial; position: fixed; display: none; border: 2px solid ${accent}; box-shadow: 0 0 0 100vmax rgba(0, 0, 0, 0.35); pointer-events: none;`;
    const hint = document.createElement('div');
    hint.textContent = 'Drag over what to keep · Esc leaves';
    hint.style.cssText = `all: initial; position: fixed; top: 16px; left: 50%; transform: translateX(-50%); padding: 6px 14px; border-radius: 99px; background: #151821; color: #e8e6df; border: 1px solid ${accent}; font: 13px Inter, system-ui, sans-serif; pointer-events: none;`;
    veil.append(box, hint);
    let from = null;
    let chosen = null;
    const leave = (found) => {
      removeEventListener('keydown', key, true);
      veil.remove();
      // the picture is taken once the page is drawn again without the veil
      requestAnimationFrame(() => requestAnimationFrame(() => done(found)));
    };
    const key = (event) => {
      if (event.key === 'Escape') {
        event.preventDefault();
        event.stopPropagation();
        leave(null);
      }
    };
    veil.addEventListener('mousedown', (event) => {
      event.preventDefault();
      from = { x: event.clientX, y: event.clientY };
      veil.style.background = 'none';
      hint.remove();
    });
    veil.addEventListener('mousemove', (event) => {
      if (!from) {
        return;
      }
      chosen = { x: Math.min(from.x, event.clientX), y: Math.min(from.y, event.clientY), width: Math.abs(event.clientX - from.x), height: Math.abs(event.clientY - from.y) };
      box.style.display = 'block';
      box.style.left = `${chosen.x}px`;
      box.style.top = `${chosen.y}px`;
      box.style.width = `${chosen.width}px`;
      box.style.height = `${chosen.height}px`;
    });
    veil.addEventListener('mouseup', () => {
      // a press with no drag chooses nothing: the veil waits for another
      if (chosen && chosen.width > 4 && chosen.height > 4) {
        leave({ ...chosen, view: innerWidth });
      } else {
        from = null;
        chosen = null;
        box.style.display = 'none';
      }
    });
    addEventListener('keydown', key, true);
    document.documentElement.append(veil);
  });
}

// In the page: how much of it is in view, how long it is, and where it is scrolled to. Its
// scroll bar is put away first, or it would be in the picture once for every screen.
function measure() {
  const at = scrollY;
  document.documentElement.dataset.kavatBar = document.documentElement.style.scrollbarWidth;
  document.documentElement.style.setProperty('scrollbar-width', 'none', 'important');
  return { width: innerWidth, height: innerHeight, full: document.scrollingElement.scrollHeight, at };
}

// In the page: scroll to a place and say where it got to. From the second screen on, what
// stays put while the page moves, a bar at the top or a panel at the side, is hidden, so
// that it is in the picture once and not on every screen.
function move(top, hide) {
  if (hide && !document.documentElement.dataset.kavatWhole) {
    document.documentElement.dataset.kavatWhole = 'on';
    for (const element of document.body.querySelectorAll('*')) {
      const position = getComputedStyle(element).position;
      if (position === 'fixed' || position === 'sticky') {
        element.dataset.kavatHidden = element.style.visibility;
        element.style.setProperty('visibility', 'hidden', 'important');
      }
    }
  }
  scrollTo({ top, behavior: 'instant' });
  return new Promise((done) => requestAnimationFrame(() => requestAnimationFrame(() => done(scrollY))));
}

// In the page: leave it as it was.
function back(top) {
  for (const element of document.querySelectorAll('[data-kavat-hidden]')) {
    element.style.visibility = element.dataset.kavatHidden;
    delete element.dataset.kavatHidden;
  }
  delete document.documentElement.dataset.kavatWhole;
  document.documentElement.style.scrollbarWidth = document.documentElement.dataset.kavatBar || '';
  delete document.documentElement.dataset.kavatBar;
  scrollTo({ top, behavior: 'instant' });
}

// A picture the browser gave, as something that can be drawn.
async function drawable(picture) {
  return createImageBitmap(await (await fetch(picture)).blob());
}

// What was drawn, as the text a message can carry, small enough for the host to take.
async function text(canvas) {
  for (const quality of [0.7, 0.5, 0.3]) {
    const bytes = new Uint8Array(await (await canvas.convertToBlob({ type: 'image/jpeg', quality })).arrayBuffer());
    let binary = '';
    for (let at = 0; at < bytes.length; at += 0x8000) {
      binary += String.fromCharCode(...bytes.subarray(at, at + 0x8000));
    }
    const made = `data:image/jpeg;base64,${btoa(binary)}`;
    if (made.length < LARGEST) {
      return made;
    }
  }
  return '';
}

// The part of what the tab shows that was dragged over.
async function region(tab) {
  const accent = ((await chrome.storage.local.get('theme')).theme || {}).accent;
  const chosen = await inside(tab, mark, /^#[0-9a-f]{6}$/i.test(accent || '') ? accent : '#f2a93b');
  if (!chosen) {
    return null;
  }
  const seen = await drawable(await shot(tab));
  // the picture is in the screen's own dots, the rectangle in the page's
  const scale = seen.width / chosen.view;
  const width = Math.max(1, Math.min(Math.round(chosen.width * scale), seen.width - Math.round(chosen.x * scale)));
  const height = Math.max(1, Math.min(Math.round(chosen.height * scale), seen.height - Math.round(chosen.y * scale)));
  const canvas = new OffscreenCanvas(width, height);
  canvas.getContext('2d').drawImage(seen, Math.round(chosen.x * scale), Math.round(chosen.y * scale), width, height, 0, 0, width, height);
  return text(canvas);
}

// The whole page, from its top: a picture of each screen, one under the other.
async function whole(tab) {
  const page = await inside(tab, measure);
  const tall = Math.min(page.full, page.height * SCREENS);
  const scale = Math.min(1, WIDE / page.width);
  const canvas = new OffscreenCanvas(Math.round(page.width * scale), Math.round(tall * scale));
  const paper = canvas.getContext('2d');
  try {
    for (let top = 0; top < tall; top += page.height) {
      // the end of a page stops short of where it was asked to go
      const reached = await inside(tab, move, top, top > 0);
      await wait(BETWEEN);
      const seen = await drawable(await chrome.tabs.captureVisibleTab(tab.windowId, { format: 'jpeg', quality: 80 }));
      paper.drawImage(seen, 0, Math.round(reached * scale), canvas.width, Math.round(page.height * scale));
      if (reached < top) {
        break;
      }
    }
  } finally {
    await inside(tab, back, page.at);
  }
  return text(canvas);
}

// Save the tab in view as a clipping. `how` is what the form holds: a comment, tags, and
// which picture: 'view', 'region' or 'whole'.
export async function clip(send, how) {
  const tab = await current();
  if (!/^https?:/.test(tab.url || '')) {
    return { failed: 'only a page of the web can be saved to the notes' };
  }
  // read before the page is touched: a drag over it could let go of what is selected
  const quote = await selection(tab);
  let picture = '';
  try {
    if (how.picture === 'region') {
      picture = await region(tab);
      if (picture === null) {
        return { failed: 'no region was chosen, and nothing was saved' };
      }
    } else if (how.picture === 'whole') {
      picture = await whole(tab);
    } else {
      picture = await shot(tab);
    }
  } catch (error) {
    // a page that does not let itself be touched is saved with what the tab shows
    picture = await shot(tab);
  }
  return send({ ask: 'clip', title: tab.title, url: tab.url, quote, picture, comment: how.comment || '', tags: how.tags || '' });
}
