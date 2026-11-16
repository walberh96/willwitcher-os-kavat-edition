#!/usr/bin/env node
// Render docs/video/index.html, the tour narrated by Kavat, to docs/video/kavat-tour.mp4.
//
// usage: NODE_PATH=<folder>/node_modules node tools/make-video.cjs [seconds]
//   seconds   render only the first seconds, for a quick look
//
// Needs Chrome or Chromium, two npm packages installed in any folder outside the repository
// (npm i puppeteer-core ffmpeg-static) and the voice: Supertonic-3, the model the design gives
// Kavat (pip install supertonic, in a virtual environment; it downloads the model on first use).
//   SUPERTONIC   path to the supertonic command, when it is not on the PATH
//   CHROME       path to the browser, when it is not Google Chrome on macOS
//   CACHE        folder that keeps the spoken lines and the picture between runs
//                (by default kavat-video-cache in the system's temporary folder)
//   MUSIC=0      no music and no sound at the chapter cards: the voice alone
//   MUSIC=<file> a piece of music of your own under the voice, in place of the one made here
//   OUT          another file to write, instead of docs/video/kavat-tour.mp4
//   CRF          how hard the picture is squeezed: 27 as it comes, lower is heavier and sharper
//
// How it works: each line of the film is spoken to a file and measured; the page is told how
// long each line lasts and how loud the voice is at every frame; then the page is asked for one
// frame at a time, so the picture never depends on how fast this machine is. The mockup's own
// timers and animations are tied to the film's clock too, so what it animates by itself moves
// at its real speed. The sound is laid under the picture at the end: the voice, a bright tune
// made here sample by sample, which steps back while Kavat speaks, and a breath of noise as each
// chapter card crosses.
//
// The spoken lines and the picture are kept in CACHE, each under a name made from what it was
// made of: a run that changes only the sound does not draw the picture again.
const fs = require('fs');
const os = require('os');
const path = require('path');
const crypto = require('crypto');
const { execFileSync, spawn, spawnSync } = require('child_process');
const puppeteer = require('puppeteer-core');
const ffmpeg = require('ffmpeg-static');

const repo = path.dirname(__dirname);
const film = path.join(repo, 'docs', 'video', 'index.html');
const out = process.env.OUT || path.join(repo, 'docs', 'video', 'kavat-tour.mp4');
const cache = process.env.CACHE || path.join(os.tmpdir(), 'kavat-video-cache');
const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'kavat-video-'));
const FPS = 30, VOICE = 'F2', SPEED = '1.05', STEPS = '12';
const supertonic = process.env.SUPERTONIC || 'supertonic';
const music = process.env.MUSIC !== '0', own = music && process.env.MUSIC && process.env.MUSIC !== '1' ? process.env.MUSIC : '';
const only = Number(process.argv[2]) || 0;
fs.mkdirSync(cache, { recursive: true });
const sha = s => crypto.createHash('sha1').update(s).digest('hex').slice(0, 16);
const ff = (...a) => execFileSync(ffmpeg, ['-y', '-loglevel', 'error', ...a]);

// what the voice should say where the caption would be read wrong
// Kavat is stressed on its last syllable; spelled as it is, the voice stresses the first
const speak = t => t.replace(/\bKavat\b/g, 'Cavat').replace(/\bAUR\b/g, 'A U R').replace(/\bVPN\b/g, 'V P N')
  .replace(/\bUSB\b/g, 'U S B').replace(/\bOBS\b/g, 'O B S');
// seconds of sound in a WAV file, read from its header
const seconds = f => { const b = fs.readFileSync(f), i = b.indexOf('data', 12, 'latin1'); return b.readUInt32LE(i + 4) / b.readUInt32LE(28); };
// how loud a file is on average, in dB, as ffmpeg measures it
const loudness = f => +/mean_volume: (-?[\d.]+) dB/.exec(spawnSync(ffmpeg, ['-i', f, '-af', 'volumedetect', '-f', 'null', '-']).stderr.toString())[1];

