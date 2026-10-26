// Original static design fixtures, never product audio or a functional plugin editor.
import { writeFileSync, mkdirSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import assert from 'node:assert/strict';

const out = fileURLToPath(new URL('.', import.meta.url));
const c = { surface:'#202226', panel:'#292D33', text:'#E4E7EB', muted:'#ADB5C0',
  control:'#343940', line:'#3D434C', action:'#779BC7', actionText:'#152233',
  wave:'#6FBEB7', gain:'#D282F5', focus:'#9AC5FF', error:'#DB8B8B', label:'#A0B9FF' };
const esc = s => String(s).replaceAll('&','&amp;').replaceAll('<','&lt;').replaceAll('"','&quot;');
const dimensions = {};
function document(name,w,h,paint) {
  let parts = [], boxes = [];
  const rect = (x,y,width,height,fill=c.panel,stroke='none',radius=4,opacity=1) => {
    assert(x>=0 && y>=0 && x+width<=w && y+height<=h, `${name}: out of bounds`);
    parts.push(`<rect x="${x}" y="${y}" width="${width}" height="${height}" rx="${radius}" fill="${fill}" fill-opacity="${opacity}" stroke="${stroke}"/>`);
  };
  const text = (x,y,value,size=14,color=c.text,weight=400) => {
    assert(x>=0 && y>=size && x<w && y<=h, `${name}: text anchor out of bounds`);
    parts.push(`<text x="${x}" y="${y}" font-size="${size}" fill="${color}" font-weight="${weight}">${esc(value)}</text>`);
  };
  const line = (x1,y1,x2,y2,color=c.line,width=1) => parts.push(`<path d="M${x1},${y1} L${x2},${y2}" stroke="${color}" stroke-width="${width}" fill="none"/>`);
  const button = (x,y,width,label,selected=false,focus=false) => {
    rect(x,y,width,30,selected?c.action:c.control,focus?c.focus:'none',4);
    text(x+10,y+20,label,14,selected?c.actionText:c.text,selected?600:400);
    boxes.push({label,x,y,width,height:30});
    if(focus) rect(x-2,y-2,width+4,34,'none',c.focus,5);
  };
  const wave = (x,y,width,height,phase=0) => {
    let d = '';
    for(let i=0;i<Math.floor(width/3);i++) {
      const a=(.11+.72*Math.max(0,Math.sin(i*.061+phase))**2)*(.42+.58*Math.abs(Math.sin(i*.67)));
      const p=x+i*3, v=height*a/2;
      d+=`M${p.toFixed(1)},${(y+height/2-v).toFixed(1)}v${(v*2).toFixed(1)}`;
    }
    parts.push(`<path d="${d}" stroke="${c.wave}" stroke-width="1.5" fill="none"/>`);
  };
  rect(0,0,w,h,c.surface,'none',0);
  paint({rect,text,line,button,wave,w,h});
  for(let i=0;i<boxes.length;i++) for(let j=i+1;j<boxes.length;j++) {
    const a=boxes[i], b=boxes[j];
    assert(a.x+a.width<=b.x || b.x+b.width<=a.x || a.y+a.height<=b.y || b.y+b.height<=a.y,
      `${name}: overlapping controls ${a.label}/${b.label}`);
  }
  const svg=`<svg xmlns="http://www.w3.org/2000/svg" width="${w}" height="${h}" viewBox="0 0 ${w} ${h}" role="img" aria-labelledby="title desc"><title id="title">${name.replaceAll('-',' ')}</title><desc id="desc">Original CleanSlate proposed layout. Synthetic signal and illustrative values; no runtime qualification.</desc><g font-family="Segoe UI">${parts.join('')}</g></svg>\n`;
  assert(svg.length<100000);
  writeFileSync(`${out}${name}.svg`,svg);
  dimensions[name]={width:w,height:h,controls:boxes.length};
}
function podcast(compact) {
  document(`podcast-${compact?'compact':'full'}`,compact?720:1000,compact?560:680,({rect,text,line,button,wave,w,h})=>{
    text(16,29,'CleanSlate Podcast',20,c.text,600);
    let x=w-292;
    for(const label of ['Original','Processed','Removed']) {button(x,12,label==='Processed'?100:88,label,label==='Processed');x+=label==='Processed'?108:96;}
    text(16,70,'Group A · 4 microphones',14);
    text(compact?244:360,70,'Verified capture 00:00–02:00.43',12,c.muted);
    button(w-86,49,70,'Setup');
    button(16,90,114,'Fit captured'); button(138,90,116,'Fit selection');
    button(262,90,76,'Center'); button(346,90,42,'−'); button(396,90,42,'+');
    button(w-256,90,112,'Catch up',true);button(w-136,90,120,'Lock playhead');
    const chartTop=148, chartBottom=compact?354:488, left=112, right=w-22;
    rect(16,132,w-32,chartBottom-132+28,'#181B1F',c.line);
    for(let i=0;i<=4;i++){const xx=left+(right-left)*i/4;line(xx,154,xx,chartBottom);text(Math.min(xx,w-65),148,`00:${24+i*4}`,12,c.muted);}
    const n=compact?2:4,lane=(chartBottom-chartTop-18)/n;
    for(let i=0;i<n;i++) {
      const y=chartTop+12+i*lane;
      text(28,y+27,['Nora','Sam','Mic 3','Mic 4'][i],14,c.text,600);
      text(28,y+47,i===0?'Selected':'Verified',12,c.muted);
      wave(left,y+10,right-left-12,lane-28,i*1.6);
      line(left,y+lane-2,right,y+lane-2);
      const gy=y+8;
      partsGain(line,left,right,gy,lane);
    }
    const selectionX=left+(right-left)*3.1/16,selectionWidth=(right-left)*3.3/16;
    rect(selectionX,chartTop+8,selectionWidth,chartBottom-chartTop-9,'#4678DC',c.label,0,.18);
    rect(selectionX,chartTop+8,selectionWidth,20,c.label);
    text(selectionX+7,chartTop+23,'Nora + Sam',12,'#101824',600);
    line(left+(right-left)*7.2/16,chartTop+28,left+(right-left)*7.2/16,chartBottom,'#FFFFFF',1.5);
    if(compact){rect(w-20,chartTop+10,4,chartBottom-chartTop-14,c.control,'none',2);
      rect(w-20,chartTop+10,4,(chartBottom-chartTop-14)/2,c.muted,'none',2);}
    text(28,chartBottom+19,compact?'2 of 4 tracks · scroll lanes':'4 verified tracks',12,c.muted);
    text(w-236,chartBottom+19,'View 00:24–00:40 · DAW 00:31.2',12,c.muted);
    const ny=chartBottom+36;
    rect(16,ny,w-32,20,c.control);rect(16+(w-32)*24/120.43,ny,(w-32)*16/120.43,20,'#3A557D',c.label);
    const sy=ny+45;
    text(16,sy,'Selection 00:27.1–00:30.4',14,c.text,600);
    text(16,sy+21,'Active speakers: Nora + Sam',12,c.muted);
    button(w-222,sy-15,102,'Edit label',false,true);button(w-112,sy-15,96,'Clear range');
    const bottom=h-80;
    line(16,bottom-9,w-16,bottom-9);
    text(16,bottom+12,'Nora · Attenuation −24 dB',14);
    button(compact?218:252,bottom-9,58,'Solo');button(compact?284:318,bottom-9,108,'Sound details');
    button(w-112,bottom-9,96,'Analyze',true);
    text(16,h-32,'Ready · examples complete · analysis current',12,c.wave);
    text(16,h-10,'Design fixture · synthetic signal · host owns transport',12,c.muted);
  });
}
function partsGain(line,left,right,y,lane) {
  // Human speech labels retain unity across the entire illustrative selected span.
  const a=left+(right-left)*3.1/16,b=left+(right-left)*6.4/16;
  line(left,y+lane*.45,a-14,y+lane*.45,c.gain,1.5);
  line(a-14,y+lane*.45,a,y+8,c.gain,1.5);line(a,y+8,b,y+8,c.gain,1.5);
  line(b,y+8,b+14,y+lane*.45,c.gain,1.5);line(b+14,y+lane*.45,right,y+lane*.45,c.gain,1.5);
}
function gate(compact) {
  document(`gate-${compact?'compact':'full'}`,compact?640:800,compact?440:560,({rect,text,line,button,wave,w,h})=>{
    text(20,31,'CleanSlate Smart Gate',20,c.text,600);
    button(w-104,13,84,'Bypass');
    text(20,65,'Detector ready · Speech',14,c.wave);
    text(20,86,'Speech activity estimate 0.86 · not calibrated certainty',12,c.muted);
    text(w-154,66,'Open · 0.0 dB',16,c.text,600);
    const gy=108,gh=compact?142:210;
    rect(20,gy,w-40,gh,'#181B1F',c.line);
    text(32,gy+24,'INPUT = OUTPUT · OPEN',12,c.muted);
    wave(32,gy+38,w-64,gh-76,.4);
    line(32,gy+gh-24,w-32,gy+gh-24,c.line);
    text(32,gy+gh-8,'Recent 8 s · display history only',12,c.muted);
    const by=gy+gh+20;
    text(20,by+16,'Applied reduction',14);
    rect(178,by+2,w-198,16,c.control);
    text(20,by+39,'0.0 dB · gate open',12,c.muted);
    const cy=by+65;
    text(20,cy,'Attenuation',14,c.text,600);text(20,cy+27,'−24 dB',20,c.text,600);line(20,cy+33,145,cy+33,c.muted);
    text(190,cy,'Activity threshold',14,c.text,600);text(190,cy+27,'0.55',20,c.text,600);line(190,cy+33,315,cy+33,c.muted);
    text(362,cy,'Timing',14,c.text,600);
    text(362,cy+25,compact?'Advanced…':'Attack 5 · Hold 150 · Release 180 ms',12,c.muted);
    if(!compact) text(20,cy+60,'Activity estimates drive a separate temporal gain controller.',12,c.muted);
    const foot=h-52;
    button(20,foot,86,'Original');button(114,foot,104,'Processed',true);button(226,foot,92,'Removed',false,true);
    button(w-162,foot,142,'Detector details');
    text(20,h-8,'Design fixture · synthetic values · no auto leveling or speaker identity',12,c.muted);
  });
}
function luminance(hex) {
  const a=hex.slice(1).match(/../g).map(x=>parseInt(x,16)/255).map(v=>v<=.04045?v/12.92:((v+.055)/1.055)**2.4);
  return a[0]*.2126+a[1]*.7152+a[2]*.0722;
}
for(const [fg,bg] of [[c.text,c.surface],[c.muted,c.surface],[c.text,c.control],[c.actionText,c.action],[c.wave,c.surface]]) {
  const a=luminance(fg),b=luminance(bg);assert((Math.max(a,b)+.05)/(Math.min(a,b)+.05)>=4.5);
}
mkdirSync(out,{recursive:true});
podcast(true);podcast(false);gate(true);gate(false);
writeFileSync(`${out}dimensions.json`,JSON.stringify(dimensions,null,2)+'\n');
console.log('PASS: four original layouts, control non-overlap/bounds, artifact-size and text-contrast assertions');
