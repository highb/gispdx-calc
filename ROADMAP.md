# Roadmap

## Done

- [x] **Dark mode** — Toggle in header (sun/moon), persists to localStorage, respects `prefers-color-scheme`
- [x] **URL state / sharing** — `#p=0&t=2&apy=5.0` hash encoding, validated restore on load, `replaceState` sync
- [x] **Multi-child support** — Add/remove children as chips, combined tuition drives all plan calculations
- [x] **Unit tests** — 20 tests covering `effective_cost`, `breakeven_apy`, `build_plans` invariants, edge cases

## Medium Value

- [ ] **Chart / visualization** — Line chart showing effective cost vs. APY for all 5 plans with crossover points visible
- [ ] **Tax-adjusted APY** — Checkbox for "after-tax APY" with a marginal rate input (interest income is taxable)
- [ ] **Print / export** — Print stylesheet or "save as PDF" so parents can bring the analysis to a discussion

## Lower Value / Polish

- [ ] **PWA / offline** — Service worker + manifest for offline use (already a static WASM app)
- [ ] **Accessibility audit** — ARIA labels on tables, slider, and program/tuition buttons for screen readers
- [ ] **Year selector** — Dropdown for future school years (with caveat that tuition may change)
- [ ] **Custom tuition input** — Let users override the amount for "what if tuition goes up 5%?" scenarios
