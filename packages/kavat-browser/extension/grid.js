// An album drawn as a grid of its sites, each with its thumbnail: used by the albums' page
// and by the start page. The sites and their pictures are the notes' own files, which the
// host reads; a picture is asked for by itself, when its card is drawn.

function ask(message) {
  return chrome.runtime.sendMessage(message);
}

// One site of an album as a card. `more` adds the buttons of the albums' page.
function card(site, album, more, again) {
  const link = document.createElement('a');
  link.className = 'card';
  link.href = site.url;
  const picture = document.createElement('div');
  picture.className = 'picture';
  if (site.picture) {
    ask({ ask: 'album-picture', file: site.picture }).then((answer) => {
      if (answer && answer.picture) {
        picture.style.backgroundImage = `url("${answer.picture}")`;
      }
    });
  } else {
    // a site with no thumbnail shows its icon
    const icon = document.createElement('img');
    icon.src = `chrome-extension://${chrome.runtime.id}/_favicon/?pageUrl=${encodeURIComponent(site.url)}&size=32`;
    icon.alt = '';
    picture.append(icon);
  }
  const name = document.createElement('span');
  name.className = 'name';
  name.textContent = site.title || site.url;
  const where = document.createElement('span');
  where.className = 'where muted';
  try {
    where.textContent = new URL(site.url).hostname;
  } catch (error) {
    where.textContent = site.url;
  }
  link.append(picture, name, where);
  if (more) {
    const buttons = document.createElement('span');
    buttons.className = 'buttons';
    const button = (text, title, message) => {
      const one = document.createElement('button');
      one.textContent = text;
      one.title = title;
      one.addEventListener('click', async (event) => {
        event.preventDefault();
        await ask(message);
        again();
      });
      buttons.append(one);
    };
    if (album === 'read-later') {
      button('✓', 'Read: put it away', { ask: 'album-read', file: site.file });
    }
    button('✕', 'Take it out of the album', { ask: 'album-remove', name: album, url: site.url });
    link.append(buttons);
  }
  return link;
}

// Draw an album into a container; says how many sites it has.
export async function grid(container, album, more = false) {
  const answer = await ask({ ask: 'album', name: album });
  const sites = (answer && answer.sites) || [];
  container.replaceChildren(...sites.map((site) => card(site, album, more, () => grid(container, album, more))));
  return sites;
}

export async function albums() {
  const answer = await ask({ ask: 'albums' });
  return (answer && answer.albums) || [];
}