// Runs in every frame of the page before its own scripts. While rendering, a frame takes far
// longer to draw than it lasts, so nothing in the mockup may follow the real clock. The film
// keeps its own, window.__vt, in milliseconds; here the mockup's timers, its drawing loops and
// its CSS animations and transitions are tied to that clock instead of the machine's.
const follow = () => {
  if (window === window.top) return;
  const clock = () => { try { return window.top.__vt || 0; } catch (e) { return 0; } };
  const timers = new Map();
  let ids = 1e6;
  const timer = rep => (fn, ms, ...args) => { ms = Math.max(rep ? 1 : 0, +ms || 0); timers.set(++ids, { fn, ms, rep, args, due: clock() + ms }); return ids; };
  window.setInterval = timer(true);
  window.setTimeout = timer(false);
  window.clearInterval = window.clearTimeout = id => { timers.delete(id); };
  const raf = window.requestAnimationFrame.bind(window);
  window.requestAnimationFrame = cb => raf(() => cb(clock()));
  const sync = () => {
    const now = clock();
    for (const [id, t] of [...timers]) {
      for (let n = 0; n < 40 && t.due <= now && timers.has(id); n++) {
        if (t.rep) t.due += t.ms; else timers.delete(id);
        try { t.fn(...t.args); } catch (e) {}
      }
      if (t.rep && t.due <= now) t.due = now + t.ms;        // it fell far behind while nobody saw it
    }
    for (const a of document.getAnimations()) {
      try { if (a.__t0 === undefined) { a.__t0 = now; a.pause(); } a.currentTime = now - a.__t0; } catch (e) {}
    }
    raf(sync);
  };
  raf(sync);
};

