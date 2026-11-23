# Official TypeScript 7 and Vue compiler boundaries

Status: Accepted for customer-requested dependency repair
Date: 2026-11-22
Scope: F01/F22, PR #75

## Context

Stock TypeScript 7 is a native compiler without the classic JavaScript compiler API used
by the published Vue checker. Vue's full source/template check therefore fails before
checking any application code. The official TypeScript 6 compatibility distribution
retains that API and is explicitly supported by Vue's exported `run(tscPath)` entry point.

## Alternatives

- Stay on the old compiler and postpone all native TypeScript checking.
- Replace the compiler with a native bridge fork and move the Alpine web build to glibc.
- Run the official native compiler and Vue's required official compatibility compiler explicitly.

## Decision

Install stock TypeScript 7 and the pinned official `@typescript/typescript6` adapter.
`check:native` uses TypeScript 7 for ordinary source modules and Vite configuration.
Its separate config inherits all strict options and excludes only `main.ts`, whose
component imports require Vue's SFC-aware compiler. `check:vue` calls the published
Vue checker with the explicit official TypeScript-6 entry point; it checks the original
complete project, including every SFC, all TypeScript modules and `main.ts`.

Both checks are mandatory in `check` and the production build. No permissive Vue-module
declaration, type-error suppression or fallback changes the application contract. A
regression test checks actual compiler versions and positive/negative script, SFC-script
and template cases. Keep lifecycle scripts disabled and preserve the Alpine AMD64/ARM64
build; the native TypeScript CLI supports that platform without a native API bridge.

## Consequences

There are two explicit build-time compilers until published Vue tooling supports the
official native API. This does not claim native TypeScript-7 checking of SFCs: those remain
fully checked by the official TypeScript-6 engine. Neither compiler ships in app assets.
Retire the compatibility adapter only after the full Vue project, negative type tests,
frozen MCP schema and both image builds pass with the replacement. Dependency and npm audit
gates remain unchanged. Merge the shared security/compatibility repair #72 first.

References: [Vue compatibility tracking](https://github.com/vuejs/language-tools/issues/5381),
[official TypeScript 6 package](https://www.npmjs.com/package/@typescript/typescript6),
[Vue's explicit compiler support](https://github.com/vuejs/language-tools/commit/1ef7ecbd0ab9b475e8e3aed791a1606be8bfa8d1).
