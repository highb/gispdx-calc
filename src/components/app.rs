use leptos::prelude::*;
use wasm_bindgen::JsCast;

use crate::data::{self, PROGRAMS, PaymentPlan};
use crate::math;
use crate::components::math_mode::MathMode;
use crate::components::chart::CostChart;

/// Parsed URL hash state
struct HashState {
    program: Option<usize>,
    tuition: Option<usize>,
    apy: Option<f64>,
    tax_enabled: bool,
    tax_rate: Option<f64>,
}

/// Parse the URL hash (e.g. `#p=0&t=2&apy=5.0&tax=1&rate=24`).
fn parse_hash() -> HashState {
    let hash = web_sys::window()
        .and_then(|w| w.location().hash().ok())
        .unwrap_or_default();
    let hash = hash.trim_start_matches('#');
    let mut p = None;
    let mut t = None;
    let mut apy = None;
    let mut tax = false;
    let mut rate = None;
    for part in hash.split('&') {
        if let Some((key, val)) = part.split_once('=') {
            match key {
                "p" => p = val.parse::<usize>().ok(),
                "t" => t = val.parse::<usize>().ok(),
                "apy" => apy = val.parse::<f64>().ok(),
                "tax" => tax = val == "1",
                "rate" => rate = val.parse::<f64>().ok(),
                _ => {}
            }
        }
    }
    HashState { program: p, tuition: t, apy, tax_enabled: tax, tax_rate: rate }
}

/// Write current state to the URL hash via `replaceState` (no history entry).
fn write_hash(program: Option<usize>, tuition: Option<usize>, apy: f64, tax_on: bool, tax_rate: f64) {
    let Some(window) = web_sys::window() else { return };
    let mut parts = Vec::new();
    if let Some(p) = program {
        parts.push(format!("p={}", p));
    }
    if let Some(t) = tuition {
        parts.push(format!("t={}", t));
    }
    parts.push(format!("apy={}", apy));
    if tax_on {
        parts.push("tax=1".into());
        parts.push(format!("rate={}", tax_rate));
    }
    let hash = format!("#{}", parts.join("&"));
    let _ = window.history().and_then(|h| {
        h.replace_state_with_url(
            &wasm_bindgen::JsValue::NULL,
            "",
            Some(&hash),
        )
    });
}

fn usd(v: f64) -> String {
    let s = format!("{:.2}", v.abs());
    let (int_part, dec_part) = s.split_once('.').unwrap();
    let int_bytes: Vec<u8> = int_part.bytes().collect();
    let mut formatted = String::new();
    for (i, &b) in int_bytes.iter().enumerate() {
        if i > 0 && (int_bytes.len() - i) % 3 == 0 {
            formatted.push(',');
        }
        formatted.push(b as char);
    }
    if v < 0.0 {
        format!("-${}.{}", formatted, dec_part)
    } else {
        format!("${}.{}", formatted, dec_part)
    }
}

fn pct(v: f64) -> String {
    format!("{:.2}%", v * 100.0)
}

