// YouTube without its Shorts: the shelf of the home page, the entry at the side, the tab of
// a channel, the ones among the subscriptions and the search. What is hidden is what the
// list gijsdev/ublock-hide-yt-shorts names; shorts-rules.js, made from it when the package
// is built, has it as KAVAT_SHORTS. They are hidden from the first moment of a page, and
// the system is asked whether they are wanted back: `kavat-browser shorts on`.
(() => {
  const rules = KAVAT_SHORTS[location.hostname] || [];
  if (!rules.length) {
    return;
  }
  const MARK = 'kavat-shorts';
  const hide = '{ display: none !important; }';
  // the rules a style sheet cannot keep by itself: for a path of the site, or for the words in an element
  const watched = rules.some((rule) => rule.path !== undefined || rule.text !== undefined);
  let hidden = true;
  let style = null;
  let path = null;
  let waiting = null;

  function matches(rule) {
    try {
      return rule.path === undefined || new RegExp(rule.path, rule.path_flags || '').test(location.pathname);
    } catch (error) {
      return false;
    }
  }

  // The style sheet: each rule on its own, so that one the browser does not understand
  // takes no other with it.
  function sheet() {
    path = location.pathname;
    if (!style) {
      style = document.createElement('style');
    }
    style.textContent = [`[${MARK}] ${hide}`, ...rules.filter((rule) => rule.css && matches(rule)).map((rule) => `${rule.css} ${hide}`)].join('\n');
    if (!style.isConnected) {
      (document.head || document.documentElement).append(style);
    }
  }

  // The elements hidden for their words are marked, and the mark is what the style sheet hides.
  function words() {
    const found = new Set();
    for (const rule of rules) {
      if (rule.text === undefined || !matches(rule)) {
        continue;
      }
      try {
        const wanted = new RegExp(rule.text, rule.flags || '');
        for (const element of document.querySelectorAll(rule.find)) {
          const one = rule.hide ? element.closest(rule.hide) : element;
          if (one && wanted.test(element.textContent.trim())) {
            found.add(one);
          }
        }
      } catch (error) { /* a rule the browser does not understand hides nothing */ }
    }
    for (const element of document.querySelectorAll(`[${MARK}]`)) {
      if (!found.has(element)) {
        element.removeAttribute(MARK);
      }
    }
    for (const element of found) {
      element.setAttribute(MARK, '');
    }
  }

  function apply() {
    waiting = null;
    if (!hidden) {
      if (style) {
        style.remove();
      }
      return;
    }
    if (path !== location.pathname || !style || !style.isConnected) {
      sheet();
    }
    words();
  }

  // YouTube changes its page without loading another: looked at again a moment after each change
  function later() {
    if (hidden && waiting === null) {
      waiting = setTimeout(apply, 150);
    }
  }

  async function ask() {
    let shown = false;
    try {
      shown = (await chrome.runtime.sendMessage({ ask: 'shorts' })).shorts === true;
    } catch (error) { /* with no answer they stay hidden */ }
    if (hidden === shown) {
      hidden = !shown;
      apply();
    }
  }

  apply();
  if (watched) {
    new MutationObserver(later).observe(document, { childList: true, subtree: true });
  }
  ask();
  // the switch may have been pressed while the tab was out of view
  document.addEventListener('visibilitychange', () => document.visibilityState === 'visible' && ask());
})();
