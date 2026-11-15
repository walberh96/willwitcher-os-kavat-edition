// The system theme on the extension's own pages: the host is asked for the colours in
// force and they go into the variables style.css draws with. They are kept from one page
// to the next, so a page opens already in its colours, and asked for again each time, so a
// change of theme shows on the next page opened.
const NAMES = { bg: '--bg', surface: '--surface', surface2: '--surface2', text: '--text', muted: '--muted', border: '--border', accent: '--accent', on_accent: '--on-accent' };

function wear(theme) {
  for (const [token, variable] of Object.entries(NAMES)) {
    if (/^#[0-9a-f]{6}$/i.test(theme[token] || '')) {
      document.documentElement.style.setProperty(variable, theme[token]);
    }
  }
  if (/^[0-9.]+$/.test(theme.radius || '')) {
    document.documentElement.style.setProperty('--radius', `${theme.radius}px`);
  }
  if (theme.mode === 'light' || theme.mode === 'dark') {
    document.documentElement.style.colorScheme = theme.mode;
  }
}

export async function theme() {
  const kept = await chrome.storage.local.get('theme');
  if (kept.theme) {
    wear(kept.theme);
  }
  const answer = await chrome.runtime.sendMessage({ ask: 'theme' });
  if (answer && answer.theme) {
    wear(answer.theme);
    chrome.storage.local.set({ theme: answer.theme });
  }
}
