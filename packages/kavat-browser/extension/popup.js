// The command console's window: the browser's own commands and yours, filtered as you
// type; the arrows move, Enter runs the one that is lit, Esc closes.
import { COMMANDS } from './commands.js';
import { theme } from './theme.js';

const find = document.getElementById('find');
const list = document.getElementById('list');
const said = document.getElementById('said');
let all = [];    // each: name, about, run (the name of a built-in), given
let shown = [];
let lit = 0;
let here = null;  // the browser window this console was opened over

function draw() {
  const typed = find.value.toLowerCase().trimStart();
  const words = typed.split(/\s+/).filter(Boolean);
  shown = all.filter((command) => words.every((word) => `${command.name} ${command.about}`.toLowerCase().includes(word)));
  // a command that takes something: what is typed after its name is given to it
  for (const command of all) {
    if (command.takes && typed.startsWith(`${command.name} `) && typed.length > command.name.length + 1) {
      shown = [{ ...command, given: find.value.trimStart().slice(command.name.length + 1).trim(), about: `${command.takes}: ${find.value.trimStart().slice(command.name.length + 1).trim()}` }];
    }
  }
  lit = Math.min(lit, Math.max(0, shown.length - 1));
  list.replaceChildren(...shown.map((command, at) => {
    const item = document.createElement('li');
    item.className = at === lit ? 'on' : '';
    const name = document.createElement('b');
    name.textContent = command.name;
    const about = document.createElement('span');
    about.className = 'muted';
    about.textContent = command.about;
    item.append(name, about);
    item.addEventListener('click', () => { lit = at; go(); });
    return item;
  }));
  const on = list.querySelector('.on');
  if (on) {
    on.scrollIntoView({ block: 'nearest' });
  }
}

async function go() {
  const command = shown[lit];
  if (!command) {
    return;
  }
  // The browser opens a panel at its side only at a press of yours, and this window is
  // where the press is: the panel is opened from here, not through the background.
  if (command.run === 'notes about this site' && here !== null) {
    await chrome.sidePanel.open({ windowId: here });
    window.close();
    return;
  }
  const result = await chrome.runtime.sendMessage({ run: command.run, given: command.given });
  if (result && result.failed) {
    said.className = 'said failed';
    said.textContent = result.failed;
  } else if (result && result.said) {
    said.className = 'said muted';
    said.textContent = result.said;
    setTimeout(() => window.close(), 1200);
  } else {
    window.close();
  }
}

find.addEventListener('input', () => { lit = 0; draw(); });
find.addEventListener('keydown', (event) => {
  if (event.key === 'ArrowDown') { lit = Math.min(lit + 1, shown.length - 1); draw(); event.preventDefault(); }
  else if (event.key === 'ArrowUp') { lit = Math.max(lit - 1, 0); draw(); event.preventDefault(); }
  else if (event.key === 'Enter') { go(); }
  else if (event.key === 'Escape') { window.close(); }
});

async function start() {
  theme();
  chrome.windows.getCurrent().then((found) => { here = found.id; });
  all = Object.entries(COMMANDS).filter(([, command]) => !command.hidden).map(([name, command]) => ({ name, about: command.about, run: name, takes: command.takes }));
  draw();
  // your own, from the file the host reads
  const answer = await chrome.runtime.sendMessage({ ask: 'commands' });
  for (const yours of (answer && answer.commands) || []) {
    if (yours.name && (yours.open || COMMANDS[yours.run])) {
      all.push({ name: yours.name, about: yours.about || (yours.open ? `Open ${yours.open}` : `Yours: ${yours.run}`), run: yours.open ? 'open' : yours.run, given: yours.open || yours.given });
    }
  }
  draw();
}
start();