#[component]
pub fn App() -> impl IntoView {
    let (selected_program, set_selected_program) = signal::<Option<usize>>(None);
    let (selected_tuition, set_selected_tuition) = signal::<Option<usize>>(None);
    let (children, set_children) = signal::<Vec<(usize, usize)>>(Vec::new());
    let (apy_value, set_apy_value) = signal(5.0_f64);

    // Feature 2: Tax-adjusted APY
    let (tax_enabled, set_tax_enabled) = signal(false);
    let (tax_rate, set_tax_rate) = signal(24.0_f64); // marginal rate %

    // Derived: the APY actually used in all calculations
    let effective_apy = move || {
        let raw = apy_value.get();
        if tax_enabled.get() {
            raw * (1.0 - tax_rate.get() / 100.0)
        } else {
            raw
        }
    };

    // Feature 3: Custom tuition override
    let (tuition_override, set_tuition_override) = signal::<Option<f64>>(None);

    // Derived: current program
    let current_program = move || selected_program.get().map(|i| &PROGRAMS[i]);

    // Whether selector has a valid program+tuition pair picked
    let can_add_child = move || {
        selected_program.get().is_some() && selected_tuition.get().is_some()
    };

    // Add current selection as a child
    let add_child = move |_| {
        if let (Some(pi), Some(ti)) = (selected_program.get(), selected_tuition.get()) {
            set_children.update(|v| v.push((pi, ti)));
        }
    };

    // Remove child at index
    let remove_child = move |idx: usize| {
        set_children.update(|v| { v.remove(idx); });
    };

    // Derived: base tuition (from children list or single selection, before override)
    let base_tuition = move || -> Option<f64> {
        let kids = children.get();
        if !kids.is_empty() {
            Some(kids.iter().map(|&(pi, ti)| {
                PROGRAMS[pi].options[ti].amount
            }).sum())
        } else {
            let prog = current_program()?;
            let ti = selected_tuition.get()?;
            let opt = prog.options.get(ti)?;
            Some(opt.amount)
        }
    };

    // Derived: total tuition (with optional custom override)
    let total_tuition = move || -> Option<f64> {
        if let Some(ov) = tuition_override.get() {
            Some(ov)
        } else {
            base_tuition()
        }
    };

    // Derived: current plans from total tuition
    let current_plans = move || -> Option<Vec<PaymentPlan>> {
        total_tuition().map(data::build_plans)
    };

    // Handler for program select
    let on_program_select = move |idx: usize| {
        set_selected_program.set(Some(idx));
        set_tuition_override.set(None); // reset custom override
        let prog = &PROGRAMS[idx];
        if prog.options.len() == 1 {
            set_selected_tuition.set(Some(0));
        } else {
            set_selected_tuition.set(None);
        }
    };

    // On mount: restore state from URL hash, falling back to defaults
    Effect::new(move || {
        let hs = parse_hash();

        let prog_idx = hs.program.filter(|&i| i < PROGRAMS.len()).unwrap_or(0);
        set_selected_program.set(Some(prog_idx));

        let max_t = PROGRAMS[prog_idx].options.len();
        let tui_idx = hs.tuition.filter(|&i| i < max_t).unwrap_or(2.min(max_t.saturating_sub(1)));
        set_selected_tuition.set(Some(tui_idx));

        if let Some(a) = hs.apy {
            set_apy_value.set(a);
        }

        // Restore tax state
        set_tax_enabled.set(hs.tax_enabled);
        if let Some(r) = hs.tax_rate {
            set_tax_rate.set(r);
        }
    });

    // Sync signals → URL hash (runs whenever any signal changes)
    Effect::new(move || {
        let p = selected_program.get();
        let t = selected_tuition.get();
        let a = apy_value.get();
        let tax_on = tax_enabled.get();
        let rate = tax_rate.get();
        write_hash(p, t, a, tax_on, rate);
    });

    // Dark mode
    let (dark, set_dark) = signal(false);

    let apply_theme = move |is_dark: bool| {
        if let Some(w) = web_sys::window() {
            if let Some(doc) = w.document() {
                if let Some(el) = doc.document_element() {
                    let html = el.unchecked_into::<web_sys::HtmlElement>();
                    let _ = html.dataset().set("theme", if is_dark { "dark" } else { "light" });
                }
            }
            if let Ok(Some(storage)) = w.local_storage() {
                let _ = storage.set_item("theme", if is_dark { "dark" } else { "light" });
            }
        }
    };

    // On mount: read localStorage or fall back to system preference
    Effect::new(move || {
        if let Some(w) = web_sys::window() {
            let prefer_dark = w
                .local_storage()
                .ok()
                .flatten()
                .and_then(|s| s.get_item("theme").ok().flatten())
                .map(|v| v == "dark")
                .unwrap_or_else(|| {
                    w.match_media("(prefers-color-scheme: dark)")
                        .ok()
                        .flatten()
                        .map(|mql| mql.matches())
                        .unwrap_or(false)
                });
            set_dark.set(prefer_dark);
            apply_theme(prefer_dark);
        }
    });

    let toggle_dark = move |_| {
        let next = !dark.get();
        set_dark.set(next);
        apply_theme(next);
    };

    view! {
        <header>
            <h1>"\u{1F1FA}\u{1F1F8}\u{1F1E9}\u{1F1EA} GIS Payment vs. APY Calculator"</h1>
            <p>"Payment Strategy vs. Investment APY Calculator \u{00B7} 2026\u{2013}2027"</p>
            <button class="theme-toggle" on:click=toggle_dark title="Toggle dark mode">
                {move || if dark.get() { "\u{2600}\u{FE0F}" } else { "\u{1F319}" }}
            </button>
        </header>

        <div class="timeline">
            <span>"May 1 \u{2019}26 \u{2014} Early pay (2% off)"</span>
            <span>"Jul 1 \u{2019}26 \u{2014} Full / 1st installment"</span>
            <span>"Oct 1 \u{2019}26 \u{2014} Q2"</span>
            <span>"Jan 1 \u{2019}27 \u{2014} Semi 2 / Q3"</span>
            <span>"Apr 1 \u{2019}27 \u{2014} Q4"</span>
            <span>"Jun 1 \u{2019}27 \u{2014} Year end"</span>
        </div>

        <p style="text-align:center;font-size:.85rem;color:var(--muted);margin-bottom:1rem;">
            "$1,000 non-refundable deposit required for all plans (excluded from analysis). Choose a program, then a schedule."
        </p>

        // Program selector
        <div class="card">
            <h2>"Program"</h2>
            <div class="program-select">
                {PROGRAMS.iter().enumerate().map(|(i, prog)| {
                    let active = move || selected_program.get() == Some(i);
                    view! {
                        <button
                            class:program-btn=true
                            class:active=active
                            on:click=move |_| on_program_select(i)
                        >
                            <div class="label">{prog.label}</div>
                            <div class="desc">{prog.desc}</div>
                        </button>
                    }
                }).collect::<Vec<_>>()}
            </div>
        </div>

        // Tuition selector
        <Show when=move || current_program().is_some()>
            {move || {
                let prog = current_program().unwrap();
                view! {
                    <div class="card">
                        <h2>{format!("{} \u{2014} Schedule", prog.label)}</h2>
                        <div class="tuition-select">
                            {prog.options.iter().enumerate().map(|(i, opt)| {
                                let active = move || selected_tuition.get() == Some(i);
                                view! {
                                    <button
                                        class:tuition-btn=true
                                        class:active=active
                                        on:click=move |_| {
                                            set_selected_tuition.set(Some(i));
                                            set_tuition_override.set(None);
                                        }
                                    >
                                        <div class="label">{opt.label}</div>
                                        <div class="amount">{usd(opt.amount)}</div>
                                    </button>
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                    </div>
                }
            }}
        </Show>

        // Feature 3: Custom tuition override
        <Show when=move || base_tuition().is_some()>
            {move || {
                let _display_val = total_tuition().unwrap_or(0.0);
                let is_overridden = tuition_override.get().is_some();
                view! {
                    <div class="card tuition-override-card">
                        <div class="tuition-override-row">
                            <label for="tuitionOverride" class="tuition-override-label">
                                "Total Tuition:"
                            </label>
                            <span class="tuition-override-dollar">"$"</span>
                            <input
                                type="number"
                                id="tuitionOverride"
                                class="tuition-override-input"
                                class:overridden=is_overridden
                                prop:value=move || format!("{:.2}", total_tuition().unwrap_or(0.0))
                                min="0" step="100"
                                on:input=move |ev| {
                                    if let Ok(v) = event_target_value(&ev).parse::<f64>() {
                                        if v > 0.0 {
                                            // Check if it matches the base; if so, clear override
                                            if let Some(base) = base_tuition() {
                                                if (v - base).abs() < 0.01 {
                                                    set_tuition_override.set(None);
                                                    return;
                                                }
                                            }
                                            set_tuition_override.set(Some(v));
                                        }
                                    }
                                }
                            />
                            <Show when=move || tuition_override.get().is_some()>
                                <button
                                    class="tuition-reset-btn"
                                    title="Reset to program tuition"
                                    on:click=move |_| set_tuition_override.set(None)
                                >
                                    "\u{21BA} Reset"
                                </button>
                            </Show>
                        </div>
                        <Show when=move || tuition_override.get().is_some()>
                            <div class="tuition-override-note">
                                {move || format!("Custom override (program default: {})", usd(base_tuition().unwrap_or(0.0)))}
                            </div>
                        </Show>
                    </div>
                }
            }}
        </Show>

        // Add Child button
        <Show when=move || can_add_child()>
            <div style="text-align:center;margin-bottom:1rem">
                <button class="add-child-btn" on:click=add_child>
                    "\u{2795} Add Child"
                </button>
            </div>
        </Show>

        // Children list
        <Show when=move || !children.get().is_empty()>
            {move || {
                let kids = children.get();
                let total: f64 = kids.iter().map(|&(pi, ti)| PROGRAMS[pi].options[ti].amount).sum();
                view! {
                    <div class="card children-card">
                        <h2>"\u{1F9D2} Children"</h2>
                        <div class="children-list">
                            {kids.iter().enumerate().map(|(idx, &(pi, ti))| {
                                let prog = &PROGRAMS[pi];
                                let opt = &prog.options[ti];
                                let label = format!("Child {}: {} \u{2014} {} ({})", idx + 1, prog.label, opt.label, usd(opt.amount));
                                view! {
                                    <div class="child-chip">
                                        <span class="child-label">{label}</span>
                                        <button class="child-remove" on:click=move |_| remove_child(idx)>
                                            "\u{00D7}"
                                        </button>
                                    </div>
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                        <div class="children-total">
                            {format!("Combined tuition: {}", usd(total))}
                        </div>
                    </div>
                }
            }}
        </Show>

        // Results
        <Show when=move || current_plans().is_some()>
            {move || {
                let plans = current_plans().unwrap();
                view! {
                    <SummaryTable plans=plans.clone() />
                    <ApySection
                        plans=plans.clone()
                        apy_value=apy_value
                        set_apy_value=set_apy_value
                        effective_apy=Signal::derive(effective_apy)
                        tax_enabled=tax_enabled
                        set_tax_enabled=set_tax_enabled
                        tax_rate=tax_rate
                        set_tax_rate=set_tax_rate
                    />
                    <CostChart plans=plans.clone() apy_value=Signal::derive(effective_apy) dark=dark />
                    <BreakevenTable plans=plans.clone() />
                    <PairwiseGrid plans=plans />
                }
            }}
        </Show>

        <MathMode />

        <footer>
            "Based on published GISB 2026\u{2013}2027 tuition & fee schedule."
            <br />
            "This is a mathematical tool, not financial advice. Not affiliated with or endorsed by German International School Portland."
        </footer>

        // Feature 4: Print-only URL footer (visible only when printing)
        <div class="print-url">
            {move || {
                let url = web_sys::window()
                    .and_then(|w| w.location().href().ok())
                    .unwrap_or_default();
                view! {
                    "Generated by GIS Payment Calculator \u{2014} "
                    <span>{url}</span>
                }
            }}
        </div>
    }
}

#[component]
fn SummaryTable(plans: Vec<PaymentPlan>) -> impl IntoView {
    view! {
        <div class="card">
            <h2>"Payment Plans"</h2>
            <div class="table-wrap">
                <table>
                    <thead>
                        <tr>
                            <th>"Plan"</th>
                            <th class="num">"Total Paid"</th>
                            <th class="num">"Finance Fees"</th>
                            <th class="num">"Avg Month"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {plans.iter().map(|plan| {
                            let tot = math::total_dollars(&plan.payments);
                            let avg = math::weighted_avg_month(&plan.payments);
                            let fee_str = if plan.discount > 0.0 {
                                format!("\u{2212}{} discount", usd(plan.discount))
                            } else if plan.fees > 0.0 {
                                usd(plan.fees)
                            } else {
                                "\u{2014}".into()
                            };
                            view! {
                                <tr>
                                    <td>{plan.name.clone()}</td>
                                    <td class="num">{usd(tot)}</td>
                                    <td class="num">{fee_str}</td>
                                    <td class="num">{format!("{:.1}", avg)}</td>
                                </tr>
                            }
                        }).collect::<Vec<_>>()}
                    </tbody>
                </table>
            </div>
        </div>
    }
}

#[component]
fn ApySection(
    plans: Vec<PaymentPlan>,
    apy_value: ReadSignal<f64>,
    set_apy_value: WriteSignal<f64>,
    effective_apy: Signal<f64>,
    tax_enabled: ReadSignal<bool>,
    set_tax_enabled: WriteSignal<bool>,
    tax_rate: ReadSignal<f64>,
    set_tax_rate: WriteSignal<f64>,
) -> impl IntoView {
    let plans_clone = plans.clone();
    let ranked = Memo::new(move |_| {
        let apy = effective_apy.get() / 100.0;
        let mut items: Vec<(String, f64, Vec<(f64, f64)>)> = plans_clone
            .iter()
            .map(|p| {
                let cost = math::effective_cost(&p.payments, apy);
                (p.name.clone(), cost, p.payments.clone())
            })
            .collect();
        items.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        items
    });

    view! {
        <div class="card">
            <h2>"Effective Cost at Your APY"</h2>
            <div class="apy-row">
                <label for="apyInput">"Your APY:"</label>
                <input
                    type="number"
                    id="apyInput"
                    class="apy-input"
                    prop:value=move || format!("{:.1}", apy_value.get())
                    min="0" max="100" step="0.1"
                    on:input=move |ev| {
                        if let Ok(v) = event_target_value(&ev).parse::<f64>() {
                            set_apy_value.set(v);
                        }
                    }
                />
                <span>"%"</span>
                <input
                    type="range"
                    id="apySlider"
                    class="apy-slider"
                    min="0" max="30" step="0.1"
                    prop:value=move || format!("{:.1}", apy_value.get())
                    on:input=move |ev| {
                        if let Ok(v) = event_target_value(&ev).parse::<f64>() {
                            set_apy_value.set(v);
                        }
                    }
                />
            </div>
            // Feature 2: Tax-adjusted APY
            <div class="tax-row">
                <label class="tax-check-label">
                    <input
                        type="checkbox"
                        class="tax-checkbox"
                        prop:checked=move || tax_enabled.get()
                        on:change=move |ev| {
                            let checked = event_target_checked(&ev);
                            set_tax_enabled.set(checked);
                        }
                    />
                    " After-tax APY"
                </label>
                <Show when=move || tax_enabled.get()>
                    <span class="tax-rate-group">
                        <label for="taxRate">"Marginal rate:"</label>
                        <input
                            type="number"
                            id="taxRate"
                            class="tax-rate-input"
                            prop:value=move || format!("{:.0}", tax_rate.get())
                            min="0" max="60" step="1"
                            on:input=move |ev| {
                                if let Ok(v) = event_target_value(&ev).parse::<f64>() {
                                    set_tax_rate.set(v);
                                }
                            }
                        />
                        <span>"%"</span>
                    </span>
                    <span class="tax-effective">
                        {move || format!("\u{2192} Effective APY: {:.2}%", effective_apy.get())}
                    </span>
                </Show>
            </div>
            <div class="table-wrap">
                <table style="margin-top:.75rem">
                    <thead>
                        <tr>
                            <th>"Rank"</th>
                            <th>"Plan"</th>
                            <th class="num">"Eff. Cost (Jun \u{2019}27)"</th>
                            <th class="num">"vs. Best"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {move || {
                            let items = ranked.get();
                            let min_cost = items[0].1;
                            items.into_iter().enumerate().map(|(i, (name, cost, _))| {
                                let extra = cost - min_cost;
                                let is_best = extra < 0.005;
                                let tag = if is_best {
                                    view! { <span class="tag tag-best">"\u{25C0} BEST"</span> }.into_any()
                                } else {
                                    view! { <span class="tag tag-extra">{format!("+{}", usd(extra))}</span> }.into_any()
                                };
                                view! {
                                    <tr class:best-row=is_best>
                                        <td class="num">{i + 1}</td>
                                        <td>{name}</td>
                                        <td class="num">{usd(cost)}</td>
                                        <td class="num">{tag}</td>
                                    </tr>
                                }
                            }).collect::<Vec<_>>()
                        }}
                    </tbody>
                </table>
            </div>
            // Recommendation
            {move || {
                let items = ranked.get();
                let best_name = &items[0].0;
                let best_payments = &items[0].2;
                let avg_m = math::weighted_avg_month(best_payments);
                let eff = effective_apy.get();
                let msg = if avg_m == 0.0 {
                    format!("{} \u{2014} The 2% early-payment discount saves the most at {:.1}% APY.", best_name, eff)
                } else if avg_m == 2.0 && best_payments.len() == 1 {
                    format!("{} \u{2014} Paying in full in July (no fee) is optimal. Invest until then.", best_name)
                } else {
                    format!("{} \u{2014} Spreading payments (avg month {:.1}) keeps money invested long enough to outweigh finance fees at {:.1}% APY.", best_name, avg_m, eff)
                };
                view! {
                    <div class="reco">
                        <strong>{msg}</strong>
                    </div>
                }
            }}
        </div>
    }
}

#[component]
fn BreakevenTable(plans: Vec<PaymentPlan>) -> impl IntoView {
    // Baseline = plans[1] (Pay in full July 1)
    let baseline = &plans[1];
    let rows: Vec<_> = plans
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != 1)
        .map(|(_, plan)| {
            let be = math::breakeven_apy(&baseline.payments, &plan.payments);
            let (be_str, meaning) = match be {
                None => ("\u{2014}".into(), "No crossover (one always cheaper)".into()),
                Some(v) if v <= 0.0 => ("\u{2264} 0%".into(), "Always cheaper than baseline".into()),
                Some(v) => {
                    let avg_late = math::weighted_avg_month(&plan.payments);
                    let avg_base = math::weighted_avg_month(&baseline.payments);
                    let dir = if avg_late > avg_base { "later" } else { "earlier" };
                    (pct(v), format!("Above this APY, the {} plan wins", dir))
                }
            };
            (plan.name.clone(), be_str, meaning)
        })
        .collect();

    view! {
        <div class="card">
            <h2>"Breakeven APYs vs. Pay in Full July 1"</h2>
            <div class="table-wrap">
                <table>
                    <thead>
                        <tr>
                            <th>"Plan"</th>
                            <th class="num">"Breakeven APY"</th>
                            <th>"Meaning"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {rows.into_iter().map(|(name, be_str, meaning)| {
                            view! {
                                <tr>
                                    <td>{name}</td>
                                    <td class="num">{be_str}</td>
                                    <td>{meaning}</td>
                                </tr>
                            }
                        }).collect::<Vec<_>>()}
                    </tbody>
                </table>
            </div>
        </div>
    }
}

#[component]
fn PairwiseGrid(plans: Vec<PaymentPlan>) -> impl IntoView {
    let n = plans.len();
    let mut pairs = Vec::new();
    for i in 0..n {
        for j in (i + 1)..n {
            let (ai, bi) = if math::weighted_avg_month(&plans[i].payments)
                > math::weighted_avg_month(&plans[j].payments)
            {
                (j, i)
            } else {
                (i, j)
            };
            let be = math::breakeven_apy(&plans[ai].payments, &plans[bi].payments);
            let result = match be {
                None => "No crossover found".into(),
                Some(v) if v <= 0.0 => {
                    "Later plan always cheaper \u{2014} no minimum APY required".into()
                }
                Some(v) => format!("Breakeven APY: {}", pct(v)),
            };
            pairs.push((plans[ai].name.clone(), plans[bi].name.clone(), result));
        }
    }

    view! {
        <div class="card">
            <h2>"All Pairwise Breakeven APYs"</h2>
            <div class="pair-grid">
                {pairs.into_iter().map(|(a_name, b_name, result)| {
                    view! {
                        <div class="pair-item">
                            <div class="plans">{format!("{} \u{2192} {}", a_name, b_name)}</div>
                            <div class="result">{result}</div>
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>
        </div>
    }
}
