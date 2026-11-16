// The part of the Kavat extension that is always there: it talks to the system through a
// small program, the host, which the browser starts and speaks to in messages. Through it
// come the commands of your own, the system's mode and what the system asks the browser to
// do; through it go the clippings for the notes and the folders to open in Yazi.
import { COMMANDS, run } from './commands.js';
import { clip } from './clip.js';

const HOST = 'dev.kavat.browser';
let port = null;
let waiting = new Map();  // questions to the host that have no answer yet, by their number
let asked = 0;

function connect() {
  if (port) {
    return port;
  }
  port = chrome.runtime.connectNative(HOST);
  port.onMessage.addListener(heard);
  port.onDisconnect.addListener(() => {
    port = null;
    for (const done of waiting.values()) {
      done({ failed: 'the system does not answer' });
    }
    waiting = new Map();
  });
  return port;
}

// Ask the host something and wait for its answer.
function send(message) {
  return new Promise((done) => {
    asked += 1;
    const number = asked;
    waiting.set(number, done);
    try {
      connect().postMessage({ ...message, number });
    } catch (error) {
      waiting.delete(number);
      done({ failed: 'the system does not answer' });
    }
    setTimeout(() => {
      if (waiting.delete(number)) {
        done({ failed: 'the system took too long to answer' });
      }
    }, 15000);
  });
}

async function heard(message) {
  if (message.number && waiting.has(message.number)) {
    const done = waiting.get(message.number);
    waiting.delete(message.number);
    done(message);
  } else if (message.event === 'mode') {
    mode(message.mode);
  } else if (message.event === 'command') {
    // the system asks for a command, as the console would
    const result = await run(message.name, send, message.given);
    connect().postMessage({ event: 'done', name: message.name, ...result });
  }
}

// What the browser does when the system's mode changes: in Gamer the tabs that are not in
// view go to sleep, to leave their memory to the game.
async function mode(now) {
  await chrome.storage.session.set({ mode: now });
  if (now === 'gamer') {
    run('sleep other tabs', send);
  }
}

chrome.commands.onCommand.addListener((name) => {
  if (name === 'clip-page') {
    run('save to notes', send).then(tell);
  } else if (name === 'read-later') {
    run('read later', send).then(tell);
  } else if (name === 'zen') {
    run('zen mode', send);
  }
});

// A word on the toolbar's button, for what was done without the console open.
function tell(result) {
  chrome.action.setBadgeBackgroundColor({ color: result.failed ? '#d9534f' : '#3fb950' });
  chrome.action.setBadgeText({ text: result.failed ? '!' : '✓' });
  chrome.action.setTitle({ title: result.failed ? `Kavat: ${result.failed}` : `Kavat: ${result.said || 'done'}` });
  setTimeout(() => chrome.action.setBadgeText({ text: '' }), 4000);
}

chrome.runtime.onInstalled.addListener(() => {
  chrome.contextMenus.create({ id: 'clip', title: 'Save to the notes', contexts: ['page', 'selection', 'link'] });
  chrome.contextMenus.create({ id: 'album', title: 'Add to the album of favourites', contexts: ['page', 'link'] });
  chrome.contextMenus.create({ id: 'later', title: 'Read later', contexts: ['page', 'link'] });
  chrome.contextMenus.create({ id: 'image', title: 'Save the image to the album of images', contexts: ['image'] });
  chrome.contextMenus.create({ id: 'about', title: 'Notes about this site', contexts: ['page'] });
});

