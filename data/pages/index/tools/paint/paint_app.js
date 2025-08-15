// paint_app.js - Extended: 一貫した領域選択(矩形/ラッソ) + ブラシ形状 + コピー系 + 履歴統合
(function(){
  'use strict';
  const VP={}; window.VP=VP;
  const $=id=>document.getElementById(id);
  // DOM refs
  const canvas=$('main'); const ctx=canvas.getContext('2d');
  const toolSel=$('tool'), colorInput=$('color')||{value:'#c33'}, widthInput=$('width')||{value:3};
  // Legacy toolbar elements removed; keep null-safe placeholders
  const infoEl=null, cursorEl=null, zoomEl=null;
  const btnUndo=$('undo'), btnRedo=$('redo'), btnDelete=$('deleteEl');
  // Legacy controls removed in new UI layout
  const btnCopy=null, btnPaste=null, btnCut=null, btnSelectAll=$('btnSelectAll'), btnDeselect=$('btnDeselect');
  const btnApplyStroke=null, btnApplyView=null, btnResetView=null;
  const layerSelect={innerHTML:'',appendChild:()=>{},value:null};
  const layerOpacity={value:1};
  const layerBlend={value:'source-over'};
  // Export panel (new layout always visible panelExport)
  const doExportBtn=$('doExport'), exportFormatSel=$('exportFormat'), exportScaleInput=$('exportScale');
  const exportBgInput=$('exportBg'), exportTransparentChk=$('exportTransparent'), exportUseCropChk=$('exportUseCrop'), exportIncludeViewChk=$('exportIncludeView');
  // Popup (mini layer panel & selection etc.) elements if present
  const selCountSpan=document.getElementById('selCount');
  const layerListMini=document.getElementById('layerListMini');
  const btnLayerUp=document.getElementById('btnLayerUp');
  const btnLayerDown=document.getElementById('btnLayerDown');
  const btnDupLayer=document.getElementById('btnLayerDup');
  const btnLayerNew=document.getElementById('btnLayerNew');
  const btnLayerDel=document.getElementById('btnLayerDel');
  const layerListEl=document.getElementById('layerList');
  const statusBar=document.getElementById('statusBar');
  const statusSpans=statusBar?Array.from(statusBar.querySelectorAll('span[data-k]')).reduce((m,el)=>{m[el.dataset.k]=el;return m;},{}):{};
  const historyListEl=document.getElementById('historyList');
  const btnLayerCopy=document.getElementById('btnLayerCopy');
  const btnLayerPaste=document.getElementById('btnLayerPaste');

  // Status bar updater (restored after refactor)
  function updateStatusBar(){
    if(!statusSpans) return;
    if(statusSpans.sel) statusSpans.sel.textContent='sel:'+selection.size;
    if(statusSpans.objs) statusSpans.objs.textContent='obj:'+allElements().length;
    if(statusSpans.layers) statusSpans.layers.textContent='layers:'+layers.length;
    if(statusSpans.zoom){ const z=view[0]; statusSpans.zoom.textContent=Math.round(z*100)+'%'; }
    if(selCountSpan) selCountSpan.textContent=selection.size;
  }

  // State
  let layers=[]; let activeLayerId=null; let selection=new Set();
  let view=[1,0,0,1,0,0];
  const input={isDown:false,lastX:0,lastY:0,startX:0,startY:0,currentPath:null,currentRect:null,spacePan:false,hoverWorld:null, regionDraft:null, regionShift:false};
  let selectionRegions=[]; // [{mode:'rect',rect:{x,y,w,h}}, {mode:'lasso',points:[]}] for export reuse
  let clipboard=[]; let currentBrushShape='round';
  let brushOpacity=1; // 0-1 stroke opacity
  let layerMoveActive=false; let layerMoveStart={x:0,y:0};

  const CONFIG={selectionDash:[4,4], previewDash:[6,4], hitTestTolerance:6}; VP.config=CONFIG;

  // History (include selection + regions)
  const history=[]; let historyIndex=-1;
  const cloneElement=el=> el.type==='path'? {...el, points:el.points.map(p=>({x:p.x,y:p.y}))}:{...el};
  const cloneLayer=l=> ({...l, elements:l.elements.map(cloneElement)});
  const snapshot=()=>({layers:layers.map(cloneLayer), activeLayerId, view:[...view], selection:[...selection], selectionRegions:JSON.parse(JSON.stringify(selectionRegions))});
  function pushHistory(label){ history.splice(historyIndex+1); const snap=snapshot(); if(label) snap._label=label; history.push(snap); historyIndex=history.length-1; updateHistoryButtons(); rebuildHistoryPanel(); }
  function restore(s){ layers=s.layers.map(cloneLayer); activeLayerId=s.activeLayerId; view=[...s.view]; selection=new Set(s.selection||[]); selectionRegions=s.selectionRegions||[]; rebuildLayerSelect(); render(); syncInfo(); syncZoom(); rebuildHistoryPanel(); }
  function undo(){ if(historyIndex<=0) return; historyIndex--; restore(history[historyIndex]); }
  function redo(){ if(historyIndex>=history.length-1) return; historyIndex++; restore(history[historyIndex]); }
  function updateHistoryButtons(){ btnUndo.disabled=historyIndex<=0; btnRedo.disabled=historyIndex>=history.length-1; }

  // Layers
  function newLayerName(){ let i=1; while(layers.some(l=>l.name==='Layer'+i)) i++; return 'Layer'+i; }
  function createLayer(opts={}){ const layer={id:'L'+Math.random().toString(36).slice(2,8), name:opts.name||newLayerName(), visible:true, opacity:opts.opacity??1, blend:opts.blend||'source-over', elements:[]}; layers.push(layer); activeLayerId=layer.id; rebuildLayerSelect(); pushHistory('layer-new'); return layer; }
  function getActiveLayer(){ return layers.find(l=>l.id===activeLayerId); }
  function deleteActiveLayer(){ if(layers.length<=1) return; const idx=layers.findIndex(l=>l.id===activeLayerId); if(idx<0)return; layers.splice(idx,1); activeLayerId=layers[Math.max(0,idx-1)].id; rebuildLayerSelect(); pushHistory('layer-del'); render(); }
  function rebuildLayerSelect(){ syncLayerUI(); rebuildLayerMini(); rebuildLayerPanel(); updateStatusBar(); }
  function syncLayerUI(){ const l=getActiveLayer(); if(!l) return; layerOpacity.value=l.opacity; layerBlend.value=l.blend; }
  function allElements(){ return layers.flatMap(l=>l.elements.map(e=>({...e,_layer:l.id}))); }
  function elementsInActiveLayer(){ const l=getActiveLayer(); if(!l) return []; return l.elements.map(e=>({...e,_layer:l.id})); }

  // Math
  const mul=(A,B)=>{const [a,b,c,d,e,f]=A,[A2,B2,C2,D2,E2,F2]=B;return [a*A2+c*B2,b*A2+d*B2,a*C2+c*D2,b*C2+d*D2,a*E2+c*F2+e,b*E2+d*F2+f];};
  const applyM=(M,x,y)=>{const [a,b,c,d,e,f]=M;return {x:a*x+c*y+e,y:b*x+d*y+f};};
  const invert=M=>{const [a,b,c,d,e,f]=M; const det=a*d-b*c; if(!det) return null; return [d/det,-b/det,-c/det,a/det,(c*f-d*e)/det,(b*e-a*f)/det]; };
  let baseScale=1; function combinedView(){ return [view[0]*baseScale,view[1]*baseScale,view[2]*baseScale,view[3]*baseScale,view[4]*baseScale,view[5]*baseScale]; }
  function screenToCanvas(sx,sy){ const inv=invert(combinedView()); if(!inv) return {x:sx,y:sy}; return applyM(inv,sx*baseScale,sy*baseScale); }
  VP.math={mul,applyM,invert,screenToCanvas};

  // Sizing
  function resize(){ baseScale=window.devicePixelRatio||1; const w=window.innerWidth, h=window.innerHeight-46; canvas.style.width=w+'px'; canvas.style.height=h+'px'; canvas.width=Math.round(w*baseScale); canvas.height=Math.round(h*baseScale); ctx.setTransform(1,0,0,1,0,0); render(); }
  window.addEventListener('resize', resize); resize();

  // Render
  function drawElement(el){ if(el.type==='path'){ if(!el.points.length)return; if(el.points.length===1){ // dot
      ctx.save(); ctx.fillStyle=el.stroke; if(el.strokeOpacity!=null) ctx.globalAlpha*=el.strokeOpacity; const p=el.points[0]; const r=Math.max(1,el.strokeWidth/2); ctx.beginPath(); ctx.arc(p.x,p.y,r,0,Math.PI*2); ctx.fill(); ctx.restore(); return; }
      if(el.brushShape==='dotted'){ drawDotted(el); return;} ctx.save(); ctx.beginPath(); ctx.lineCap=(el.brushShape==='square')?'butt':'round'; ctx.lineJoin=(el.brushShape==='square')?'miter':'round'; ctx.lineWidth=el.strokeWidth; ctx.strokeStyle=el.stroke; if(el.strokeOpacity!=null) ctx.globalAlpha*=el.strokeOpacity; ctx.moveTo(el.points[0].x,el.points[0].y); for(let i=1;i<el.points.length;i++) ctx.lineTo(el.points[i].x,el.points[i].y); ctx.stroke(); ctx.restore(); } else if(el.type==='rect'){ ctx.save(); ctx.lineWidth=el.strokeWidth||1; ctx.strokeStyle=el.stroke||'#000'; if(el.strokeOpacity!=null) ctx.globalAlpha*=el.strokeOpacity; ctx.strokeRect(el.x,el.y,el.w,el.h); ctx.restore(); } }
  function drawSelectionOutline(el){ ctx.save(); ctx.lineWidth=1; ctx.setLineDash(CONFIG.selectionDash); ctx.strokeStyle='#1d6ae5'; if(el.type==='path'){ ctx.beginPath(); ctx.moveTo(el.points[0].x,el.points[0].y); for(let i=1;i<el.points.length;i++) ctx.lineTo(el.points[i].x,el.points[i].y); ctx.stroke(); } else ctx.strokeRect(el.x,el.y,el.w,el.h); ctx.restore(); }
  function drawSelectionRegions(){ if(!selectionRegions.length && !input.regionDraft) return; ctx.save(); ctx.lineWidth=1; ctx.setLineDash([6,4]); ctx.strokeStyle='rgba(0,150,255,.9)'; const regs=[...selectionRegions]; if(input.regionDraft) regs.push(input.regionDraft); for(const r of regs){ if(r.mode==='rect'){ const rr=r.rect; ctx.strokeRect(rr.x,rr.y,rr.w,rr.h); } else if(r.mode==='lasso'&&r.points.length>1){ ctx.beginPath(); ctx.moveTo(r.points[0].x,r.points[0].y); for(let i=1;i<r.points.length;i++) ctx.lineTo(r.points[i].x,r.points[i].y); if(!r.draft) ctx.closePath(); ctx.stroke(); }} ctx.restore(); }
  function render(){ ctx.setTransform(1,0,0,1,0,0); ctx.clearRect(0,0,canvas.width,canvas.height); ctx.setTransform(view[0]*baseScale,view[1]*baseScale,view[2]*baseScale,view[3]*baseScale,view[4]*baseScale,view[5]*baseScale); for(const l of layers){ if(!l.visible||!l.elements.length) continue; ctx.save(); ctx.globalAlpha=l.opacity; ctx.globalCompositeOperation=l.blend; for(const el of l.elements) drawElement(el); ctx.restore(); } if(input.currentPath) drawElement(input.currentPath); if(input.currentRect){ const r=input.currentRect; ctx.save(); ctx.setLineDash(CONFIG.previewDash); ctx.lineWidth=r.strokeWidth; ctx.strokeStyle=r.stroke; ctx.strokeRect(r.x,r.y,r.w,r.h); ctx.restore(); } for(const id of selection){ const el=allElements().find(e=>e.id===id); if(el) drawSelectionOutline(el); } drawSelectionRegions(); }

  // Utils
  const newId=()=> 'e'+Math.random().toString(36).slice(2,9);
  function addElement(el){ const layer=getActiveLayer(); if(!layer) return; layer.elements.push(el); pushHistory('add-el'); rebuildLayerPanel(); syncInfo(); }
  function syncInfo(){ updateStatusBar(); }
  function syncZoom(){ updateStatusBar(); }
  function currentTool(){ return input.spacePan? 'pan' : toolSel.value; }
  function cloneSelection(){ return allElements().filter(e=>selection.has(e.id)).map(el=> el.type==='path'? {id:newId(),type:'path',points:el.points.map(p=>({x:p.x,y:p.y})),stroke:el.stroke,strokeWidth:el.strokeWidth,brushShape:el.brushShape}:{...el,id:newId()}); }
  function pasteAt(dx=20,dy=20){ if(!clipboard.length) return; const layer=getActiveLayer(); if(!layer) return; const clones=clipboard.map(el=> el.type==='path'? {...el,id:newId(),points:el.points.map(p=>({x:p.x+dx,y:p.y+dy}))}:{...el,id:newId(),x:el.x+dx,y:el.y+dy}); layer.elements.push(...clones); selection.clear(); clones.forEach(c=>selection.add(c.id)); pushHistory(); render(); syncInfo(); }

  // Hit test
  function hitTest(x,y){ // 制限: アクティブレイヤー内のみ
    const flat=elementsInActiveLayer(); const tol=CONFIG.hitTestTolerance/Math.hypot(view[0],view[1]);
    for(let i=flat.length-1;i>=0;i--){ const el=flat[i]; if(el.type==='rect'){ if(x>=el.x&&x<=el.x+el.w && y>=el.y&&y<=el.y+el.h) return el; } else if(el.type==='path'){ if(pointNearPath(el,x,y,tol)) return el; } }
    return null; }
  function pointNearPath(path,x,y,th){ for(let i=1;i<path.points.length;i++){ if(distSeg(x,y,path.points[i-1],path.points[i])<=th) return true; } return false; }
  function distSeg(px,py,a,b){ const dx=b.x-a.x, dy=b.y-a.y, l2=dx*dx+dy*dy; if(!l2) return Math.hypot(px-a.x,py-a.y); let t=((px-a.x)*dx+(py-a.y)*dy)/l2; t=Math.max(0,Math.min(1,t)); const cx=a.x+dx*t, cy=a.y+dy*t; return Math.hypot(px-cx,py-cy); }

  // Region hit
  function regionHitsRect(region,el){ // 完全内包のみ
    if(region.mode==='rect'){ const r=region.rect; return (el.x>=r.x && el.y>=r.y && el.x+el.w<=r.x+r.w && el.y+el.h<=r.y+r.h); }
    if(region.mode==='lasso'){ const pts=[[el.x,el.y],[el.x+el.w,el.y],[el.x,el.y+el.h],[el.x+el.w,el.y+el.h]]; return pts.every(p=>pointInPolygon(p[0],p[1],region.points)); }
    return false; }
  function regionHitsPath(region,el){ // 路線上全ポイント内包(簡易)
    if(region.mode==='rect'){ const r=region.rect; return el.points.every(p=> p.x>=r.x&&p.x<=r.x+r.w && p.y>=r.y&&p.y<=r.y+r.h); }
    if(region.mode==='lasso'){ return el.points.every(p=>pointInPolygon(p.x,p.y,region.points)); }
    return false; }
  function pointInPolygon(x,y,pts){ let inside=false; for(let i=0,j=pts.length-1;i<pts.length;j=i++){ const xi=pts[i].x, yi=pts[i].y, xj=pts[j].x, yj=pts[j].y; const intersect=((yi>y)!=(yj>y)) && (x < (xj-xi)*(y-yi)/(yj-yi)+xi); if(intersect) inside=!inside; } return inside; }

  // Brushes
  function drawDotted(el){ if(el.points.length<2) return; ctx.save(); ctx.fillStyle=el.stroke; const step=el.strokeWidth*1.5; for(let i=1;i<el.points.length;i++){ const a=el.points[i-1], b=el.points[i]; const dx=b.x-a.x, dy=b.y-a.y, dist=Math.hypot(dx,dy); const segs=Math.max(1,Math.floor(dist/step)); for(let s=0;s<=segs;s++){ const t=s/segs; const px=a.x+dx*t, py=a.y+dy*t; ctx.beginPath(); ctx.arc(px,py,el.strokeWidth/2,0,Math.PI*2); ctx.fill(); }} ctx.restore(); }

  // Cursor
  function updateCursor(){ let cls='select'; const t=currentTool(); if(t==='draw') cls='crosshair'; else if(t==='pan') cls=input.isDown?'grabbing':'grab'; else if(t==='rect') cls='crosshair'; else if(t.startsWith('select')) cls='select'; if(canvas._cursorCls!==cls){ canvas.classList.remove('crosshair','grab','grabbing','move','select'); canvas.classList.add(cls); canvas._cursorCls=cls; } }
  toolSel.addEventListener('change', updateCursor);
  // ensure initial tool value available
  if(!toolSel.value) toolSel.value='select';

  // Pointer events
  canvas.addEventListener('pointerdown', e=>{ const r=canvas.getBoundingClientRect(); const sx=e.clientX-r.left, sy=e.clientY-r.top; input.isDown=true; input.startX=sx; input.startY=sy; input.lastX=sx; input.lastY=sy; canvas.setPointerCapture(e.pointerId); const tool=currentTool(); const strokeW=Math.max(1,Number(widthInput.value)||3); if(tool==='draw'){ input.currentPath={id:newId(),type:'path',brushShape:currentBrushShape,points:[screenToCanvas(sx,sy)],stroke:colorInput.value||'#c33',strokeWidth:strokeW,strokeOpacity:brushOpacity}; } else if(tool==='rect'){ const p=screenToCanvas(sx,sy); input.currentRect={id:newId(),type:'rect',x:p.x,y:p.y,w:0,h:0,stroke:colorInput.value||'#c33',strokeWidth:strokeW,strokeOpacity:brushOpacity}; } else if(tool==='bucket'){ const wpt=screenToCanvas(sx,sy); const hit=hitTest(wpt.x,wpt.y); if(hit){ // recolor existing element
  hit.stroke=colorInput.value; hit.strokeOpacity=brushOpacity; pushHistory('bucket-recolor'); render(); }
        else { // add small fill marker (placeholder)
          const el={id:newId(),type:'rect',x:wpt.x-8,y:wpt.y-8,w:16,h:16,stroke:colorInput.value,strokeWidth:1,strokeOpacity:brushOpacity}; addElement(el); }
        input.isDown=false; canvas.releasePointerCapture(e.pointerId); return; }
  else if(tool==='select'){ const w=screenToCanvas(sx,sy); const hit=hitTest(w.x,w.y); if(hit){ if(!e.shiftKey&&!selection.has(hit.id)) selection.clear(); if(selection.has(hit.id)&&e.shiftKey) selection.delete(hit.id); else selection.add(hit.id); pushHistory('select'); render(); } else { // 背景: Alt+ドラッグでレイヤー移動
        if(!e.shiftKey && e.altKey){ layerMoveActive=true; layerMoveStart=screenToCanvas(sx,sy); }
        else if(!e.shiftKey){ if(selection.size){ selection.clear(); pushHistory('deselect'); render(); } }
      } } else if(tool==='select-rect'){ const p=screenToCanvas(sx,sy); input.regionDraft={mode:'rect',rect:{x:p.x,y:p.y,w:0,h:0},draft:true}; input.regionShift=e.shiftKey; render(); } else if(tool==='select-lasso'){ const p=screenToCanvas(sx,sy); input.regionDraft={mode:'lasso',points:[p],draft:true}; input.regionShift=e.shiftKey; render(); } });
  canvas.addEventListener('pointermove', e=>{ const r=canvas.getBoundingClientRect(); const sx=e.clientX-r.left, sy=e.clientY-r.top; const dx=sx-input.lastX, dy=sy-input.lastY; input.lastX=sx; input.lastY=sy; const world=screenToCanvas(sx,sy); input.hoverWorld=world; if(statusSpans.coords) statusSpans.coords.textContent='('+world.x.toFixed(1)+','+world.y.toFixed(1)+')'; updateCursor(); const tool=currentTool(); if(!input.isDown){ render(); return; } if(layerMoveActive){ const now=screenToCanvas(sx,sy); const l=getActiveLayer(); if(l){ const mx=now.x-layerMoveStart.x, my=now.y-layerMoveStart.y; for(const el of l.elements){ if(el.type==='path'){ for(const p of el.points){ p.x+=mx; p.y+=my; } } else { el.x+=mx; el.y+=my; } } layerMoveStart=now; render(); }
      return; }
    if(tool==='pan'){ view[4]+=dx; view[5]+=dy; syncZoom(); render(); return; } if(tool==='draw' && input.currentPath){ input.currentPath.points.push(screenToCanvas(sx,sy)); render(); return; } if(tool==='rect' && input.currentRect){ const p=screenToCanvas(sx,sy); input.currentRect.w=p.x-input.currentRect.x; input.currentRect.h=p.y-input.currentRect.y; render(); return; } if(tool==='select-rect' && input.regionDraft){ const p=screenToCanvas(sx,sy); input.regionDraft.rect.w=p.x-input.regionDraft.rect.x; input.regionDraft.rect.h=p.y-input.regionDraft.rect.y; render(); return; } if(tool==='select-lasso' && input.regionDraft){ const p=screenToCanvas(sx,sy); const pts=input.regionDraft.points; if(!pts.length || Math.hypot(pts[pts.length-1].x-p.x, pts[pts.length-1].y-p.y)>0.5) pts.push(p); render(); return; } });
  canvas.addEventListener('pointerup', e=>{ canvas.releasePointerCapture(e.pointerId); if(!input.isDown) return; input.isDown=false; if(layerMoveActive){ layerMoveActive=false; pushHistory('layer-move'); render(); }
    if(input.currentPath){ if(input.currentPath.points.length===1){ // ensure dot remains
        const p=input.currentPath.points[0]; input.currentPath.points.push({x:p.x+0.01,y:p.y}); }
      addElement(input.currentPath); input.currentPath=null; render(); }
    if(input.currentRect){ const r=input.currentRect; if(Math.abs(r.w)<1&&Math.abs(r.h)<1){ input.currentRect=null; render(); return;} if(r.w<0){ r.x+=r.w;r.w*=-1;} if(r.h<0){ r.y+=r.h;r.h*=-1;} addElement(r); input.currentRect=null; render(); }
  if(input.regionDraft){ const draft=input.regionDraft; draft.draft=false; if(draft.mode==='rect'){ const rr=draft.rect; if(rr.w<0){ rr.x+=rr.w; rr.w*=-1;} if(rr.h<0){ rr.y+=rr.h; rr.h*=-1;} } const added=new Set(); for(const el of allElements()){ if(el.type==='rect'){ if(regionHitsRect(draft,el)) added.add(el.id);} else if(el.type==='path'){ if(regionHitsPath(draft,el)) added.add(el.id);} } if(!input.regionShift){ selection.clear(); selectionRegions=[]; } if(added.size){ for(const id of added) selection.add(id); } selectionRegions.push(JSON.parse(JSON.stringify(draft))); input.regionDraft=null; input.regionShift=false; pushHistory('region-select'); render(); } });
  canvas.addEventListener('wheel', e=>{ if(e.ctrlKey) return; e.preventDefault(); const r=canvas.getBoundingClientRect(); const sx=e.clientX-r.left, sy=e.clientY-r.top; const factor=Math.exp((-e.deltaY)*(e.shiftKey?0.0005:0.0015)); const before=screenToCanvas(sx,sy); view=mul([factor,0,0,factor,0,0],view); const after=applyM(view,before.x,before.y); view[4]+=sx-after.x; view[5]+=sy-after.y; syncZoom(); render(); }, {passive:false});

  // Keyboard
  window.addEventListener('keydown', e=>{ const key=e.key.toLowerCase(); if((e.ctrlKey||e.metaKey)&&key==='z'){ e.preventDefault(); if(e.shiftKey) redo(); else undo(); return; } if((e.ctrlKey||e.metaKey)&&key==='y'){ e.preventDefault(); redo(); return; } if((e.ctrlKey||e.metaKey)&&key==='c'){ if(selection.size) clipboard=cloneSelection(); return; } if((e.ctrlKey||e.metaKey)&&key==='x'){ if(selection.size){ clipboard=cloneSelection(); for(const l of layers){ l.elements=l.elements.filter(el=>!selection.has(el.id)); } selection.clear(); pushHistory(); render(); syncInfo(); } return; } if((e.ctrlKey||e.metaKey)&&key==='v'){ e.preventDefault(); pasteAt(); return; } if((e.ctrlKey||e.metaKey)&&key==='a'){ e.preventDefault(); selection.clear(); for(const el of allElements()) selection.add(el.id); render(); return; } if(key==='delete'||key==='backspace'){ if(selection.size){ for(const l of layers){ l.elements=l.elements.filter(el=>!selection.has(el.id)); } selection.clear(); pushHistory(); render(); syncInfo(); } return; } if(key==='escape'){ if(selection.size){ selection.clear(); pushHistory(); render(); } return; } if(key==='v'){ toolSel.value='select'; toolSel.dispatchEvent(new Event('change')); return; } if(key==='b'||key==='p'){ toolSel.value='draw'; toolSel.dispatchEvent(new Event('change')); return; } if(key==='r'||key==='m'){ toolSel.value='rect'; toolSel.dispatchEvent(new Event('change')); return; } if(key==='h'){ toolSel.value='pan'; toolSel.dispatchEvent(new Event('change')); return; } if(key==='w'){ toolSel.value='select-rect'; toolSel.dispatchEvent(new Event('change')); return; } if(key==='l'){ toolSel.value='select-lasso'; toolSel.dispatchEvent(new Event('change')); return; } if(key==='1'){ currentBrushShape='round'; return; } if(key==='2'){ currentBrushShape='square'; return; } if(key==='3'){ currentBrushShape='dotted'; return; } if(e.code==='Space'){ input.spacePan=true; updateCursor(); return; } });
  window.addEventListener('keyup', e=>{ if(e.code==='Space'){ input.spacePan=false; updateCursor(); }});

  // Buttons
  // Selection buttons in new panel
  if(btnSelectAll) btnSelectAll.addEventListener('click', ()=>{ selection.clear(); for(const el of allElements()) selection.add(el.id); pushHistory('sel-all'); render(); updateStatusBar(); });
  if(btnDeselect) btnDeselect.addEventListener('click', ()=>{ if(selection.size){ selection.clear(); pushHistory('sel-none'); render(); updateStatusBar(); }});
  if(btnDelete) btnDelete.addEventListener('click', ()=>{ if(!selection.size) return; for(const l of layers){ l.elements=l.elements.filter(el=>!selection.has(el.id)); } selection.clear(); pushHistory('delete'); rebuildLayerPanel(); render(); syncInfo(); });

  if(exportTransparentChk) exportTransparentChk.addEventListener('change', ()=>{ exportBgInput.disabled=exportTransparentChk.checked; });
  if(doExportBtn) doExportBtn.addEventListener('click', handleExport);

  function computeExportBounds(){ // basic union of all element bounds
    let xs=[], ys=[]; for(const l of layers){ if(!l.visible) continue; for(const el of l.elements){ if(el.type==='rect'){ xs.push(el.x,el.x+el.w); ys.push(el.y,el.y+el.h);} else if(el.type==='path'){ for(const p of el.points){ xs.push(p.x); ys.push(p.y);} } } }
    if(!xs.length) return {x:0,y:0,w:512,h:512};
    const minx=Math.min(...xs), maxx=Math.max(...xs), miny=Math.min(...ys), maxy=Math.max(...ys); return {x:minx,y:miny,w:maxx-minx,h:maxy-miny};
  }

  // --- existing export logic continues (no change) ---
  function handleExport(){ // PNG/JPEG 無効化 -> 常にSVG
    if(exportFormatSel) exportFormatSel.value='svg';
    const bounds=computeExportBounds(); exportAsSVG(bounds); }
  function exportAsRaster(bounds,fmt,scaleFactor){ const out=document.createElement('canvas'); const pxScale=window.devicePixelRatio||1; out.width=Math.ceil(bounds.w*pxScale*scaleFactor); out.height=Math.ceil(bounds.h*pxScale*scaleFactor); const octx=out.getContext('2d'); if(!exportTransparentChk.checked){ octx.fillStyle=exportBgInput.value; octx.fillRect(0,0,out.width,out.height);} octx.setTransform(pxScale*scaleFactor,0,0,pxScale*scaleFactor,-bounds.x*pxScale*scaleFactor,-bounds.y*pxScale*scaleFactor); for(const l of layers){ if(!l.visible) continue; octx.save(); octx.globalAlpha=l.opacity; octx.globalCompositeOperation=l.blend; for(const el of l.elements){ if(el.type==='path'){ if(el.brushShape==='dotted'){ octx.fillStyle=el.stroke; const step=el.strokeWidth*1.5; for(let i=1;i<el.points.length;i++){ const a=el.points[i-1], b=el.points[i]; const dx=b.x-a.x, dy=b.y-a.y, dist=Math.hypot(dx,dy); const segs=Math.max(1,Math.floor(dist/step)); for(let s=0;s<=segs;s++){ const t=s/segs; const px=a.x+dx*t, py=a.y+dy*t; octx.beginPath(); octx.arc(px,py,el.strokeWidth/2,0,Math.PI*2); octx.fill(); } } } else { octx.beginPath(); octx.lineCap=(el.brushShape==='square')?'butt':'round'; octx.lineJoin=(el.brushShape==='square')?'miter':'round'; octx.lineWidth=el.strokeWidth; octx.strokeStyle=el.stroke; if(el.strokeOpacity!=null) octx.globalAlpha*=el.strokeOpacity; octx.moveTo(el.points[0].x,el.points[0].y); for(let i=1;i<el.points.length;i++) octx.lineTo(el.points[i].x,el.points[i].y); octx.stroke(); } } else if(el.type==='rect'){ octx.lineWidth=el.strokeWidth||1; octx.strokeStyle=el.stroke||'#000'; if(el.strokeOpacity!=null) octx.globalAlpha*=el.strokeOpacity; octx.strokeRect(el.x,el.y,el.w,el.h); } } octx.restore(); }
    const mime=fmt==='jpeg'?'image/jpeg':'image/png'; const data=out.toDataURL(mime, fmt==='jpeg'?0.92:undefined); openDataInNewTab(data); }
  function exportAsSVG(bounds){ const esc=s=>String(s).replace(/["&<>]/g,c=>({'"':'&quot;','&':'&amp;','<':'&lt;','>':'&gt;'}[c])); let body=''; for(const l of layers){ if(!l.visible) continue; body+=`<g opacity="${l.opacity}" mix-blend-mode="${esc(l.blend)}">`; for(const el of l.elements){ if(el.type==='rect'){ body+=`<rect x="${el.x-bounds.x}" y="${el.y-bounds.y}" width="${el.w}" height="${el.h}" fill="none" stroke="${esc(el.stroke)}" stroke-width="${el.strokeWidth}" stroke-linejoin="round" stroke-linecap="round" />`; } else if(el.type==='path'&&el.points.length){ const d='M'+el.points.map((p,i)=>(i?'L':'')+(p.x-bounds.x)+' '+(p.y-bounds.y)).join(' '); body+=`<path d="${d}" fill="none" stroke="${esc(el.stroke)}" stroke-width="${el.strokeWidth}" stroke-linejoin="round" stroke-linecap="${el.brushShape==='square'?'butt':'round'}" />`; } } body+='</g>'; } if(!body) body='<rect x="0" y="0" width="512" height="512" fill="none" />'; const bg=(!exportTransparentChk.checked)? `<rect x="0" y="0" width="${bounds.w}" height="${bounds.h}" fill="${esc(exportBgInput.value)}" />`:''; const svg=`<?xml version="1.0" encoding="UTF-8"?>\n<svg xmlns="http://www.w3.org/2000/svg" width="${bounds.w}" height="${bounds.h}" viewBox="0 0 ${bounds.w} ${bounds.h}">${bg}${body}</svg>`; const blob=new Blob([svg],{type:'image/svg+xml'}); const url=URL.createObjectURL(blob); openDataInNewTab(url); setTimeout(()=>URL.revokeObjectURL(url),5000); }
  function openDataInNewTab(dataUrl){ const win=window.open('about:blank','_blank'); if(win){ if(dataUrl.startsWith('blob:')){ const iframe=document.createElement('iframe'); iframe.style.width='100%'; iframe.style.height='100%'; iframe.src=dataUrl; win.document.body.appendChild(iframe);} else { win.document.body.innerHTML='<img style="max-width:100%;image-rendering:pixelated;" src="'+dataUrl+'">'; } } }

  // Mini layer popup support (legacy calls safe no-op if not present in new UI)
  function rebuildLayerMini(){}
  function rebuildLayerPanel(){ if(!layerListEl) return; layerListEl.innerHTML=''; for(let i=layers.length-1;i>=0;i--){ const l=layers[i]; const item=document.createElement('div'); item.className='layer-item'+(l.id===activeLayerId?' active':''); item.dataset.id=l.id; const vis=document.createElement('div'); vis.className='vis-toggle'; vis.textContent=l.visible?'👁':'🚫'; vis.title='表示切替'; vis.addEventListener('click',e=>{ e.stopPropagation(); l.visible=!l.visible; vis.textContent=l.visible?'👁':'🚫'; pushHistory('layer-vis'); render(); rebuildLayerPanel(); }); const meta=document.createElement('div'); meta.className='layer-meta'; const nameInput=document.createElement('input'); nameInput.className='name'; nameInput.value=l.name; nameInput.addEventListener('change',()=>{ l.name=nameInput.value.trim()||l.name; pushHistory('layer-rename'); rebuildLayerPanel(); }); meta.appendChild(nameInput); // blend select
      const blendSel=document.createElement('select'); blendSel.className='blend'; ['source-over','multiply','screen','overlay','darken','lighten','color-dodge','color-burn','difference','exclusion','hue','saturation','color','luminosity'].forEach(m=>{ const op=document.createElement('option'); op.value=m; op.textContent=m; if(m===l.blend) op.selected=true; blendSel.appendChild(op); }); blendSel.addEventListener('change',()=>{ l.blend=blendSel.value; pushHistory('layer-blend'); render(); }); meta.appendChild(blendSel); // opacity slider
      const opWrap=document.createElement('div'); opWrap.style.display='flex'; opWrap.style.alignItems='center'; opWrap.style.gap='4px'; const opLab=document.createElement('span'); opLab.style.fontSize='10px'; opLab.textContent=Math.round(l.opacity*100)+'%'; const opRange=document.createElement('input'); opRange.type='range'; opRange.min=0; opRange.max=1; opRange.step=0.01; opRange.value=l.opacity; opRange.style.flex='1'; opRange.addEventListener('input',()=>{ l.opacity=Number(opRange.value); opLab.textContent=Math.round(l.opacity*100)+'%'; render(); }); opRange.addEventListener('change',()=>{ pushHistory('layer-opacity'); }); opWrap.appendChild(opRange); opWrap.appendChild(opLab); meta.appendChild(opWrap); const info=document.createElement('div'); info.style.fontSize='10px'; info.style.opacity=.7; info.textContent=`objs:${l.elements.length}`; meta.appendChild(info); item.appendChild(vis); item.appendChild(meta); item.addEventListener('click',()=>{ activeLayerId=l.id; rebuildLayerPanel(); updateStatusBar(); }); item.addEventListener('dblclick',()=>{ // レイヤー全選択
        selection.clear(); for(const el of l.elements) selection.add(el.id); pushHistory('layer-select-all'); render(); updateStatusBar(); }); layerListEl.appendChild(item); } }
  function rebuildHistoryPanel(){ if(!historyListEl) return; historyListEl.innerHTML=''; history.forEach((snap,i)=>{ const b=document.createElement('button'); b.textContent=(snap._label||'state')+' #'+i; if(i===historyIndex) b.classList.add('active'); b.addEventListener('click',()=>{ historyIndex=i; restore(history[i]); updateHistoryButtons(); }); historyListEl.appendChild(b); }); }

  // Layer panel buttons
  if(btnLayerNew) btnLayerNew.addEventListener('click',()=>{ createLayer({}); render(); });
  if(btnLayerDel) btnLayerDel.addEventListener('click',()=>{ deleteActiveLayer(); render(); });
  if(btnDupLayer) btnDupLayer.addEventListener('click',()=>{ const l=getActiveLayer(); if(!l) return; const clone={id:'L'+Math.random().toString(36).slice(2,8), name:l.name+' Copy', visible:l.visible, opacity:l.opacity, blend:l.blend, elements:l.elements.map(cloneElement)}; layers.push(clone); activeLayerId=clone.id; rebuildLayerSelect(); pushHistory('layer-dup'); render(); });
  if(btnLayerUp) btnLayerUp.addEventListener('click',()=>{ const idx=layers.findIndex(l=>l.id===activeLayerId); if(idx<0||idx>=layers.length-1) return; const tmp=layers[idx]; layers[idx]=layers[idx+1]; layers[idx+1]=tmp; rebuildLayerSelect(); pushHistory('layer-up'); render(); });
  if(btnLayerDown) btnLayerDown.addEventListener('click',()=>{ const idx=layers.findIndex(l=>l.id===activeLayerId); if(idx<=0) return; const tmp=layers[idx]; layers[idx]=layers[idx-1]; layers[idx-1]=tmp; rebuildLayerSelect(); pushHistory('layer-down'); render(); });
  if(btnLayerCopy) btnLayerCopy.addEventListener('click',()=>{ const l=getActiveLayer(); if(!l) return; VP._layerClipboard=cloneLayer(l); });
  if(btnLayerPaste) btnLayerPaste.addEventListener('click',()=>{ if(!VP._layerClipboard) return; const c=VP._layerClipboard; const clone={...cloneLayer(c), id:'L'+Math.random().toString(36).slice(2,8), name:c.name+' Paste'}; layers.push(clone); activeLayerId=clone.id; rebuildLayerSelect(); pushHistory('layer-paste'); render(); });
  if(btnUndo) btnUndo.addEventListener('click',()=>undo());
  if(btnRedo) btnRedo.addEventListener('click',()=>redo());

  // Init if not already
  if(!layers.length){ createLayer({name:'Layer1'}); rebuildLayerSelect(); rebuildHistoryPanel(); }

  // Public API for new UI panels
  VP.requestRender=()=>render();
  VP.allElementIds=()=> allElements().map(e=>e.id);
  VP.getSelection=()=> new Set(selection);
  VP.setSelection=(set,record)=>{ selection=set; if(record) pushHistory('set-selection'); render(); updateStatusBar(); };
  VP.clearSelectionRegions=()=>{ selectionRegions=[]; pushHistory('clear-sel-regions'); render(); };
  VP.setBrushOpacity=(v)=>{ brushOpacity=v; };
  VP.applyStrokePropsToSelection=()=>{ if(!selection.size) return; for(const l of layers){ for(const el of l.elements){ if(selection.has(el.id)){ el.stroke=colorInput.value; el.strokeWidth=Number(widthInput.value); el.strokeOpacity=brushOpacity; }}} pushHistory('apply-stroke'); render(); };
  VP.currentBrushShape='round'; Object.defineProperty(VP,'currentBrushShape',{get(){return currentBrushShape;},set(v){ currentBrushShape=v; }});
  VP.setTool=(t)=>{ toolSel.value=t; toolSel.dispatchEvent(new Event('change')); };

  // initial render
  // パフォーマンス計測 loop
  let lastT=performance.now(), frameCount=0, acc=0; function perfLoop(t){ const dt=t-lastT; lastT=t; frameCount++; acc+=dt; if(acc>=500){ const fps=(frameCount*1000/acc).toFixed(0); const ms=(acc/frameCount).toFixed(1); if(statusSpans.fps) statusSpans.fps.textContent='fps:'+fps; if(statusSpans.ms) statusSpans.ms.textContent='ms:'+ms; // memory
      if(performance.memory && statusSpans.mem){ const used=(performance.memory.usedJSHeapSize/1048576).toFixed(1); statusSpans.mem.textContent='mem:'+used+'M'; }
      frameCount=0; acc=0; }
    requestAnimationFrame(perfLoop); }
  requestAnimationFrame(perfLoop);
  render(); updateStatusBar(); rebuildHistoryPanel();
})();
