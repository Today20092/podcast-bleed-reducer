import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { readFileSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

// ImageMagick is an asset-authoring tool; application builds use the checked-in files.
const asset = (name) => fileURLToPath(new URL(`../assets/${name}`, import.meta.url));
const source = asset('cleanslate.svg');
const render = (size, format = 'png:-') => execFileSync('magick', [
    '-background', 'none', source, '-resize', `${size}x${size}`, '-depth', '8', format,
], { maxBuffer: 8 * 1024 * 1024 });

const sizes = [256, 128, 64, 48, 40, 32, 24, 20, 16];
execFileSync('magick', ['-background', 'none', source, '-define',
    `icon:auto-resize=${sizes.join(',')}`, asset('cleanslate.ico')]);
const ico = readFileSync(asset('cleanslate.ico'));
assert.equal(ico.readUInt16LE(2), 1);
assert.equal(ico.readUInt16LE(4), sizes.length);
sizes.forEach((size, index) => assert.equal(ico[6 + index * 16] || 256, size));

const rgba = render(64, 'RGBA:-');
assert.equal(rgba.length, 64 * 64 * 4);
assert(rgba.some((channel, index) => index % 4 === 3 && channel > 0));
writeFileSync(asset('cleanslate.rgba'), rgba);
writeFileSync(asset('cleanslate.png'), render(1024));
writeFileSync(asset('github-avatar.png'), execFileSync('magick', [
    '-background', '#181b24', source, '-resize', '512x512', '-alpha', 'remove', 'png:-',
]));

const records = [['icp4', 16], ['icp5', 32], ['icp6', 64], ['ic07', 128],
    ['ic08', 256], ['ic09', 512], ['ic10', 1024]].map(([type, size]) => {
    const png = render(size);
    assert.equal(png.subarray(0, 8).toString('hex'), '89504e470d0a1a0a');
    assert.equal(png.readUInt32BE(16), size);
    assert.equal(png.readUInt32BE(20), size);
    const header = Buffer.alloc(8);
    header.write(type, 0, 'ascii');
    header.writeUInt32BE(8 + png.length, 4);
    return Buffer.concat([header, png]);
});
const header = Buffer.alloc(8);
header.write('icns');
header.writeUInt32BE(8 + records.reduce((length, record) => length + record.length, 0), 4);
const icns = Buffer.concat([header, ...records]);
assert.equal(icns.readUInt32BE(4), icns.length);
writeFileSync(asset('cleanslate.icns'), icns);
console.log('Checked and generated Windows ICO, 64px RGBA, macOS ICNS, master PNG and GitHub avatar.');
