use leptos::prelude::*;
use wasm_bindgen::JsCast;

use crate::data::{self, PROGRAMS, PaymentPlan};
use crate::math;
use crate::components::math_mode::MathMode;

/// Parse the URL hash (e.g. `#p=0&t=2&apy=5.0`) into (program, tuition, apy).
fn parse_hash() -> (Option<usize>, Option<usize>, Option<f64>) {
    let hash = web_sys::window()
        .and_then(|w| w.location().hash().ok())
        .unwrap_or_default();
    let hash = hash.trim_start_matches('#');
    let mut p = None;
    let mut t = None;
    let mut apy = None;
    for part in hash.split('&') {
        if let Some((key, val)) = part.split_once('=') {
            match key {
                "p" => p = val.parse::<usize>().ok(),
                "t" => t = val.parse::<usize>().ok(),
                "apy" => apy = val.parse::<f64>().ok(),
                _ => {}
            }
        }
    }
    (p, t, apy)
}

/// Write current state to the URL hash via `replaceState` (no history entry).
fn write_hash(program: Option<usize>, tuition: Option<usize>, apy: f64) {
    let Some(window) = web_sys::window() else { return };
    let mut parts = Vec::new();
    if let Some(p) = program {
        parts.push(format!("p={}", p));
    }
    if let Some(t) = tuition {
        parts.push(format!("t={}", t));
    }
    parts.push(format!("apy={}", apy));
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
    let (apy_value, set_apy_value) = signal(5.0_f64);

    // Derived: current program
    let current_program = move || selected_program.get().map(|i| &PROGRAMS[i]);

    // Derived: current plans
    let current_plans = move || -> Option<Vec<PaymentPlan>> {
        let prog = current_program()?;
        let ti = selected_tuition.get()?;
        let opt = prog.options.get(ti)?;
        Some(data::build_plans(opt.amount))
    };

    // Handler for program select
    let on_program_select = move |idx: usize| {
        set_selected_program.set(Some(idx));
        let prog = &PROGRAMS[idx];
        if prog.options.len() == 1 {
            set_selected_tuition.set(Some(0));
        } else {
            set_selected_tuition.set(None);
        }
    };

    // On mount: restore state from URL hash, falling back to defaults
    Effect::new(move || {
        let (p, t, apy) = parse_hash();

        let prog_idx = p.filter(|&i| i < PROGRAMS.len()).unwrap_or(0);
        set_selected_program.set(Some(prog_idx));

        let max_t = PROGRAMS[prog_idx].options.len();
        let tui_idx = t.filter(|&i| i < max_t).unwrap_or(2.min(max_t.saturating_sub(1)));
        set_selected_tuition.set(Some(tui_idx));

        if let Some(a) = apy {
            set_apy_value.set(a);
        }
    });

    // Sync signals → URL hash (runs whenever any signal changes)
    Effect::new(move || {
        let p = selected_program.get();
        let t = selected_tuition.get();
        let a = apy_value.get();
        write_hash(p, t, a);
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
                                        on:click=move |_| set_selected_tuition.set(Some(i))
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

        // Results
        <Show when=move || current_plans().is_some()>
            {move || {
                let plans = current_plans().unwrap();
                view! {
                    <SummaryTable plans=plans.clone() />
                    <ApySection plans=plans.clone() apy_value=apy_value set_apy_value=set_apy_value />
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
) -> impl IntoView {
    let plans_clone = plans.clone();
    let ranked = Memo::new(move |_| {
        let apy = apy_value.get() / 100.0;
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
                let apy = apy_value.get();
                let msg = if avg_m == 0.0 {
                    format!("{} \u{2014} The 2% early-payment discount saves the most at {:.1}% APY.", best_name, apy)
                } else if avg_m == 2.0 && best_payments.len() == 1 {
                    format!("{} \u{2014} Paying in full in July (no fee) is optimal. Invest until then.", best_name)
                } else {
                    format!("{} \u{2014} Spreading payments (avg month {:.1}) keeps money invested long enough to outweigh finance fees at {:.1}% APY.", best_name, avg_m, apy)
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
