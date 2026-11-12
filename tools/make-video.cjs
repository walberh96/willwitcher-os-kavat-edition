#!/usr/bin/env node
// Render docs/video/index.html, the tour narrated by Kavat, to docs/video/kavat-tour.mp4.
//
// usage: NODE_PATH=<folder>/node_modules node tools/make-video.cjs [seconds]
//   seconds   render only the first seconds, for a quick look
//
// Needs macOS (the voice is `say`), Google Chrome, and two npm packages installed in any
// folder outside the repository:  npm i puppeteer-core ffmpeg-static
//
// How it works: each line of the film is spoken to a file and measured; the page is told how
// long each line lasts; then the page is asked for one frame at a time, so the picture never
// depends on how fast this machine is. The voice is laid under the picture at the end.
const fs = require('fs');
const os = require('os');
const path = require('path');
const { execFileSync, spawn } = require('child_process');
const puppeteer = require('puppeteer-core');
const ffmpeg = require('ffmpeg-static');

const repo = path.dirname(__dirname);
const out = path.join(repo, 'docs', 'video', 'kavat-tour.mp4');
const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'kavat-video-'));
const FPS = 30, VOICE = 'Samantha', RATE = '172';
const only = Number(process.argv[2]) || 0;
// what the voice should say where the caption would be read wrong
const speak = t => t.replace(/\bAUR\b/g, 'A U R').replace(/\bkav\b/g, 'cav').replace(/\bISO\b/g, 'I S O').replace(/Yazi/g, 'Yah-zee');

(async () => {
  const browser = await puppeteer.launch({
    executablePath: '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome',
    headless: 'new', args: ['--hide-scrollbars', '--allow-file-access-from-files'],
    defaultViewport: { width: 1920, height: 1080 },
  });
  const page = await browser.newPage();
  await page.goto('file://' + path.join(repo, 'docs', 'video', 'index.html') + '#render');
  await page.evaluate(() => window.ready);

  // 1. speak every line and measure it
  const lines = await page.evaluate(() => window.cues().map(c => c.say));
  const durations = lines.map((text, i) => {
    const f = path.join(tmp, `line${i}.aiff`);
    execFileSync('say', ['-v', VOICE, '-r', RATE, '-o', f, speak(text)]);
    const info = execFileSync('afinfo', [f]).toString();
    return Number(/estimated duration: ([\d.]+)/.exec(info)[1]);
  });
  const total = await page.evaluate(d => window.setDurations(d), durations);
  const cues = await page.evaluate(() => window.cues().map(c => c.at));
  const length = only ? Math.min(only, total) : total;
  console.log(`film: ${total.toFixed(1)} s · rendering ${length.toFixed(1)} s at ${FPS} fps`);

  // 2. the voice track: every line delayed to its cue, mixed into one file
  const voice = path.join(tmp, 'voice.wav');
  const inputs = lines.flatMap((_, i) => ['-i', path.join(tmp, `line${i}.aiff`)]);
  const graph = lines.map((_, i) => `[${i}]adelay=${Math.round(cues[i] * 1000)}:all=1[a${i}]`).join(';')
    + ';' + lines.map((_, i) => `[a${i}]`).join('') + `amix=inputs=${lines.length}:normalize=0,apad=whole_dur=${total}[v]`;
  execFileSync(ffmpeg, ['-y', '-loglevel', 'error', ...inputs, '-filter_complex', graph, '-map', '[v]', '-t', String(total), voice]);

  // 3. the picture, frame by frame, straight into the encoder
  const enc = spawn(ffmpeg, ['-y', '-loglevel', 'error', '-f', 'image2pipe', '-framerate', String(FPS), '-c:v', 'mjpeg', '-i', '-',
    '-i', voice, '-t', String(length), '-c:v', 'libx264', '-preset', 'slow', '-crf', '26', '-pix_fmt', 'yuv420p',
    '-c:a', 'aac', '-b:a', '128k', '-movflags', '+faststart', out], { stdio: ['pipe', 'inherit', 'inherit'] });
  const frames = Math.round(length * FPS);
  for (let n = 0; n < frames; n++) {
    await page.evaluate(t => window.setT(t), n / FPS);
    const jpg = await page.screenshot({ type: 'jpeg', quality: 93 });
    if (!enc.stdin.write(jpg)) await new Promise(r => enc.stdin.once('drain', r));
    if (n % 150 === 0) console.log(`  ${Math.round(n / frames * 100)}%`);
  }
  enc.stdin.end();
  await new Promise(r => enc.on('close', r));
  await browser.close();
  fs.rmSync(tmp, { recursive: true, force: true });
  console.log('written:', path.relative(repo, out), `· ${(fs.statSync(out).size / 1e6).toFixed(1)} MB`);
})().catch(e => { console.error(e); process.exit(1); });
