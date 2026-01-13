import{a as h,b as u}from"./svelte-vendor.js";import"./environment.js";let f={};function x(n){}function k(n){f=n}let _=null;function w(n){_=n}function C(n){}function g(n,e){n.component(t=>{let{stores:i,page:a,constructors:s,components:p=[],form:o,data_0:d=null,data_1:c=null}=e;h("__svelte__",i),i.page.set(a);const m=s[1];if(s[1]){t.push("<!--[-->");const l=s[0];t.push("<!---->"),l(t,{data:d,form:o,params:a.params,children:r=>{r.push("<!---->"),m(r,{data:c,form:o,params:a.params}),r.push("<!---->")},$$slots:{default:!0}}),t.push("<!---->")}else{t.push("<!--[!-->");const l=s[0];t.push("<!---->"),l(t,{data:d,form:o,params:a.params}),t.push("<!---->")}t.push("<!--]--> "),t.push("<!--[!-->"),t.push("<!--]-->")})}const v=u(g),E={app_template_contains_nonce:!1,async:!1,csp:{mode:"auto",directives:{"script-src":["self"],"upgrade-insecure-requests":!1,"block-all-mixed-content":!1},reportOnly:{"upgrade-insecure-requests":!1,"block-all-mixed-content":!1}},csrf_check_origin:!0,csrf_trusted_origins:[],embedded:!1,env_public_prefix:"PUBLIC_",env_private_prefix:"",hash_routing:!1,hooks:null,preload_strategy:"modulepreload",root:v,service_worker:!1,service_worker_options:void 0,templates:{app:({head:n,body:e,assets:t,nonce:i,env:a})=>`<!doctype html>
<html lang="en">
	<head>
		<meta charset="utf-8" />
		<meta name="viewport" content="width=device-width, initial-scale=1" />
		`+n+`
		<title>Scriptoria</title>
	</head>
	<body data-sveltekit-preload-data="hover">
		<div style="display: contents">`+e+`</div>
	</body>
</html>
`,error:({status:n,message:e})=>`<!doctype html>
<html lang="en">
	<head>
		<meta charset="utf-8" />
		<title>`+e+`</title>

		<style>
			body {
				--bg: white;
				--fg: #222;
				--divider: #ccc;
				background: var(--bg);
				color: var(--fg);
				font-family:
					system-ui,
					-apple-system,
					BlinkMacSystemFont,
					'Segoe UI',
					Roboto,
					Oxygen,
					Ubuntu,
					Cantarell,
					'Open Sans',
					'Helvetica Neue',
					sans-serif;
				display: flex;
				align-items: center;
				justify-content: center;
				height: 100vh;
				margin: 0;
			}

			.error {
				display: flex;
				align-items: center;
				max-width: 32rem;
				margin: 0 1rem;
			}

			.status {
				font-weight: 200;
				font-size: 3rem;
				line-height: 1;
				position: relative;
				top: -0.05rem;
			}

			.message {
				border-left: 1px solid var(--divider);
				padding: 0 0 0 1rem;
				margin: 0 0 0 1rem;
				min-height: 2.5rem;
				display: flex;
				align-items: center;
			}

			.message h1 {
				font-weight: 400;
				font-size: 1em;
				margin: 0;
			}

			@media (prefers-color-scheme: dark) {
				body {
					--bg: #222;
					--fg: #ddd;
					--divider: #666;
				}
			}
		</style>
	</head>
	<body>
		<div class="error">
			<span class="status">`+n+`</span>
			<div class="message">
				<h1>`+e+`</h1>
			</div>
		</div>
	</body>
</html>
`},version_hash:"1sdjn3t"};async function P(){return{handle:void 0,handleFetch:void 0,handleError:void 0,handleValidationError:void 0,init:void 0,reroute:void 0,transport:void 0}}export{k as a,w as b,C as c,P as g,E as o,f as p,_ as r,x as s};
