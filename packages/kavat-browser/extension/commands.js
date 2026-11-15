// The commands of the console: what each is called, and what it does. They are used by the
// console's window, by the keys the manifest names, and by the system, which can ask for one
// through the host. A command of your own, from ~/.config/kavat/browser/commands.toml, is
// one of these by its name with something to give it, or an address to open.

async function current() {
  const [tab] = await chrome.tabs.query({ active: true, lastFocusedWindow: true });
  return tab;
}

// what is selected in the page, when the page lets itself be read
async function selection(tab) {
  try {
    const [found] = await chrome.scripting.executeScript({ target: { tabId: tab.id }, func: () => String(getSelection()) });
    return (found && found.result) || '';
  } catch (error) {
    return '';
  }
}

// a picture of what the tab shows; nothing when the browser will not give one
async function shot(tab) {
  try {
    return await chrome.tabs.captureVisibleTab(tab.windowId, { format: 'jpeg', quality: 70 });
  } catch (error) {
    return '';
  }
}

export const COMMANDS = {
  'new tab': { about: 'Open a new tab', run: () => chrome.tabs.create({}) },
  'close other tabs': {
    about: 'Close every tab of this window but this one',
    run: async () => {
      const tabs = await chrome.tabs.query({ currentWindow: true, active: false, pinned: false });
      await chrome.tabs.remove(tabs.map((tab) => tab.id));
    },
  },
  'duplicate tab': { about: 'This tab, twice', run: async () => chrome.tabs.duplicate((await current()).id) },
  'pin tab': {
    about: 'Pin this tab, or let it go',
    run: async () => {
      const tab = await current();
      await chrome.tabs.update(tab.id, { pinned: !tab.pinned });
    },
  },
  'mute tab': {
    about: 'Silence this tab, or let it sound',
    run: async () => {
      const tab = await current();
      await chrome.tabs.update(tab.id, { muted: !tab.mutedInfo.muted });
    },
  },
  'zen mode': {
    about: 'The page and nothing else; again to come back',
    run: async () => {
      const window = await chrome.windows.getLastFocused();
      await chrome.windows.update(window.id, { state: window.state === 'fullscreen' ? 'normal' : 'fullscreen' });
    },
  },
  'sleep other tabs': {
    about: 'Free the memory of the tabs that are not in view',
    run: async () => {
      const tabs = await chrome.tabs.query({ active: false, discarded: false });
      for (const tab of tabs) {
        try { await chrome.tabs.discard(tab.id); } catch (error) { /* a tab that cannot sleep stays awake */ }
      }
    },
  },
  'move tab to new window': { about: 'This tab in a window of its own', run: async () => chrome.windows.create({ tabId: (await current()).id }) },
  'private window': { about: 'A window that keeps nothing', run: () => chrome.windows.create({ incognito: true }) },
  'copy link': {
    about: 'The address of this page, as a Markdown link',
    run: async () => {
      const tab = await current();
      await chrome.scripting.executeScript({
        target: { tabId: tab.id },
        func: (text) => navigator.clipboard.writeText(text),
        args: [`[${tab.title}](${tab.url})`],
      });
    },
  },
  'save to notes': {
    about: 'This page in the notes, with what is selected as a quote',
    run: async (send) => {
      const tab = await current();
      return send({ ask: 'clip', title: tab.title, url: tab.url, quote: await selection(tab), picture: await shot(tab) });
    },
  },
  'add to album': {
    about: 'This site in an album: type its name after the command, or it goes to favourites',
    takes: 'album',
    run: async (send, album) => {
      const tab = await current();
      return send({ ask: 'album-add', name: album || 'favourites', title: tab.title, url: tab.url, picture: await shot(tab) });
    },
  },
  'read later': {
    about: 'This page in the album of what waits to be read',
    run: async (send) => {
      const tab = await current();
      return send({ ask: 'album-add', name: 'read-later', title: tab.title, url: tab.url, picture: await shot(tab) });
    },
  },
  'tabs to album': {
    about: "Every tab of this window in an album: type its name after the command, or it is today's",
    takes: 'album',
    run: async (send, album) => {
      const name = album || `tabs-${new Date().toISOString().slice(0, 10)}`;
      const tabs = (await chrome.tabs.query({ currentWindow: true })).filter((tab) => /^https?:/.test(tab.url || ''));
      for (const tab of tabs) {
        const answer = await send({ ask: 'album-add', name, title: tab.title, url: tab.url, picture: tab.active ? await shot(tab) : '' });
        if (answer.failed) {
          return answer;
        }
      }
      return { said: `${tabs.length} tabs in ${name}` };
    },
  },
  'albums': { about: 'Your albums of sites', run: () => chrome.tabs.create({ url: chrome.runtime.getURL('albums.html') }) },
  'open album': {
    about: 'Every site of an album, each in a tab: type its name after the command',
    takes: 'album',
    run: async (send, album) => {
      const answer = await send({ ask: 'album', name: album || 'favourites' });
      for (const site of answer.sites || []) {
        await chrome.tabs.create({ url: site.url, active: false });
      }
      return { said: `${(answer.sites || []).length} sites opened` };
    },
  },
  'notes about this site': {
    about: 'What your notes already have from this site, in a panel at the side',
    // the browser allows it only at a press of yours: from the console and from the right
    // button it opens; asked for from outside the browser, it is refused, and that is said
    run: async () => chrome.sidePanel.open({ windowId: (await chrome.windows.getLastFocused()).id }),
  },
  'downloads': { about: 'The downloads folder, in Yazi', run: (send) => send({ ask: 'folder', which: 'downloads' }) },
  'history': { about: 'Where you have been', run: () => chrome.tabs.create({ url: 'chrome://history' }) },
  'extensions': { about: 'The extensions', run: () => chrome.tabs.create({ url: 'chrome://extensions' }) },
  'settings': { about: "The browser's settings", run: () => chrome.tabs.create({ url: 'chrome://settings' }) },
  'open': { about: 'Open an address', hidden: true, run: (send, address) => chrome.tabs.create({ url: address }) },
};

// Run a command by its name; `given` is what a command of your own gives it.
export async function run(name, send, given) {
  const command = COMMANDS[name];
  if (!command) {
    return { failed: `there is no command called ${name}` };
  }
  try {
    return (await command.run(send, given)) || {};
  } catch (error) {
    return { failed: String(error.message || error) };
  }
}
