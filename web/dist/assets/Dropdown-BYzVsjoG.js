import{f as Re,B as Ce,V as Ie,j as Ke,h as ue,r as _e,N as Oe,u as $e,p as fe,d as ze}from"./Popover-BT2YCJ3w.js";import{b9 as De,q as Ae,aM as q,ae as de,b0 as G,ba as je,bb as Fe,ad as Te,r as T,d as K,C as a,I as ae,aF as J,U as B,s as k,v as P,x as D,b3 as he,ar as le,E as ve,G as Z,R as be,p as w,bc as Be,b7 as Me,aN as X,bd as me,J as H,P as Le,be as Ee,bf as He,bg as Ue,b6 as Ve,A as ce,y as z,aS as We,X as re,V as I,bh as qe,W as F}from"./index-DQgClpag.js";function Ge(e={},n){const i=Ae({ctrl:!1,command:!1,win:!1,shift:!1,tab:!1}),{keydown:r,keyup:t}=e,o=d=>{switch(d.key){case"Control":i.ctrl=!0;break;case"Meta":i.command=!0,i.win=!0;break;case"Shift":i.shift=!0;break;case"Tab":i.tab=!0;break}r!==void 0&&Object.keys(r).forEach(y=>{if(y!==d.key)return;const h=r[y];if(typeof h=="function")h(d);else{const{stop:g=!1,prevent:S=!1}=h;g&&d.stopPropagation(),S&&d.preventDefault(),h.handler(d)}})},s=d=>{switch(d.key){case"Control":i.ctrl=!1;break;case"Meta":i.command=!1,i.win=!1;break;case"Shift":i.shift=!1;break;case"Tab":i.tab=!1;break}t!==void 0&&Object.keys(t).forEach(y=>{if(y!==d.key)return;const h=t[y];if(typeof h=="function")h(d);else{const{stop:g=!1,prevent:S=!1}=h;g&&d.stopPropagation(),S&&d.preventDefault(),h.handler(d)}})},c=()=>{(n===void 0||n.value)&&(q("keydown",document,o),q("keyup",document,s)),n!==void 0&&de(n,d=>{d?(q("keydown",document,o),q("keyup",document,s)):(G("keydown",document,o),G("keyup",document,s))})};return je()?(Fe(c),Te(()=>{(n===void 0||n.value)&&(G("keydown",document,o),G("keyup",document,s))})):c(),De(i)}function Xe(e,n,i){const r=T(e.value);let t=null;return de(e,o=>{t!==null&&window.clearTimeout(t),o===!0?i&&!i.value?r.value=!0:t=window.setTimeout(()=>{r.value=!0},n):r.value=!1}),r}function Je(e){return n=>{n?e.value=n.$el:e.value=null}}const Ze=K({name:"ChevronRight",render(){return a("svg",{viewBox:"0 0 16 16",fill:"none",xmlns:"http://www.w3.org/2000/svg"},a("path",{d:"M5.64645 3.14645C5.45118 3.34171 5.45118 3.65829 5.64645 3.85355L9.79289 8L5.64645 12.1464C5.45118 12.3417 5.45118 12.6583 5.64645 12.8536C5.84171 13.0488 6.15829 13.0488 6.35355 12.8536L10.8536 8.35355C11.0488 8.15829 11.0488 7.84171 10.8536 7.64645L6.35355 3.14645C6.15829 2.95118 5.84171 2.95118 5.64645 3.14645Z",fill:"currentColor"}))}}),se=ae("n-dropdown-menu"),Q=ae("n-dropdown"),pe=ae("n-dropdown-option"),we=K({name:"DropdownDivider",props:{clsPrefix:{type:String,required:!0}},render(){return a("div",{class:`${this.clsPrefix}-dropdown-divider`})}}),Qe=K({name:"DropdownGroupHeader",props:{clsPrefix:{type:String,required:!0},tmNode:{type:Object,required:!0}},setup(){const{showIconRef:e,hasSubmenuRef:n}=B(se),{renderLabelRef:i,labelFieldRef:r,nodePropsRef:t,renderOptionRef:o}=B(Q);return{labelField:r,showIcon:e,hasSubmenu:n,renderLabel:i,nodeProps:t,renderOption:o}},render(){var e;const{clsPrefix:n,hasSubmenu:i,showIcon:r,nodeProps:t,renderLabel:o,renderOption:s}=this,{rawNode:c}=this.tmNode,d=a("div",Object.assign({class:`${n}-dropdown-option`},t==null?void 0:t(c)),a("div",{class:`${n}-dropdown-option-body ${n}-dropdown-option-body--group`},a("div",{"data-dropdown-option":!0,class:[`${n}-dropdown-option-body__prefix`,r&&`${n}-dropdown-option-body__prefix--show-icon`]},J(c.icon)),a("div",{class:`${n}-dropdown-option-body__label`,"data-dropdown-option":!0},o?o(c):J((e=c.title)!==null&&e!==void 0?e:c[this.labelField])),a("div",{class:[`${n}-dropdown-option-body__suffix`,i&&`${n}-dropdown-option-body__suffix--has-submenu`],"data-dropdown-option":!0})));return s?s({node:d,option:c}):d}}),Ye=k("icon",`
 height: 1em;
 width: 1em;
 line-height: 1em;
 text-align: center;
 display: inline-block;
 position: relative;
 fill: currentColor;
`,[P("color-transition",{transition:"color .3s var(--n-bezier)"}),P("depth",{color:"var(--n-color)"},[D("svg",{opacity:"var(--n-opacity)",transition:"opacity .3s var(--n-bezier)"})]),D("svg",{height:"1em",width:"1em"})]),eo=Object.assign(Object.assign({},Z.props),{depth:[String,Number],size:[Number,String],color:String,component:[Object,Function]}),oo=K({_n_icon__:!0,name:"Icon",inheritAttrs:!1,props:eo,setup(e){const{mergedClsPrefixRef:n,inlineThemeDisabled:i}=ve(e),r=Z("Icon","-icon",Ye,Be,e,n),t=w(()=>{const{depth:s}=e,{common:{cubicBezierEaseInOut:c},self:d}=r.value;if(s!==void 0){const{color:y,[`opacity${s}Depth`]:h}=d;return{"--n-bezier":c,"--n-color":y,"--n-opacity":h}}return{"--n-bezier":c,"--n-color":"","--n-opacity":""}}),o=i?be("icon",w(()=>`${e.depth||"d"}`),t,e):void 0;return{mergedClsPrefix:n,mergedStyle:w(()=>{const{size:s,color:c}=e;return{fontSize:Re(s),color:c}}),cssVars:i?void 0:t,themeClass:o==null?void 0:o.themeClass,onRender:o==null?void 0:o.onRender}},render(){var e;const{$parent:n,depth:i,mergedClsPrefix:r,component:t,onRender:o,themeClass:s}=this;return!((e=n==null?void 0:n.$options)===null||e===void 0)&&e._n_icon__&&he("icon","don't wrap `n-icon` inside `n-icon`"),o==null||o(),a("i",le(this.$attrs,{role:"img",class:[`${r}-icon`,s,{[`${r}-icon--depth`]:i,[`${r}-icon--color-transition`]:i!==void 0}],style:[this.cssVars,this.mergedStyle]}),t?a(t):this.$slots)}});function ie(e,n){return e.type==="submenu"||e.type===void 0&&e[n]!==void 0}function no(e){return e.type==="group"}function ye(e){return e.type==="divider"}function to(e){return e.type==="render"}const ge=K({name:"DropdownOption",props:{clsPrefix:{type:String,required:!0},tmNode:{type:Object,required:!0},parentKey:{type:[String,Number],default:null},placement:{type:String,default:"right-start"},props:Object,scrollable:Boolean},setup(e){const n=B(Q),{hoverKeyRef:i,keyboardKeyRef:r,lastToggledSubmenuKeyRef:t,pendingKeyPathRef:o,activeKeyPathRef:s,animatedRef:c,mergedShowRef:d,renderLabelRef:y,renderIconRef:h,labelFieldRef:g,childrenFieldRef:S,renderOptionRef:R,nodePropsRef:_,menuPropsRef:M}=n,x=B(pe,null),O=B(se),U=B(me),Y=w(()=>e.tmNode.rawNode),V=w(()=>{const{value:u}=S;return ie(e.tmNode.rawNode,u)}),ee=w(()=>{const{disabled:u}=e.tmNode;return u}),oe=w(()=>{if(!V.value)return!1;const{key:u,disabled:m}=e.tmNode;if(m)return!1;const{value:C}=i,{value:A}=r,{value:te}=t,{value:j}=o;return C!==null?j.includes(u):A!==null?j.includes(u)&&j[j.length-1]!==u:te!==null?j.includes(u):!1}),ne=w(()=>r.value===null&&!c.value),W=Xe(oe,300,ne),L=w(()=>!!(x!=null&&x.enteringSubmenuRef.value)),E=T(!1);H(pe,{enteringSubmenuRef:E});function $(){E.value=!0}function l(){E.value=!1}function b(){const{parentKey:u,tmNode:m}=e;m.disabled||d.value&&(t.value=u,r.value=null,i.value=m.key)}function f(){const{tmNode:u}=e;u.disabled||d.value&&i.value!==u.key&&b()}function p(u){if(e.tmNode.disabled||!d.value)return;const{relatedTarget:m}=u;m&&!ue({target:m},"dropdownOption")&&!ue({target:m},"scrollbarRail")&&(i.value=null)}function N(){const{value:u}=V,{tmNode:m}=e;d.value&&!u&&!m.disabled&&(n.doSelect(m.key,m.rawNode),n.doUpdateShow(!1))}return{labelField:g,renderLabel:y,renderIcon:h,siblingHasIcon:O.showIconRef,siblingHasSubmenu:O.hasSubmenuRef,menuProps:M,popoverBody:U,animated:c,mergedShowSubmenu:w(()=>W.value&&!L.value),rawNode:Y,hasSubmenu:V,pending:X(()=>{const{value:u}=o,{key:m}=e.tmNode;return u.includes(m)}),childActive:X(()=>{const{value:u}=s,{key:m}=e.tmNode,C=u.findIndex(A=>m===A);return C===-1?!1:C<u.length-1}),active:X(()=>{const{value:u}=s,{key:m}=e.tmNode,C=u.findIndex(A=>m===A);return C===-1?!1:C===u.length-1}),mergedDisabled:ee,renderOption:R,nodeProps:_,handleClick:N,handleMouseMove:f,handleMouseEnter:b,handleMouseLeave:p,handleSubmenuBeforeEnter:$,handleSubmenuAfterEnter:l}},render(){var e,n;const{animated:i,rawNode:r,mergedShowSubmenu:t,clsPrefix:o,siblingHasIcon:s,siblingHasSubmenu:c,renderLabel:d,renderIcon:y,renderOption:h,nodeProps:g,props:S,scrollable:R}=this;let _=null;if(t){const U=(e=this.menuProps)===null||e===void 0?void 0:e.call(this,r,r.children);_=a(xe,Object.assign({},U,{clsPrefix:o,scrollable:this.scrollable,tmNodes:this.tmNode.children,parentKey:this.tmNode.key}))}const M={class:[`${o}-dropdown-option-body`,this.pending&&`${o}-dropdown-option-body--pending`,this.active&&`${o}-dropdown-option-body--active`,this.childActive&&`${o}-dropdown-option-body--child-active`,this.mergedDisabled&&`${o}-dropdown-option-body--disabled`],onMousemove:this.handleMouseMove,onMouseenter:this.handleMouseEnter,onMouseleave:this.handleMouseLeave,onClick:this.handleClick},x=g==null?void 0:g(r),O=a("div",Object.assign({class:[`${o}-dropdown-option`,x==null?void 0:x.class],"data-dropdown-option":!0},x),a("div",le(M,S),[a("div",{class:[`${o}-dropdown-option-body__prefix`,s&&`${o}-dropdown-option-body__prefix--show-icon`]},[y?y(r):J(r.icon)]),a("div",{"data-dropdown-option":!0,class:`${o}-dropdown-option-body__label`},d?d(r):J((n=r[this.labelField])!==null&&n!==void 0?n:r.title)),a("div",{"data-dropdown-option":!0,class:[`${o}-dropdown-option-body__suffix`,c&&`${o}-dropdown-option-body__suffix--has-submenu`]},this.hasSubmenu?a(oo,null,{default:()=>a(Ze,null)}):null)]),this.hasSubmenu?a(Ce,null,{default:()=>[a(Ie,null,{default:()=>a("div",{class:`${o}-dropdown-offset-container`},a(Ke,{show:this.mergedShowSubmenu,placement:this.placement,to:R&&this.popoverBody||void 0,teleportDisabled:!R},{default:()=>a("div",{class:`${o}-dropdown-menu-wrapper`},i?a(Me,{onBeforeEnter:this.handleSubmenuBeforeEnter,onAfterEnter:this.handleSubmenuAfterEnter,name:"fade-in-scale-up-transition",appear:!0},{default:()=>_}):_)}))})]}):null);return h?h({node:O,option:r}):O}}),ro=K({name:"NDropdownGroup",props:{clsPrefix:{type:String,required:!0},tmNode:{type:Object,required:!0},parentKey:{type:[String,Number],default:null}},render(){const{tmNode:e,parentKey:n,clsPrefix:i}=this,{children:r}=e;return a(Le,null,a(Qe,{clsPrefix:i,tmNode:e,key:e.key}),r==null?void 0:r.map(t=>{const{rawNode:o}=t;return o.show===!1?null:ye(o)?a(we,{clsPrefix:i,key:t.key}):t.isGroup?(he("dropdown","`group` node is not allowed to be put in `group` node."),null):a(ge,{clsPrefix:i,tmNode:t,parentKey:n,key:t.key})}))}}),io=K({name:"DropdownRenderOption",props:{tmNode:{type:Object,required:!0}},render(){const{rawNode:{render:e,props:n}}=this.tmNode;return a("div",n,[e==null?void 0:e()])}}),xe=K({name:"DropdownMenu",props:{scrollable:Boolean,showArrow:Boolean,arrowStyle:[String,Object],clsPrefix:{type:String,required:!0},tmNodes:{type:Array,default:()=>[]},parentKey:{type:[String,Number],default:null}},setup(e){const{renderIconRef:n,childrenFieldRef:i}=B(Q);H(se,{showIconRef:w(()=>{const t=n.value;return e.tmNodes.some(o=>{var s;if(o.isGroup)return(s=o.children)===null||s===void 0?void 0:s.some(({rawNode:d})=>t?t(d):d.icon);const{rawNode:c}=o;return t?t(c):c.icon})}),hasSubmenuRef:w(()=>{const{value:t}=i;return e.tmNodes.some(o=>{var s;if(o.isGroup)return(s=o.children)===null||s===void 0?void 0:s.some(({rawNode:d})=>ie(d,t));const{rawNode:c}=o;return ie(c,t)})})});const r=T(null);return H(He,null),H(Ue,null),H(me,r),{bodyRef:r}},render(){const{parentKey:e,clsPrefix:n,scrollable:i}=this,r=this.tmNodes.map(t=>{const{rawNode:o}=t;return o.show===!1?null:to(o)?a(io,{tmNode:t,key:t.key}):ye(o)?a(we,{clsPrefix:n,key:t.key}):no(o)?a(ro,{clsPrefix:n,tmNode:t,parentKey:e,key:t.key}):a(ge,{clsPrefix:n,tmNode:t,parentKey:e,key:t.key,props:o.props,scrollable:i})});return a("div",{class:[`${n}-dropdown-menu`,i&&`${n}-dropdown-menu--scrollable`],ref:"bodyRef"},i?a(Ee,{contentClass:`${n}-dropdown-menu__content`},{default:()=>r}):r,this.showArrow?_e({clsPrefix:n,arrowStyle:this.arrowStyle,arrowClass:void 0,arrowWrapperClass:void 0,arrowWrapperStyle:void 0}):null)}}),ao=k("dropdown-menu",`
 transform-origin: var(--v-transform-origin);
 background-color: var(--n-color);
 border-radius: var(--n-border-radius);
 box-shadow: var(--n-box-shadow);
 position: relative;
 transition:
 background-color .3s var(--n-bezier),
 box-shadow .3s var(--n-bezier);
`,[Ve(),k("dropdown-option",`
 position: relative;
 `,[D("a",`
 text-decoration: none;
 color: inherit;
 outline: none;
 `,[D("&::before",`
 content: "";
 position: absolute;
 left: 0;
 right: 0;
 top: 0;
 bottom: 0;
 `)]),k("dropdown-option-body",`
 display: flex;
 cursor: pointer;
 position: relative;
 height: var(--n-option-height);
 line-height: var(--n-option-height);
 font-size: var(--n-font-size);
 color: var(--n-option-text-color);
 transition: color .3s var(--n-bezier);
 `,[D("&::before",`
 content: "";
 position: absolute;
 top: 0;
 bottom: 0;
 left: 4px;
 right: 4px;
 transition: background-color .3s var(--n-bezier);
 border-radius: var(--n-border-radius);
 `),ce("disabled",[P("pending",`
 color: var(--n-option-text-color-hover);
 `,[z("prefix, suffix",`
 color: var(--n-option-text-color-hover);
 `),D("&::before","background-color: var(--n-option-color-hover);")]),P("active",`
 color: var(--n-option-text-color-active);
 `,[z("prefix, suffix",`
 color: var(--n-option-text-color-active);
 `),D("&::before","background-color: var(--n-option-color-active);")]),P("child-active",`
 color: var(--n-option-text-color-child-active);
 `,[z("prefix, suffix",`
 color: var(--n-option-text-color-child-active);
 `)])]),P("disabled",`
 cursor: not-allowed;
 opacity: var(--n-option-opacity-disabled);
 `),P("group",`
 font-size: calc(var(--n-font-size) - 1px);
 color: var(--n-group-header-text-color);
 `,[z("prefix",`
 width: calc(var(--n-option-prefix-width) / 2);
 `,[P("show-icon",`
 width: calc(var(--n-option-icon-prefix-width) / 2);
 `)])]),z("prefix",`
 width: var(--n-option-prefix-width);
 display: flex;
 justify-content: center;
 align-items: center;
 color: var(--n-prefix-color);
 transition: color .3s var(--n-bezier);
 z-index: 1;
 `,[P("show-icon",`
 width: var(--n-option-icon-prefix-width);
 `),k("icon",`
 font-size: var(--n-option-icon-size);
 `)]),z("label",`
 white-space: nowrap;
 flex: 1;
 z-index: 1;
 `),z("suffix",`
 box-sizing: border-box;
 flex-grow: 0;
 flex-shrink: 0;
 display: flex;
 justify-content: flex-end;
 align-items: center;
 min-width: var(--n-option-suffix-width);
 padding: 0 8px;
 transition: color .3s var(--n-bezier);
 color: var(--n-suffix-color);
 z-index: 1;
 `,[P("has-submenu",`
 width: var(--n-option-icon-suffix-width);
 `),k("icon",`
 font-size: var(--n-option-icon-size);
 `)]),k("dropdown-menu","pointer-events: all;")]),k("dropdown-offset-container",`
 pointer-events: none;
 position: absolute;
 left: 0;
 right: 0;
 top: -4px;
 bottom: -4px;
 `)]),k("dropdown-divider",`
 transition: background-color .3s var(--n-bezier);
 background-color: var(--n-divider-color);
 height: 1px;
 margin: 4px 0;
 `),k("dropdown-menu-wrapper",`
 transform-origin: var(--v-transform-origin);
 width: fit-content;
 `),D(">",[k("scrollbar",`
 height: inherit;
 max-height: inherit;
 `)]),ce("scrollable",`
 padding: var(--n-padding);
 `),P("scrollable",[z("content",`
 padding: var(--n-padding);
 `)])]),lo={animated:{type:Boolean,default:!0},keyboard:{type:Boolean,default:!0},size:{type:String,default:"medium"},inverted:Boolean,placement:{type:String,default:"bottom"},onSelect:[Function,Array],options:{type:Array,default:()=>[]},menuProps:Function,showArrow:Boolean,renderLabel:Function,renderIcon:Function,renderOption:Function,nodeProps:Function,labelField:{type:String,default:"label"},keyField:{type:String,default:"key"},childrenField:{type:String,default:"children"},value:[String,Number]},so=Object.keys(fe),uo=Object.assign(Object.assign(Object.assign({},fe),lo),Z.props),fo=K({name:"Dropdown",inheritAttrs:!1,props:uo,setup(e){const n=T(!1),i=$e(I(e,"show"),n),r=w(()=>{const{keyField:l,childrenField:b}=e;return ze(e.options,{getKey(f){return f[l]},getDisabled(f){return f.disabled===!0},getIgnored(f){return f.type==="divider"||f.type==="render"},getChildren(f){return f[b]}})}),t=w(()=>r.value.treeNodes),o=T(null),s=T(null),c=T(null),d=w(()=>{var l,b,f;return(f=(b=(l=o.value)!==null&&l!==void 0?l:s.value)!==null&&b!==void 0?b:c.value)!==null&&f!==void 0?f:null}),y=w(()=>r.value.getPath(d.value).keyPath),h=w(()=>r.value.getPath(e.value).keyPath),g=X(()=>e.keyboard&&i.value);Ge({keydown:{ArrowUp:{prevent:!0,handler:ee},ArrowRight:{prevent:!0,handler:V},ArrowDown:{prevent:!0,handler:oe},ArrowLeft:{prevent:!0,handler:Y},Enter:{prevent:!0,handler:ne},Escape:U}},g);const{mergedClsPrefixRef:S,inlineThemeDisabled:R}=ve(e),_=Z("Dropdown","-dropdown",ao,qe,e,S);H(Q,{labelFieldRef:I(e,"labelField"),childrenFieldRef:I(e,"childrenField"),renderLabelRef:I(e,"renderLabel"),renderIconRef:I(e,"renderIcon"),hoverKeyRef:o,keyboardKeyRef:s,lastToggledSubmenuKeyRef:c,pendingKeyPathRef:y,activeKeyPathRef:h,animatedRef:I(e,"animated"),mergedShowRef:i,nodePropsRef:I(e,"nodeProps"),renderOptionRef:I(e,"renderOption"),menuPropsRef:I(e,"menuProps"),doSelect:M,doUpdateShow:x}),de(i,l=>{!e.animated&&!l&&O()});function M(l,b){const{onSelect:f}=e;f&&re(f,l,b)}function x(l){const{"onUpdate:show":b,onUpdateShow:f}=e;b&&re(b,l),f&&re(f,l),n.value=l}function O(){o.value=null,s.value=null,c.value=null}function U(){x(!1)}function Y(){L("left")}function V(){L("right")}function ee(){L("up")}function oe(){L("down")}function ne(){const l=W();l!=null&&l.isLeaf&&i.value&&(M(l.key,l.rawNode),x(!1))}function W(){var l;const{value:b}=r,{value:f}=d;return!b||f===null?null:(l=b.getNode(f))!==null&&l!==void 0?l:null}function L(l){const{value:b}=d,{value:{getFirstAvailableNode:f}}=r;let p=null;if(b===null){const N=f();N!==null&&(p=N.key)}else{const N=W();if(N){let u;switch(l){case"down":u=N.getNext();break;case"up":u=N.getPrev();break;case"right":u=N.getChild();break;case"left":u=N.getParent();break}u&&(p=u.key)}}p!==null&&(o.value=null,s.value=p)}const E=w(()=>{const{size:l,inverted:b}=e,{common:{cubicBezierEaseInOut:f},self:p}=_.value,{padding:N,dividerColor:u,borderRadius:m,optionOpacityDisabled:C,[F("optionIconSuffixWidth",l)]:A,[F("optionSuffixWidth",l)]:te,[F("optionIconPrefixWidth",l)]:j,[F("optionPrefixWidth",l)]:Se,[F("fontSize",l)]:Ne,[F("optionHeight",l)]:ke,[F("optionIconSize",l)]:Pe}=p,v={"--n-bezier":f,"--n-font-size":Ne,"--n-padding":N,"--n-border-radius":m,"--n-option-height":ke,"--n-option-prefix-width":Se,"--n-option-icon-prefix-width":j,"--n-option-suffix-width":te,"--n-option-icon-suffix-width":A,"--n-option-icon-size":Pe,"--n-divider-color":u,"--n-option-opacity-disabled":C};return b?(v["--n-color"]=p.colorInverted,v["--n-option-color-hover"]=p.optionColorHoverInverted,v["--n-option-color-active"]=p.optionColorActiveInverted,v["--n-option-text-color"]=p.optionTextColorInverted,v["--n-option-text-color-hover"]=p.optionTextColorHoverInverted,v["--n-option-text-color-active"]=p.optionTextColorActiveInverted,v["--n-option-text-color-child-active"]=p.optionTextColorChildActiveInverted,v["--n-prefix-color"]=p.prefixColorInverted,v["--n-suffix-color"]=p.suffixColorInverted,v["--n-group-header-text-color"]=p.groupHeaderTextColorInverted):(v["--n-color"]=p.color,v["--n-option-color-hover"]=p.optionColorHover,v["--n-option-color-active"]=p.optionColorActive,v["--n-option-text-color"]=p.optionTextColor,v["--n-option-text-color-hover"]=p.optionTextColorHover,v["--n-option-text-color-active"]=p.optionTextColorActive,v["--n-option-text-color-child-active"]=p.optionTextColorChildActive,v["--n-prefix-color"]=p.prefixColor,v["--n-suffix-color"]=p.suffixColor,v["--n-group-header-text-color"]=p.groupHeaderTextColor),v}),$=R?be("dropdown",w(()=>`${e.size[0]}${e.inverted?"i":""}`),E,e):void 0;return{mergedClsPrefix:S,mergedTheme:_,tmNodes:t,mergedShow:i,handleAfterLeave:()=>{e.animated&&O()},doUpdateShow:x,cssVars:R?void 0:E,themeClass:$==null?void 0:$.themeClass,onRender:$==null?void 0:$.onRender}},render(){const e=(r,t,o,s,c)=>{var d;const{mergedClsPrefix:y,menuProps:h}=this;(d=this.onRender)===null||d===void 0||d.call(this);const g=(h==null?void 0:h(void 0,this.tmNodes.map(R=>R.rawNode)))||{},S={ref:Je(t),class:[r,`${y}-dropdown`,this.themeClass],clsPrefix:y,tmNodes:this.tmNodes,style:[...o,this.cssVars],showArrow:this.showArrow,arrowStyle:this.arrowStyle,scrollable:this.scrollable,onMouseenter:s,onMouseleave:c};return a(xe,le(this.$attrs,S,g))},{mergedTheme:n}=this,i={show:this.mergedShow,theme:n.peers.Popover,themeOverrides:n.peerOverrides.Popover,internalOnAfterLeave:this.handleAfterLeave,internalRenderBody:e,onUpdateShow:this.doUpdateShow,"onUpdate:show":void 0};return a(Oe,Object.assign({},We(this.$props,so),i),{trigger:()=>{var r,t;return(t=(r=this.$slots).default)===null||t===void 0?void 0:t.call(r)}})}});export{Ze as C,fo as N,Je as c};
