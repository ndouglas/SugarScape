/** Real layout assertions for the mounted episode UI; run in a browser, not Node. */
export function episodeResponsiveChecks(root:HTMLElement):{
 checks:{name:string;passed:boolean}[];
 viewport:number;pageWidth:number;episodeWidth:number;gridTrack:string;
 children:{class:string;width:number;right:number}[];
 map:{client:number;scroll:number;overflow:string}|null;
} {
 const edge=root.getBoundingClientRect(),children=[...root.children].filter(node=>node.getBoundingClientRect().width>0).map(node=>({class:node.className,width:node.getBoundingClientRect().width,right:node.getBoundingClientRect().right}));
 const viewport=root.querySelector<HTMLElement>('.spatial-viewport');
 const map=viewport?{client:viewport.clientWidth,scroll:viewport.scrollWidth,overflow:getComputedStyle(viewport).overflowX}:null;
 const pageWidth=Math.max(document.body.scrollWidth,document.documentElement.scrollWidth);
 return {
  viewport:innerWidth,pageWidth,episodeWidth:root.clientWidth,gridTrack:getComputedStyle(root).gridTemplateColumns,children,map,
  checks:[
   {name:'Episode page stays within viewport',passed:pageWidth<=innerWidth},
   {name:'Episode direct children fit the available column',passed:children.every(child=>child.right<=edge.right+1&&child.width<=edge.width+1)},
   {name:'Episode content remains visible rather than clipped',passed:!['hidden','clip'].includes(getComputedStyle(root).overflowX)},
   {name:'Spatial map retains its internal scrolling viewport',passed:map!==null&&map.overflow==='auto'},
  ],
 };
}
