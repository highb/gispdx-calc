# Roadmap

## Done

- [x] **Dark mode** — Toggle in header (sun/moon), persists to localStorage, respects `prefers-color-scheme`
- [x] **URL state / sharing** — `#p=0&t=2&apy=5.0` hash encoding, validated restore on load, `replaceState` sync
- [x] **Multi-child support** — Add/remove children as chips, combined tuition drives all plan calculations
- [x] **Unit tests** — 20 tests covering `effective_cost`, `breakeven_apy`, `build_plans` invariants, edge cases

## Medium Value

- [x] **Chart / visualization** — Line chart showing effective cost vs. APY for all 5 plans with crossover points visible
- [x] **Tax-adjusted APY** — Checkbox for "after-tax APY" with a marginal rate input (interest income is taxable)
- [x] **Print / export** — Print stylesheet; hides interactive controls, forces light theme, shows URL footer

## Lower Value / Polish

- ~~**PWA / offline**~~ — Won't do. Already a static WASM app that loads in <1s; service worker adds complexity for zero real-world value.
- [ ] **Accessibility audit** — ARIA labels on tables, slider, and program/tuition buttons for screen readers
- ~~**Year selector**~~ — Won't do. Without actual future-year tuition data this is meaningless; custom tuition input covers the "what if" scenario better.
- [x] **Custom tuition input** — Let users override the amount for "what if tuition goes up 5%?" scenarios