// The music: a small tune made here, sample by sample, so that the film needs no file of anyone
// else's. Bright and quick: 116 beats a minute in C major, a plucked run of chord notes in
// sixteenths, a bouncing bass, a soft kick on every beat, a clap on the second and the fourth and
// a hat between them. It opens and closes with the run and the chords alone; `from` and `to` are
// the seconds where the drums come in and where they leave.
const song = (file, total, from, to) => {
  const SR = 44100, N = Math.ceil(total * SR), L = new Float32Array(N), R = new Float32Array(N);
  const beat = 60 / 116, step = beat / 4, bar = beat * 4, hz = m => 440 * 2 ** ((m - 69) / 12);
  let seed = 7; const noise = () => { seed = (seed * 16807) % 2147483647; return seed / 1073741823.5 - 1; };
  // a voice is a function of the seconds since its note began; pan goes from -1, left, to 1
  const play = (t, dur, gain, pan, fn) => {
    const a = Math.round(t * SR), n = Math.min(Math.round(dur * SR), N - a), l = gain * (1 - Math.max(0, pan)), r = gain * (1 + Math.min(0, pan));
    for (let i = 0; i < n; i++) { const v = fn(i / SR); L[a + i] += v * l; R[a + i] += v * r; }
  };
  const pluck = (t, m, gain, pan) => { const w = 2 * Math.PI * hz(m);
    play(t, 0.7, gain, pan, x => Math.min(1, x / 0.004) * Math.exp(-x / 0.17) * (Math.sin(w * x) + 0.4 * Math.exp(-x / 0.06) * Math.sin(2 * w * x) + 0.15 * Math.exp(-x / 0.03) * Math.sin(3 * w * x))); };
  const bell = (t, m, gain, pan) => { const w = 2 * Math.PI * hz(m);
    play(t, 1.6, gain, pan, x => Math.min(1, x / 0.003) * Math.exp(-x / 0.45) * (Math.sin(w * x) + 0.25 * Math.sin(2.76 * w * x) * Math.exp(-x / 0.15))); };
  const bass = (t, m, gain) => { const w = 2 * Math.PI * hz(m);
    play(t, 0.36, gain, 0, x => Math.min(1, x / 0.006) * Math.exp(-x / 0.19) * (Math.sin(w * x) + 0.35 * Math.sin(2 * w * x))); };
  const kick = (t, gain) => { let ph = 0;
    play(t, 0.24, gain, 0, x => { ph += 2 * Math.PI * (46 + 95 * Math.exp(-x / 0.028)) / SR; return Math.exp(-x / 0.085) * Math.sin(ph); }); };
  const hiss = (t, gain, pan, decay, keep) => { let low = 0;                // noise with its low part taken out
    play(t, decay * 6, gain, pan, x => { const v = noise(); low += keep * (v - low); return (v - low) * Math.exp(-x / decay); }); };
  const pad = (t, dur, notes, gain) => notes.forEach((m, j) => { const w = 2 * Math.PI * hz(m);
    play(t, dur + 0.6, gain, j % 2 ? 0.4 : -0.4, x => Math.min(1, x / 0.5) * Math.min(1, (dur + 0.6 - x) / 0.6) * (Math.sin(w * x) + 0.5 * Math.sin(w * 1.004 * x))); });

  // root, third and fifth of each chord, around middle C: I, V, vi, IV and a second line that climbs home
  const C = [60, 64, 67], G = [55, 59, 62], Am = [57, 60, 64], F = [53, 57, 60], Dm = [62, 65, 69], Em = [52, 55, 59];
  const line = [C, G, Am, F, C, G, F, G, Am, F, C, G, Dm, Em, F, G];
  const run = [0, 2, 3, 2, 4, 2, 3, 2, 0, 2, 3, 2, 4, 3, 2, 1];
  const bars = Math.ceil(total / bar);
  for (let n = 0; n < bars; n++) {
    const t = n * bar, [a, b, c] = line[n % line.length], tones = [a + 12, b + 12, c + 12, a + 24, b + 24];
    const full = t >= from && t + bar <= to, last = t + bar > total - 2.2;
    if (last) { pad(t, total - t, [48, 60, 64, 67], 0.07); bell(t, 84, 0.16, 0); pluck(t, 72, 0.22, -0.3); pluck(t + step * 2, 76, 0.2, 0.3); pluck(t + step * 4, 79, 0.2, 0); break; }
    pad(t, bar, [a, b, c], full ? 0.035 : 0.055);
    bell(t, a + 36, full ? 0.07 : 0.1, n % 2 ? 0.5 : -0.5);
    for (let k = 0; k < 16; k++) {
      const at = t + k * step;
      if (full || k % 2 === 0) pluck(at, tones[run[k]], (k % 4 ? 0.15 : 0.2) * (full ? 1 : 0.9), k % 2 ? 0.35 : -0.35);
      if (!full) continue;
      if (k % 4 === 0) kick(at, 0.5);
      if (k === 0 || k === 6 || k === 8 || k === 14) bass(at, a - 24 + (k === 14 ? 7 : 0), 0.34);
      if (k === 4 || k === 12) hiss(at, 0.2, 0.1, 0.05, 0.25);
      if (k % 4 === 2) hiss(at, 0.07, -0.2, 0.02, 0.6);
      if (n % 8 === 7 && k >= 13) hiss(at, 0.13, 0.2, 0.04, 0.25);       // three claps that turn the corner
    }
  }
  // a gentle ceiling, and the loudest sample to nine tenths
  let peak = 0; for (let i = 0; i < N; i++) { L[i] = Math.tanh(L[i] * 1.2); R[i] = Math.tanh(R[i] * 1.2); peak = Math.max(peak, Math.abs(L[i]), Math.abs(R[i])); }
  const wav = Buffer.alloc(44 + N * 4);
  wav.write('RIFF', 0); wav.writeUInt32LE(36 + N * 4, 4); wav.write('WAVEfmt ', 8); wav.writeUInt32LE(16, 16); wav.writeUInt16LE(1, 20); wav.writeUInt16LE(2, 22);
  wav.writeUInt32LE(SR, 24); wav.writeUInt32LE(SR * 4, 28); wav.writeUInt16LE(4, 32); wav.writeUInt16LE(16, 34); wav.write('data', 36); wav.writeUInt32LE(N * 4, 40);
  for (let i = 0; i < N; i++) { wav.writeInt16LE(Math.round(L[i] / peak * 0.9 * 32767), 44 + i * 4); wav.writeInt16LE(Math.round(R[i] / peak * 0.9 * 32767), 46 + i * 4); }
  fs.writeFileSync(file, wav);
};

