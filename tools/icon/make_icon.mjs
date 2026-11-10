// Generates every PromptShelf icon from one OKLCH source.
// The mark: a slash command that leans on a text cursor, both standing on a shelf.
// Light appearance: paper tile, ink slash, amber cursor. Dark appearance: dark tile, paper slash, light amber cursor.
//
// Writes:
//   docs/design/logo/*.svg                    vector masters: light and dark icons, one-colour mark, menu bar icon
//   app/src-tauri/icons/AppIcon.icon/         Icon Composer package for macOS 26: light, dark, clear and tinted
//   app/src-tauri/icons/icon.icns             fallback for macOS before 26, compiled by actool (Xcode only)
//   app/src-tauri/icons/*.png, icon.ico       Windows and Linux icons
//   app/src-tauri/icons/tray.svg, tray.png    menu bar template image (macOS tints it)
//   raycast/assets/icon.png, icon@dark.png    Raycast extension icon for the light and dark themes
//
// Usage: node tools/icon/make_icon.mjs [--preview <dir>]
//   --preview renders the macOS icon in each appearance with ictool into <dir>.
// Needs rsvg-convert (librsvg). The macOS steps need Xcode; on other systems the script skips them.
import { spawnSync } from 'node:child_process';
import { mkdirSync, readFileSync, renameSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = join(dirname(fileURLToPath(import.meta.url)), '../..');
const OUT_LOGO = join(ROOT, 'docs/design/logo');
const OUT_APP = join(ROOT, 'app/src-tauri/icons');
const OUT_PACKAGE = join(OUT_APP, 'AppIcon.icon');
const OUT_RAYCAST = join(ROOT, 'raycast/assets');
const OUT_UI = join(ROOT, 'app/src/assets'); // Settings → About

// ---- Colours (OKLCH; neutrals tinted toward hue 265 like the app palette) ----
const PAPER = [0.965, 0.007, 265];       // light tile, dark slash
const INK = [0.22, 0.014, 265];          // light slash
const DARK = [0.24, 0.014, 265];         // dark tile
const AMBER = [0.64, 0.16, 60];          // cursor on paper
const AMBER_LIGHT = [0.82, 0.15, 80];    // cursor on the dark tile
const SHELF = [0.78, 0.012, 265];        // shelf on paper
const SHELF_DARK = [0.58, 0.012, 265];   // shelf on the dark tile
const EDGE = [0.86, 0.01, 265];          // thin edge around the flat light icon, so it shows on white

// OKLCH → gamma-encoded sRGB 0…1 (OKLab matrices, gamut-clamped). Icon tools don't read OKLCH.
function srgb([L, C, h]) {
  const a = C * Math.cos((h * Math.PI) / 180), b = C * Math.sin((h * Math.PI) / 180);
  const l = (L + 0.3963377774 * a + 0.2158037573 * b) ** 3;
  const m = (L - 0.1055613458 * a - 0.0638541728 * b) ** 3;
  const s = (L - 0.0894841775 * a - 1.291485548 * b) ** 3;
  return [
    4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
    -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
    -0.0041960863 * l - 0.7034186147 * m + 1.707614701 * s,
  ].map((x) => {
    x = Math.max(0, Math.min(1, x));
    return x <= 0.0031308 ? 12.92 * x : 1.055 * x ** (1 / 2.4) - 0.055;
  });
}
const hex = (c) => '#' + srgb(c).map((v) => Math.round(v * 255).toString(16).padStart(2, '0')).join('');
const icColor = (c) => 'srgb:' + [...srgb(c), 1].map((v) => v.toFixed(5)).join(',');
const okl = ([L, C, h]) => `oklch(${L} ${C} ${h})`;

// ---- Geometry, on a 1024 square (the macOS icon canvas; the system applies the rounded mask) ----
// Every shape is a filled path: actool fills each path in a layer, so strokes would turn into wedges.
const GLYPH = {
  slash: { a: [378, 690], b: [568, 306], r: 52 },   // capsule; its top stops 14 units before the cursor
  cursor: { x: 634, y: 264, w: 82, h: 480, r: 16 }, // stands on the shelf
  shelf: { x: 202, y: 744, w: 620, h: 38, r: 19 },  // glyph spans y 254…782, a little above centre
};
const f = (v) => +v.toFixed(2);
function capsule([ax, ay], [bx, by], r) {
  const len = Math.hypot(bx - ax, by - ay), nx = (-(by - ay) / len) * r, ny = ((bx - ax) / len) * r;
  return `M${f(ax + nx)} ${f(ay + ny)}L${f(bx + nx)} ${f(by + ny)}A${r} ${r} 0 0 0 ${f(bx - nx)} ${f(by - ny)}`
    + `L${f(ax - nx)} ${f(ay - ny)}A${r} ${r} 0 0 0 ${f(ax + nx)} ${f(ay + ny)}Z`;
}
function roundRect({ x, y, w, h, r }) {
  return `M${f(x + r)} ${f(y)}H${f(x + w - r)}A${r} ${r} 0 0 1 ${f(x + w)} ${f(y + r)}V${f(y + h - r)}`
    + `A${r} ${r} 0 0 1 ${f(x + w - r)} ${f(y + h)}H${f(x + r)}A${r} ${r} 0 0 1 ${f(x)} ${f(y + h - r)}`
    + `V${f(y + r)}A${r} ${r} 0 0 1 ${f(x + r)} ${f(y)}Z`;
}
// Scale the glyph into another square: s is the factor, (dx, dy) the offset; minH keeps thin shapes visible.
function scaled(s, dx, dy, minH = 0) {
  const p = ([x, y]) => [x * s + dx, y * s + dy];
  const rect = (q) => {
    const h = Math.max(q.h * s, minH);
    return { x: q.x * s + dx, y: q.y * s + dy, w: q.w * s, h, r: Math.min(q.r * s, h / 2) };
  };
  return {
    slash: capsule(p(GLYPH.slash.a), p(GLYPH.slash.b), GLYPH.slash.r * s),
    cursor: roundRect(rect(GLYPH.cursor)),
    shelf: roundRect(rect(GLYPH.shelf)),
  };
}
const FULL = scaled(1, 0, 0);

// ---- SVG ----
const svgDoc = (size, body, note) => `<svg xmlns="http://www.w3.org/2000/svg" width="${size}" height="${size}" viewBox="0 0 ${size} ${size}">
<!-- PromptShelf ${note}. Generated by tools/icon/make_icon.mjs; edit the script, not this file. -->
${body}
</svg>
`;
const path = (d, c, note) => `<path d="${d}" fill="${hex(c)}"/>${note ? ` <!-- ${note} ${okl(c)} -->` : ''}`;
const marks = (slash, cursor, shelf, g = FULL) => [
  path(g.shelf, shelf, 'shelf'), path(g.slash, slash, 'slash'), path(g.cursor, cursor, 'cursor'),
].join('\n');
// Flat icon for Windows, Linux, Raycast and the docs: a rounded tile with the glyph, no system mask.
const TILE = { x: 40, y: 40, w: 944, h: 944, r: 212 };
const flatLight = [
  path(roundRect(TILE), EDGE, 'edge'),
  path(roundRect({ x: 48, y: 48, w: 928, h: 928, r: 204 }), PAPER, 'tile'),
  marks(INK, AMBER, SHELF),
].join('\n');
const flatDark = [path(roundRect(TILE), DARK, 'tile'), marks(PAPER, AMBER_LIGHT, SHELF_DARK)].join('\n');
const mono = `<path d="${FULL.shelf}${FULL.slash}${FULL.cursor}" fill="#000"/>`;
// Menu bar: the glyph fitted to 64 px, with a shelf at least 5 px high so it survives the 16-pt menu bar.
const TRAY = scaled(56 / 620, 4 - (202 * 56) / 620, 8 - (254 * 56) / 620, 5);
const tray = `<path d="${TRAY.shelf}${TRAY.slash}${TRAY.cursor}" fill="#000"/>`;

// ---- macOS: Icon Composer package ----
// Fill: paper, pinned to the dark tile in the dark appearance. Each mark is its own glass layer with an explicit
// sRGB fill per appearance. In tinted and clear, every layer is plain white and the system tints it.
const TINTED_WHITE = { appearance: 'tinted', value: { solid: 'extended-gray:1.00000,1.00000' } };
const layer = (name, light, dark) => ({
  'fill-specializations': [{ value: { solid: icColor(light) } }, { appearance: 'dark', value: { solid: icColor(dark) } }, TINTED_WHITE],
  glass: true,
  'image-name': `${name}.svg`,
  name,
});
const iconJSON = {
  'fill-specializations': [{ value: { solid: icColor(PAPER) } }, { appearance: 'dark', value: { solid: icColor(DARK) } }],
  groups: [
    {
      name: 'Mark',
      layers: [layer('cursor', AMBER, AMBER_LIGHT), layer('slash', INK, PAPER)],
      lighting: 'individual',
      shadow: { kind: 'neutral', opacity: 0.5 },
      specular: true,
      translucency: { enabled: true, value: 0.4 },
    },
    {
      name: 'Shelf',
      layers: [{ ...layer('shelf', SHELF, SHELF_DARK), 'opacity-specializations': [{ appearance: 'tinted', value: 0.55 }] }],
      shadow: { kind: 'none', opacity: 0.5 },
      specular: false,
      translucency: { enabled: true, value: 0.4 },
    },
  ],
  'supported-platforms': { squares: 'shared' },
};

// ---- Write ----
const write = (dir, name, body) => { mkdirSync(dir, { recursive: true }); writeFileSync(join(dir, name), body); };
const run = (cmd, args) => {
  const r = spawnSync(cmd, args, { encoding: 'utf8' });
  if (r.error || r.status !== 0) throw new Error(`${cmd} failed: ${r.error?.message ?? r.stderr ?? r.stdout}`);
  return r.stdout;
};

write(OUT_LOGO, 'promptshelf-icon-light.svg', svgDoc(1024, flatLight, 'icon, light'));
write(OUT_LOGO, 'promptshelf-icon-dark.svg', svgDoc(1024, flatDark, 'icon, dark'));
write(OUT_LOGO, 'promptshelf-mark.svg', svgDoc(1024, mono, 'one-colour mark'));
write(OUT_LOGO, 'promptshelf-tray.svg', svgDoc(64, tray, 'menu bar template image'));
write(OUT_UI, 'promptshelf-icon-light.svg', svgDoc(1024, flatLight, 'icon, light'));
write(OUT_UI, 'promptshelf-icon-dark.svg', svgDoc(1024, flatDark, 'icon, dark'));
write(OUT_APP, 'tray.svg', svgDoc(64, tray, 'menu bar template image (black; macOS tints it)'));

rmSync(OUT_PACKAGE, { recursive: true, force: true });
write(OUT_PACKAGE, 'icon.json', JSON.stringify(iconJSON, null, 2) + '\n');
write(join(OUT_PACKAGE, 'Assets'), 'slash.svg', svgDoc(1024, path(FULL.slash, INK), `slash layer, ${okl(INK)}`));
write(join(OUT_PACKAGE, 'Assets'), 'cursor.svg', svgDoc(1024, path(FULL.cursor, AMBER), `cursor layer, ${okl(AMBER)}`));
write(join(OUT_PACKAGE, 'Assets'), 'shelf.svg', svgDoc(1024, path(FULL.shelf, SHELF), `shelf layer, ${okl(SHELF)}`));

// PNGs by rsvg-convert, from SVGs in a temporary folder.
const TMP = join(tmpdir(), `promptshelf-icon-${process.pid}`);
mkdirSync(TMP, { recursive: true });
const png = (svg, size, out) => {
  const src = join(TMP, `${size}-${Math.random().toString(36).slice(2)}.svg`);
  writeFileSync(src, svg);
  run('rsvg-convert', ['-w', `${size}`, '-h', `${size}`, '-o', out, src]);
  return readFileSync(out);
};
const LIGHT_SVG = svgDoc(1024, flatLight, 'icon, light');
for (const [name, size] of [['32x32.png', 32], ['64x64.png', 64], ['128x128.png', 128], ['128x128@2x.png', 256], ['icon.png', 512]]) {
  png(LIGHT_SVG, size, join(OUT_APP, name));
}
png(svgDoc(64, tray, 'tray'), 64, join(OUT_APP, 'tray.png'));
png(LIGHT_SVG, 512, join(OUT_RAYCAST, 'icon.png'));
png(svgDoc(1024, flatDark, 'icon, dark'), 512, join(OUT_RAYCAST, 'icon@dark.png'));

// Windows ICO: PNG entries (Vista and later read them).
const icoSizes = [16, 24, 32, 48, 64, 256];
const images = icoSizes.map((n) => png(LIGHT_SVG, n, join(TMP, `ico-${n}.png`)));
const head = Buffer.alloc(6 + 16 * images.length);
head.writeUInt16LE(1, 2);
head.writeUInt16LE(images.length, 4);
let offset = head.length;
images.forEach((img, i) => {
  const e = 6 + 16 * i, n = icoSizes[i] % 256;
  head[e] = n; head[e + 1] = n;
  head.writeUInt16LE(1, e + 4);
  head.writeUInt16LE(32, e + 6);
  head.writeUInt32LE(img.length, e + 8);
  head.writeUInt32LE(offset, e + 12);
  offset += img.length;
});
writeFileSync(join(OUT_APP, 'icon.ico'), Buffer.concat([head, ...images]));

// macOS: Tauri compiles AppIcon.icon into Assets.car when it bundles the app (it is listed in tauri.conf.json).
// Here actool compiles it once more only to get icon.icns, the fallback for macOS before 26.
const ICTOOL = '/Applications/Xcode.app/Contents/Applications/Icon Composer.app/Contents/Executables/ictool';
if (process.platform === 'darwin') {
  const out = join(TMP, 'actool');
  mkdirSync(out, { recursive: true });
  run('xcrun', ['actool', OUT_PACKAGE, '--compile', out, '--app-icon', 'AppIcon', '--platform', 'macosx',
    '--minimum-deployment-target', '11.0', '--output-partial-info-plist', join(out, 'partial.plist'),
    '--output-format', 'human-readable-text', '--errors', '--warnings']);
  renameSync(join(out, 'AppIcon.icns'), join(OUT_APP, 'icon.icns'));

  const at = process.argv.indexOf('--preview');
  if (at !== -1) {
    const dir = process.argv[at + 1];
    if (!dir) throw new Error('usage: --preview <dir>');
    mkdirSync(dir, { recursive: true });
    for (const r of ['Default', 'Dark', 'ClearLight', 'ClearDark', 'TintedLight', 'TintedDark']) {
      run(ICTOOL, [OUT_PACKAGE, '--export-image', '--output-file', join(dir, `macos-${r}.png`), '--platform', 'macOS',
        '--rendition', r, '--width', '512', '--height', '512', '--scale', '1']);
    }
    console.log(`preview: ${dir}`);
  }
} else {
  console.log('Not macOS: skipped icon.icns. Run the script on a Mac with Xcode to update it.');
}
rmSync(TMP, { recursive: true, force: true });
console.log(`icons: ${OUT_LOGO}\n       ${OUT_APP}\n       ${OUT_RAYCAST}`);
