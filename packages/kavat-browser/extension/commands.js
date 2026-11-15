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
      return send({ ask: 'clip', title: tab.title, url: tab.url, quote: await selection(tab) });
    },
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