(async () => {
  const browser = await puppeteer.launch({
    executablePath: process.env.CHROME || '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome',
    // the last one gives the browser room to keep every picture drawn: with less, it drops parts of them
    headless: 'new', args: ['--hide-scrollbars', '--allow-file-access-from-files', '--force-gpu-mem-available-mb=8192'],
    defaultViewport: { width: 1920, height: 1080 }, protocolTimeout: 600000,
  });
  const page = await browser.newPage();
  await page.evaluateOnNewDocument(follow);
  await page.goto('file://' + film + '#render');
  await page.evaluate(() => window.ready);

  // 1. speak every line and measure it
  const lines = await page.evaluate(() => window.cues().map(c => c.say));
  const wavs = lines.map((text, i) => {
    const f = path.join(cache, `line-${sha([speak(text), VOICE, SPEED, STEPS].join('|'))}.wav`);
    if (!fs.existsSync(f)) {
      process.stdout.write(`  speaking line ${i + 1} of ${lines.length}\r`);
      execFileSync(supertonic, ['tts', speak(text), '-o', f, '--voice', VOICE, '--lang', 'en', '--speed', SPEED, '--steps', STEPS], { stdio: 'ignore' });
    }
    return f;
  });
  const durations = wavs.map(seconds);
  const total = await page.evaluate(d => window.setDurations(d), durations);
  const cues = await page.evaluate(() => window.cues());
  const length = only ? Math.min(only, total) : total;
  console.log(`film: ${total.toFixed(1)} s · ${length.toFixed(1)} s at ${FPS} fps`);

  // 2. the voice: every line delayed to its cue, mixed into one file
  const voice = path.join(tmp, 'voice.wav');
  ff(...wavs.flatMap(f => ['-i', f]), '-filter_complex',
    wavs.map((_, i) => `[${i}]adelay=${Math.round(cues[i].at * 1000)}:all=1[a${i}]`).join(';')
    + ';' + wavs.map((_, i) => `[a${i}]`).join('') + `amix=inputs=${wavs.length}:normalize=0,apad=whole_dur=${total}[v]`,
    '-map', '[v]', '-t', String(total), '-ar', '44100', '-ac', '2', voice);

  // 3. how loud the voice is at every frame, for Kavat's whiskers
  const pcm = execFileSync(ffmpeg, ['-loglevel', 'error', '-i', voice, '-ac', '1', '-ar', '8000', '-f', 's16le', '-'], { maxBuffer: 1 << 28 });
  const raw = [];
  for (let n = 0; n < Math.ceil(total * FPS); n++) {
    const a = Math.floor(n * 8000 / FPS), b = Math.min(pcm.length / 2, Math.floor((n + 1) * 8000 / FPS));
    let s = 0; for (let i = a; i < b; i++) { const v = pcm.readInt16LE(i * 2); s += v * v; }
    raw.push(b > a ? Math.sqrt(s / (b - a)) : 0);
  }
  const top = [...raw].sort((x, y) => x - y)[Math.floor(raw.length * 0.97)] || 1;
  let held = 0;
  const levels = raw.map(v => { held = Math.max(Math.min(1, v / top), held * 0.78); return +held.toFixed(3); });
  await page.evaluate(l => window.setLevels(l), levels);

  // 4. the sound: the voice, and under it the music and a breath of noise at each card.
  // The music steps back while Kavat speaks and comes forward in the gaps.
  const sound = path.join(tmp, 'sound.wav');
  if (music) {
    let tune = path.join(tmp, 'tune.wav');
    if (own) ff('-stream_loop', '-1', '-i', own, '-t', String(total), '-ac', '2', '-ar', '44100', tune);
    else song(tune, total, cues[1].card, cues[cues.length - 1].at + durations[durations.length - 1] + 0.4);
    const gain = -25 - loudness(tune);
    const cards = cues.filter(c => c.card !== null).map(c => Math.round(c.card * 1000));
    ff('-i', voice, '-i', tune, '-f', 'lavfi', '-i', 'anoisesrc=color=pink:d=0.95:a=0.5:r=44100', '-filter_complex',
      `[0]asplit[say][key];[1]volume=${gain.toFixed(1)}dB,afade=t=in:d=0.8,afade=t=out:st=${(total - 2.5).toFixed(2)}:d=2.5[tune];`
      + `[tune][key]sidechaincompress=threshold=0.02:ratio=3.5:attack=25:release=450:makeup=1[under];`
      + `[2]highpass=f=350,lowpass=f=3200,afade=t=in:d=0.42,afade=t=out:st=0.42:d=0.5,volume=-19dB,asplit=${cards.length}`
      + cards.map((_, i) => `[w${i}]`).join('') + ';' + cards.map((ms, i) => `[w${i}]adelay=${ms}:all=1[x${i}]`).join(';')
      + `;[say][under]${cards.map((_, i) => `[x${i}]`).join('')}amix=inputs=${cards.length + 2}:normalize=0,alimiter=limit=0.95[s]`,
      '-map', '[s]', '-t', String(total), sound);
  } else fs.copyFileSync(voice, sound);

  // 5. the picture, frame by frame, straight into the encoder; kept, unless it is only a quick look
  const key = sha([fs.readFileSync(film), fs.readFileSync(path.join(repo, 'docs', 'mockup', 'index.html')), JSON.stringify(durations), FPS].join('|'));
  const picture = only ? path.join(tmp, 'picture.mp4') : path.join(cache, `picture-${key}.mp4`);
  if (only || !fs.existsSync(picture)) {
    const part = picture + '.part.mp4';
    const enc = spawn(ffmpeg, ['-y', '-loglevel', 'error', '-f', 'image2pipe', '-framerate', String(FPS), '-c:v', 'mjpeg', '-i', '-',
      '-c:v', 'libx264', '-preset', 'slow', '-crf', process.env.CRF || '27', '-pix_fmt', 'yuv420p', '-movflags', '+faststart', part], { stdio: ['pipe', 'inherit', 'inherit'] });
    const frames = Math.round(length * FPS), began = Date.now();
    for (let n = 0; n < frames; n++) {
      await page.evaluate(t => { window.__vt = t * 1000; window.setT(t); }, n / FPS);
      const jpg = await page.screenshot({ type: 'jpeg', quality: 94 });
      if (!enc.stdin.write(jpg)) await new Promise(r => enc.stdin.once('drain', r));
      if (n % 150 === 0) console.log(`  ${Math.round(n / frames * 100)}% · ${((Date.now() - began) / 60000).toFixed(1)} min`);
    }
    enc.stdin.end();
    await new Promise(r => enc.on('close', r));
    fs.renameSync(part, picture);
  } else console.log('  the picture is the one kept from the last run');
  await browser.close();

  // 6. the two together
  const peak = +/max_volume: (-?[\d.]+) dB/.exec(spawnSync(ffmpeg, ['-i', sound, '-af', 'volumedetect', '-f', 'null', '-']).stderr.toString())[1];
  ff('-i', picture, '-i', sound, '-t', String(length), '-c:v', 'copy', '-af', `volume=${(-1.5 - peak).toFixed(1)}dB`, '-c:a', 'aac', '-b:a', '160k', '-movflags', '+faststart', out);
  fs.rmSync(tmp, { recursive: true, force: true });
  console.log('written:', path.relative(repo, out), `· ${(fs.statSync(out).size / 1e6).toFixed(1)} MB`);
})().catch(e => { console.error(e); process.exit(1); });
