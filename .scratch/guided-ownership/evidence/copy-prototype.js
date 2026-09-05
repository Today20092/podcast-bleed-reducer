// Run with node. Bounded pre-implementation experiment, no source audio access.
const n = 4096;
const x = Array.from({ length: n }, (_, i) =>
  0.2 * (Math.sin(i * 0.31) + 0.45 * Math.sin(i * 0.077 + Math.sin(i * 0.002))));
const z = Array.from({ length: n }, (_, i) =>
  0.12 * (Math.sin(i * 0.193) + 0.3 * Math.sin(i * 0.047)));
for (const [label, gain, overlap] of [
  ['delayed inverted bleed', -0.25, 0],
  ['quiet independent overlap', -0.25, 0.5],
  ['gain mismatch copy', 1.6, 0],
]) {
  const y = x.map((_, i) => gain * (x[i - 13] || 0) + overlap * z[i]);
  let best = 0, lag = 0;
  for (let l = -48; l <= 48; l++) {
    let xy = 0, xx = 0, yy = 0;
    for (let i = 64; i < n - 64; i++) {
      xy += x[i] * y[i + l];
      xx += x[i] * x[i];
      yy += y[i + l] * y[i + l];
    }
    const r = xy * xy / (xx * yy);
    if (r > best) { best = r; lag = l; }
  }
  if (lag !== 13 || (overlap ? best >= 0.5 : best < 0.999)) {
    throw new Error(`Unexpected result for ${label}`);
  }
  console.log({ label, lag, explainedPower: best, residualFraction: 1 - best });
}
