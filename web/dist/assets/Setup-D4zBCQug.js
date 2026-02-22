import{s as n,v as $,x as z,y as T,z as q,A as M,d as L,C as l,D as ce,E as te,F as de,G as se,H as pe,I as ue,J as fe,K as H,L as me,M as Q,O,P as W,Q as he,R as ve,p as N,T as ge,U as be,V as K,W as y,X as G,u as xe,S as J,a as we,c as D,e as g,f as d,w as b,g as o,N as ze,i as B,j as V,t as k,m as ye,B as X,n as Y,r as R,q as Se,b as A,h as Ce}from"./index-DQgClpag.js";import{b as _e}from"./browser-JP79f-a9.js";import{g as ke}from"./get-slot-Bk_rJcZu.js";import{F as Pe}from"./Checkmark-BOYRWfKy.js";import{u as Ne,N as F}from"./use-message-DfHQKw6A.js";import{N as Re}from"./Dropdown-BYzVsjoG.js";import{N as U}from"./FormItem-8Lqb3O-i.js";import{N as Z}from"./Form-CpuxP8S5.js";import{_ as Ie}from"./_plugin-vue_export-helper-DlAUqK2U.js";import"./Popover-BT2YCJ3w.js";const $e=n("steps",`
 width: 100%;
 display: flex;
`,[n("step",`
 position: relative;
 display: flex;
 flex: 1;
 `,[$("disabled","cursor: not-allowed"),$("clickable",`
 cursor: pointer;
 `),z("&:last-child",[n("step-splitor","display: none;")])]),n("step-splitor",`
 background-color: var(--n-splitor-color);
 margin-top: calc(var(--n-step-header-font-size) / 2);
 height: 1px;
 flex: 1;
 align-self: flex-start;
 margin-left: 12px;
 margin-right: 12px;
 transition:
 color .3s var(--n-bezier),
 background-color .3s var(--n-bezier);
 `),n("step-content","flex: 1;",[n("step-content-header",`
 color: var(--n-header-text-color);
 margin-top: calc(var(--n-indicator-size) / 2 - var(--n-step-header-font-size) / 2);
 line-height: var(--n-step-header-font-size);
 font-size: var(--n-step-header-font-size);
 position: relative;
 display: flex;
 font-weight: var(--n-step-header-font-weight);
 margin-left: 9px;
 transition:
 color .3s var(--n-bezier),
 background-color .3s var(--n-bezier);
 `,[T("title",`
 white-space: nowrap;
 flex: 0;
 `)]),T("description",`
 color: var(--n-description-text-color);
 margin-top: 12px;
 margin-left: 9px;
 transition:
 color .3s var(--n-bezier),
 background-color .3s var(--n-bezier);
 `)]),n("step-indicator",`
 background-color: var(--n-indicator-color);
 box-shadow: 0 0 0 1px var(--n-indicator-border-color);
 height: var(--n-indicator-size);
 width: var(--n-indicator-size);
 border-radius: 50%;
 display: flex;
 align-items: center;
 justify-content: center;
 transition:
 background-color .3s var(--n-bezier),
 box-shadow .3s var(--n-bezier);
 `,[n("step-indicator-slot",`
 position: relative;
 width: var(--n-indicator-icon-size);
 height: var(--n-indicator-icon-size);
 font-size: var(--n-indicator-icon-size);
 line-height: var(--n-indicator-icon-size);
 `,[T("index",`
 display: inline-block;
 text-align: center;
 position: absolute;
 left: 0;
 top: 0;
 white-space: nowrap;
 font-size: var(--n-indicator-index-font-size);
 width: var(--n-indicator-icon-size);
 height: var(--n-indicator-icon-size);
 line-height: var(--n-indicator-icon-size);
 color: var(--n-indicator-text-color);
 transition: color .3s var(--n-bezier);
 `,[q()]),n("icon",`
 color: var(--n-indicator-text-color);
 transition: color .3s var(--n-bezier);
 `,[q()]),n("base-icon",`
 color: var(--n-indicator-text-color);
 transition: color .3s var(--n-bezier);
 `,[q()])])]),$("vertical","flex-direction: column;",[M("show-description",[z(">",[n("step","padding-bottom: 8px;")])]),z(">",[n("step","margin-bottom: 16px;",[z("&:last-child","margin-bottom: 0;"),z(">",[n("step-indicator",[z(">",[n("step-splitor",`
 position: absolute;
 bottom: -8px;
 width: 1px;
 margin: 0 !important;
 left: calc(var(--n-indicator-size) / 2);
 height: calc(100% - var(--n-indicator-size));
 `)])]),n("step-content",[T("description","margin-top: 8px;")])])])])]),$("content-bottom",[M("vertical",[z(">",[n("step","flex-direction: column",[z(">",[n("step-line","display: flex;",[z(">",[n("step-splitor",`
 margin-top: 0;
 align-self: center;
 `)])])]),z(">",[n("step-content","margin-top: calc(var(--n-indicator-size) / 2 - var(--n-step-header-font-size) / 2);",[n("step-content-header",`
 margin-left: 0;
 `),n("step-content__description",`
 margin-left: 0;
 `)])])])])])])]);function Te(e,u){return typeof e!="object"||e===null||Array.isArray(e)?null:(e.props||(e.props={}),e.props.internalIndex=u+1,e)}function Be(e){return e.map((u,s)=>Te(u,s))}const Fe=Object.assign(Object.assign({},se.props),{current:Number,status:{type:String,default:"process"},size:{type:String,default:"medium"},vertical:Boolean,contentPlacement:{type:String,default:"right"},"onUpdate:current":[Function,Array],onUpdateCurrent:[Function,Array]}),oe=ue("n-steps"),Ue=L({name:"Steps",props:Fe,slots:Object,setup(e,{slots:u}){const{mergedClsPrefixRef:s,mergedRtlRef:c}=te(e),S=de("Steps",c,s),C=se("Steps","-steps",$e,pe,e,s);return fe(oe,{props:e,mergedThemeRef:C,mergedClsPrefixRef:s,stepsSlots:u}),{mergedClsPrefix:s,rtlEnabled:S}},render(){const{mergedClsPrefix:e}=this;return l("div",{class:[`${e}-steps`,this.rtlEnabled&&`${e}-steps--rtl`,this.vertical&&`${e}-steps--vertical`,this.contentPlacement==="bottom"&&`${e}-steps--content-bottom`]},Be(ce(ke(this))))}}),je={status:String,title:String,description:String,disabled:Boolean,internalIndex:{type:Number,default:0}},ee=L({name:"Step",props:je,slots:Object,setup(e){const u=be(oe,null);u||he("step","`n-step` must be placed inside `n-steps`.");const{inlineThemeDisabled:s}=te(),{props:c,mergedThemeRef:S,mergedClsPrefixRef:C,stepsSlots:P}=u,x=K(c,"vertical"),h=K(c,"contentPlacement"),v=N(()=>{const{status:a}=e;if(a)return a;{const{internalIndex:t}=e,{current:_}=c;if(_===void 0)return"process";if(t<_)return"finish";if(t===_)return c.status||"process";if(t>_)return"wait"}return"process"}),w=N(()=>{const{value:a}=v,{size:t}=c,{common:{cubicBezierEaseInOut:_},self:{stepHeaderFontWeight:j,[y("stepHeaderFontSize",t)]:E,[y("indicatorIndexFontSize",t)]:i,[y("indicatorSize",t)]:r,[y("indicatorIconSize",t)]:p,[y("indicatorTextColor",a)]:I,[y("indicatorBorderColor",a)]:re,[y("headerTextColor",a)]:ne,[y("splitorColor",a)]:ae,[y("indicatorColor",a)]:ie,[y("descriptionTextColor",a)]:le}}=S.value;return{"--n-bezier":_,"--n-description-text-color":le,"--n-header-text-color":ne,"--n-indicator-border-color":re,"--n-indicator-color":ie,"--n-indicator-icon-size":p,"--n-indicator-index-font-size":i,"--n-indicator-size":r,"--n-indicator-text-color":I,"--n-splitor-color":ae,"--n-step-header-font-size":E,"--n-step-header-font-weight":j}}),f=s?ve("step",N(()=>{const{value:a}=v,{size:t}=c;return`${a[0]}${t[0]}`}),w,c):void 0,m=N(()=>{if(e.disabled)return;const{onUpdateCurrent:a,"onUpdate:current":t}=c;return a||t?()=>{a&&G(a,e.internalIndex),t&&G(t,e.internalIndex)}:void 0});return{stepsSlots:P,mergedClsPrefix:C,vertical:x,mergedStatus:v,handleStepClick:m,cssVars:s?void 0:w,themeClass:f==null?void 0:f.themeClass,onRender:f==null?void 0:f.onRender,contentPlacement:h}},render(){const{mergedClsPrefix:e,onRender:u,handleStepClick:s,disabled:c,contentPlacement:S,vertical:C}=this,P=H(this.$slots.default,f=>{const m=f||this.description;return m?l("div",{class:`${e}-step-content__description`},m):null}),x=l("div",{class:`${e}-step-splitor`}),h=l("div",{class:`${e}-step-indicator`,key:S},l("div",{class:`${e}-step-indicator-slot`},l(me,null,{default:()=>H(this.$slots.icon,f=>{const{mergedStatus:m,stepsSlots:a}=this;return m==="finish"||m==="error"?m==="finish"?l(Q,{clsPrefix:e,key:"finish"},{default:()=>O(a["finish-icon"],()=>[l(Pe,null)])}):m==="error"?l(Q,{clsPrefix:e,key:"error"},{default:()=>O(a["error-icon"],()=>[l(ge,null)])}):null:f||l("div",{key:this.internalIndex,class:`${e}-step-indicator-slot__index`},this.internalIndex)})})),C?x:null),v=l("div",{class:`${e}-step-content`},l("div",{class:`${e}-step-content-header`},l("div",{class:`${e}-step-content-header__title`},O(this.$slots.title,()=>[this.title])),!C&&S==="right"?x:null),P);let w;return!C&&S==="bottom"?w=l(W,null,l("div",{class:`${e}-step-line`},h,x),v):w=l(W,null,h,v),u==null||u(),l("div",{class:[`${e}-step`,c&&`${e}-step--disabled`,!c&&s&&`${e}-step--clickable`,this.themeClass,P&&`${e}-step--show-description`,`${e}-step--${this.mergedStatus}-status`],style:this.cssVars,onClick:s},w)}}),Ee={class:"setup-page"},qe={class:"setup-lang"},Oe={class:"setup-lang-btn"},De={key:1,class:"totp-step"},Ve={class:"scan-hint"},Ae={key:0,class:"qr-code"},Le=["src"],Me={class:"otpauth"},He=L({__name:"Setup",setup(e){const u=Ce(),{t:s,locale:c}=xe(),S=J.map(i=>({label:i.label,key:i.code})),C=N(()=>{var i;return((i=J.find(r=>r.code===c.value))==null?void 0:i.label)??c.value});function P(i){c.value=i,localStorage.setItem("proxy_locale",i)}const x=Ne();we();const h=R(!1),v=R(1),w=R(""),f=R(""),m=R("");async function a(i){try{m.value=await _e.toDataURL(i,{width:200,margin:2,color:{dark:"#000000",light:"#ffffff"}})}catch(r){console.error("QR code generation failed:",r)}}const t=Se({username:"",password:"",confirmPassword:"",totp:""}),_=N(()=>({username:{required:!0,message:s("validation.required"),trigger:"blur"},password:{required:!0,message:s("validation.required"),trigger:"blur"},confirmPassword:[{required:!0,message:s("validation.required"),trigger:"blur"},{validator:()=>t.password===t.confirmPassword,message:s("account.passwordMismatch"),trigger:"blur"}]}));async function j(){var i,r;if(t.password!==t.confirmPassword){x.error(s("account.passwordMismatch"));return}h.value=!0;try{const p=await A.setup(t.username,t.password),I=await A.totpSetup(p.setup_token);w.value=I.secret,f.value=I.qr_data_url||`otpauth://totp/admin?secret=${w.value}&issuer=MinecraftProxy`,await a(f.value),v.value=2}catch(p){x.error(((r=(i=p.response)==null?void 0:i.data)==null?void 0:r.message)||"Setup failed")}finally{h.value=!1}}async function E(){var i,r;h.value=!0;try{await A.totpConfirm(t.username,w.value,t.totp),x.success(s("auth.bindSuccess")),u.push("/login")}catch(p){x.error(((r=(i=p.response)==null?void 0:i.data)==null?void 0:r.message)||"Bind failed")}finally{h.value=!1}}return(i,r)=>(B(),D("div",Ee,[g("div",qe,[d(o(Re),{options:o(S),onSelect:P,placement:"bottom-end","show-arrow":!1},{default:b(()=>[g("button",Oe,[r[4]||(r[4]=g("svg",{xmlns:"http://www.w3.org/2000/svg",width:"14",height:"14",viewBox:"0 0 24 24",fill:"none",stroke:"currentColor","stroke-width":"2","stroke-linecap":"round","stroke-linejoin":"round"},[g("circle",{cx:"12",cy:"12",r:"10"}),g("line",{x1:"2",y1:"12",x2:"22",y2:"12"}),g("path",{d:"M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"})],-1)),V(" "+k(C.value),1)])]),_:1},8,["options"])]),d(o(ze),{class:"setup-card"},{default:b(()=>[g("h1",null,k(o(s)("auth.setupTitle")),1),d(o(Ue),{current:v.value},{default:b(()=>[d(o(ee),{title:o(s)("auth.setupStep1")},null,8,["title"]),d(o(ee),{title:o(s)("auth.setupStep2")},null,8,["title"])]),_:1},8,["current"]),v.value===1?(B(),ye(o(Z),{key:0,ref:"formRef",model:t,rules:_.value},{default:b(()=>[d(o(U),{path:"username",label:o(s)("auth.username")},{default:b(()=>[d(o(F),{value:t.username,"onUpdate:value":r[0]||(r[0]=p=>t.username=p)},null,8,["value"])]),_:1},8,["label"]),d(o(U),{path:"password",label:o(s)("auth.password")},{default:b(()=>[d(o(F),{value:t.password,"onUpdate:value":r[1]||(r[1]=p=>t.password=p),type:"password","show-password-on":"click"},null,8,["value"])]),_:1},8,["label"]),d(o(U),{path:"confirmPassword",label:o(s)("account.confirmPassword")},{default:b(()=>[d(o(F),{value:t.confirmPassword,"onUpdate:value":r[2]||(r[2]=p=>t.confirmPassword=p),type:"password","show-password-on":"click"},null,8,["value"])]),_:1},8,["label"]),d(o(X),{type:"primary",block:"",loading:h.value,onClick:j},{default:b(()=>[V(k(o(s)("common.confirm")),1)]),_:1},8,["loading"])]),_:1},8,["model","rules"])):v.value===2?(B(),D("div",De,[g("p",null,k(o(s)("auth.bindDesc")),1),g("p",Ve,k(o(s)("auth.scanQR")),1),m.value?(B(),D("div",Ae,[g("img",{src:m.value,alt:"TOTP QR Code"},null,8,Le)])):Y("",!0),g("div",Me,k(f.value),1),d(o(Z),null,{default:b(()=>[d(o(U),{label:o(s)("auth.totpTitle")},{default:b(()=>[d(o(F),{value:t.totp,"onUpdate:value":r[3]||(r[3]=p=>t.totp=p),placeholder:o(s)("auth.totpPlaceholder")},null,8,["value","placeholder"])]),_:1},8,["label"]),d(o(X),{type:"primary",block:"",loading:h.value,onClick:E},{default:b(()=>[V(k(o(s)("auth.totpVerify")),1)]),_:1},8,["loading"])]),_:1})])):Y("",!0)]),_:1})]))}}),st=Ie(He,[["__scopeId","data-v-cf67baa3"]]);export{st as default};