// An image of a page, as the text a message can carry; nothing when it cannot be fetched.
async function fetched(address) {
  try {
    const blob = await (await fetch(address)).blob();
    if (!/^image\/(png|jpeg|webp)$/.test(blob.type) || blob.size > 5000000) {
      return '';
    }
    const bytes = new Uint8Array(await blob.arrayBuffer());
    let text = '';
    for (let at = 0; at < bytes.length; at += 0x8000) {
      text += String.fromCharCode(...bytes.subarray(at, at + 0x8000));
    }
    return `data:${blob.type};base64,${btoa(text)}`;
  } catch (error) {
    return '';
  }
}
chrome.contextMenus.onClicked.addListener(async (info, tab) => {
  if (info.menuItemId === 'clip') {
    tell(await send({ ask: 'clip', title: tab.title, url: info.linkUrl || tab.url, quote: info.selectionText || '' }));
  } else if (info.menuItemId === 'about') {
    // asked for before anything is waited for: the browser opens a panel only at a press
    chrome.sidePanel.open({ windowId: tab.windowId });
  } else if (info.menuItemId === 'album' || info.menuItemId === 'later') {
    const name = info.menuItemId === 'album' ? 'favourites' : 'read-later';
    tell(await send({ ask: 'album-add', name, title: info.linkUrl ? info.linkUrl : tab.title, url: info.linkUrl || tab.url }));
  } else if (info.menuItemId === 'image') {
    // the image, with the page it came from
    tell(await send({ ask: 'album-add', name: 'images', title: tab.title, url: tab.url, picture: await fetched(info.srcUrl) }));
  }
});

// The console's window and the start page ask through here.
chrome.runtime.onMessage.addListener((message, sender, answer) => {
  if (message.clip) {
    // the console's form: its window may be gone by the time this is done, when a region
    // is dragged over the page, so the toolbar's button says how it went too
    clip(send, message.clip).catch((error) => ({ failed: String(error.message || error) })).then((result) => {
      tell(result);
      answer(result);
    });
  } else if (message.run) {
    run(message.run, send, message.given).then(answer);
  } else if (message.ask) {
    send(message).then(answer);
  } else {
    answer({ failed: 'not understood' });
  }
  return true;  // the answer comes later
});

// The first tab of a launch is drawn before this extension is loaded, so it is the
// browser's own new tab page and not the system's: a new one, which is ours, takes its place.
async function claim() {
  for (const tab of await chrome.tabs.query({})) {
    const address = tab.pendingUrl || tab.url || '';
    if (address.startsWith('chrome://new-tab-page') || address.startsWith('chrome://newtab')) {
      await chrome.tabs.create({ windowId: tab.windowId, index: tab.index, active: tab.active });
      await chrome.tabs.remove(tab.id);
    }
  }
}
// once, when the browser starts; a tab opened later is already ours
chrome.storage.session.get('claimed').then((kept) => {
  if (!kept.claimed) {
    chrome.storage.session.set({ claimed: true });
    setTimeout(claim, 300);
  }
});

// The tabs that are open are told to the host at each change of them, a moment after the
// last one: the launcher of the desktop lists them from what the host keeps. The private
// ones are not told.
let telling = null;
function tabs() {
  clearTimeout(telling);
  telling = setTimeout(async () => {
    const open = (await chrome.tabs.query({})).filter((tab) => !tab.incognito);
    send({
      ask: 'tabs',
      tabs: open.map((tab) => ({ id: tab.id, window: tab.windowId, active: tab.active, title: tab.title || '', url: tab.url || tab.pendingUrl || '' })),
    });
  }, 300);
}
for (const change of [chrome.tabs.onCreated, chrome.tabs.onRemoved, chrome.tabs.onActivated, chrome.tabs.onAttached, chrome.tabs.onDetached, chrome.tabs.onReplaced]) {
  change.addListener(tabs);
}
// a page that loads says many things about itself: only its title and its address matter here
chrome.tabs.onUpdated.addListener((id, change) => {
  if (change.title !== undefined || change.url !== undefined) {
    tabs();
  }
});
tabs();

// The host is started with the browser, so that the system can reach it from then on.
chrome.runtime.onStartup.addListener(() => send({ ask: 'mode' }).then((answer) => answer.mode && mode(answer.mode)));
send({ ask: 'mode' }).then((answer) => answer.mode && chrome.storage.session.set({ mode: answer.mode }));

export { COMMANDS };
