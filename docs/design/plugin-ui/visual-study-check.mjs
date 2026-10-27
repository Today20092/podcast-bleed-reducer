import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import vm from 'node:vm';

const source = readFileSync(new URL('./visual-study.html', import.meta.url), 'utf8');
const script = source.match(/<script>([\s\S]*?)<\/script>/)?.[1];
assert.ok(script, 'one self-contained script');
assert.equal((source.match(/<script/g) || []).length, 1);
assert.ok(!/(?:src|href)\s*=\s*["'](?:https?:|\/\/)/i.test(source), 'no remote resources');
assert.ok(!/\b(?:fetch|XMLHttpRequest|WebSocket|AudioContext|setInterval|setTimeout)\s*\(/.test(script), 'no network/audio/background timer');
for(const id of ['gate','podcast','play','rate','reduce','amount','timing','position'])assert.ok(source.includes(`id="${id}"`),'accessible external preview controls exist');
const context = vm.createContext({});
vm.runInContext(script, context, { timeout: 1000 });
const { renderPanel, pathFor, shouldAnimate, previewAttenuation } = context.cleanSlateStudy;
for(const p of [.25,.3,.4,.43]){assert.equal(previewAttenuation(0,p,80),0,'human-present span stays at unity');for(const i of [1,2])assert.equal(previewAttenuation(i,p,80),.8,'other labeled tracks retain explicit absence');}
for (const playing of [false,true]) for (const reduced of [false,true]) for (const hidden of [false,true]) {
  assert.equal(shouldAnimate(playing,reduced,hidden), playing && !reduced && !hidden);
}
assert.equal((pathFor(256,p=>[p,p]).match(/[ML]/g)||[]).length,256);
for (const mode of ['gate','podcast']) for (const width of [mode==='gate'?640:720,1000]) {
  const a=renderPanel(mode,5,{width,amount:50,timing:180}),b=renderPanel(mode,8,{width,amount:80,timing:350});
  const height=mode==='gate'?(width<800?440:620):(width<800?560:680);
  assert.ok(a.includes(`viewBox="0 0 ${width} ${height}"`));
  assert.ok(a.includes('generated signal') && a.includes('not a running audio plugin'));
  assert.ok(!/NaN|Infinity|undefined/.test(a+b));assert.notEqual(a,b,'time/controls change the visible study');
  assert.equal(a,renderPanel(mode,5,{width,amount:50,timing:180}),'deterministic frozen frame');
  assert.equal(a,readFileSync(new URL(`./study-renders/${mode}-${width<800?'compact':'full'}.svg`,import.meta.url),'utf8').trimEnd(),'frozen SVG matches current renderer');
  assert.ok((a.match(/[ML][\d.-]+,[\d.-]+/g)||[]).length<1600,'bounded geometry');
  for(const match of a.matchAll(/<rect x="([\d.-]+)" y="([\d.-]+)" width="([\d.-]+)" height="([\d.-]+)"/g)){
    const [,x,y,w,h]=match.map(Number);assert.ok(x>=0&&y>=0&&w>=0&&h>=0&&x+w<=width+.02&&y+h<=height+.02,'rectangle inside canvas');
  }
}
// Small DOM/scheduler double: test lifecycle/control wiring without launching a browser.
const elements=Object.fromEntries([...source.matchAll(/id="([\w-]+)"/g)].map(([,id])=>[id,{value:'',checked:false,clientWidth:1000,dataset:{},attrs:{},events:{},setAttribute(k,v){this.attrs[k]=v;},addEventListener(k,v){this.events[k]=v;}}]));
elements.amount.value='50';elements.timing.value='180';elements.position.value='5';elements.rate.value='30';
const pending=new Map(),docEvents={},pref={matches:false,addEventListener(k,v){this[k]=v;}};let next=0;
const document={hidden:false,getElementById:id=>elements[id],addEventListener:(k,v)=>{docEvents[k]=v;}};
const dom=vm.createContext({document,matchMedia:()=>pref,requestAnimationFrame:cb=>{pending.set(++next,cb);return next;},cancelAnimationFrame:id=>pending.delete(id),ResizeObserver:class{observe(){}}});
vm.runInContext(script,dom,{timeout:1000});assert.equal(pending.size,0,'paused initially');
elements.play.events.click();assert.equal(pending.size,1,'only one scheduled frame');
for(const now of [100,200]){const [id,callback]=pending.entries().next().value;pending.delete(id);callback(now);assert.equal(pending.size,1);}
document.hidden=true;docEvents.visibilitychange();assert.equal(pending.size,0,'hidden cancels all preview work');
document.hidden=false;docEvents.visibilitychange();assert.equal(pending.size,1,'visible resumes existing intent');
elements.reduce.checked=true;elements.reduce.events.change();assert.equal(pending.size,0,'reduced motion cancels work');assert.equal(elements.play.disabled,true);
elements.amount.value='80';elements.amount.events.input();assert.equal(elements['amount-value'].textContent,'80%');assert.ok(elements.viewer.innerHTML.includes('>80</text>'),'immediate control feedback even with reduced motion');
elements.podcast.events.click();assert.equal(elements.viewer.dataset.mode,'podcast');assert.ok(elements.viewer.innerHTML.includes('>Removed</text>'));assert.equal(elements.podcast.attrs['aria-pressed'],'true');
pref.change({matches:false});assert.equal(pending.size,1);elements.play.events.click();assert.equal(pending.size,0,'pause cancels work');
console.log('Visual study: deterministic compact/full states, bounded geometry, human-label gain, control feedback and animation lifecycle pass. Browser/native rendering and presentation performance are not covered.');
