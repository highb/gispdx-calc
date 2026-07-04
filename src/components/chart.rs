use leptos::prelude::*;
use wasm_bindgen::JsCast;

use crate::data::PaymentPlan;
use crate::math;

const PLAN_COLORS: &[&str] = &["#e74c3c", "#3498db", "#2ecc71", "#f39c12", "#9b59b6"];
const PLAN_COLORS_DARK: &[&str] = &["#ff6b6b", "#5dade2", "#58d68d", "#f5b041", "#bb8fce"];
const NUM_POINTS: usize = 301;

fn build_chart_json(plans: &[PaymentPlan], current_apy: f64, dark: bool) -> String {
    let colors = if dark { PLAN_COLORS_DARK } else { PLAN_COLORS };

    // Build datasets
    let mut datasets = String::from("[");
    for (i, plan) in plans.iter().enumerate() {
        if i > 0 {
            datasets.push(',');
        }
        let color = colors.get(i).unwrap_or(&"#888");
        let short_name = plan.name.split('(').next().unwrap_or(&plan.name).trim();
        datasets.push_str(&format!(
            r#"{{"label":"{}","borderColor":"{}","backgroundColor":"{}","pointRadius":0,"borderWidth":2,"tension":0.1,"showLine":true,"data":["#,
            short_name, color, color
        ));
        for j in 0..NUM_POINTS {
            let apy = j as f64 / (NUM_POINTS - 1) as f64 * 0.30;
            let cost = math::effective_cost(&plan.payments, apy);
            if j > 0 {
                datasets.push(',');
            }
            datasets.push_str(&format!("{{\"x\":{:.4},\"y\":{:.2}}}", apy * 100.0, cost));
        }
        datasets.push_str("]}");
    }
    datasets.push(']');

    // Compute crossover points
    let mut crossovers = String::from("[");
    let mut first = true;
    let n = plans.len();
    for i in 0..n {
        for j in (i + 1)..n {
            if let Some(be) = math::breakeven_apy(&plans[i].payments, &plans[j].payments) {
                if be > 0.0 && be < 0.30 {
                    let cost = math::effective_cost(&plans[i].payments, be);
                    if !first {
                        crossovers.push(',');
                    }
                    first = false;
                    crossovers.push_str(&format!(
                        r#"{{"x":{:.4},"y":{:.2}}}"#,
                        be * 100.0, cost
                    ));
                }
            }
        }
    }
    crossovers.push(']');

    format!(
        r#"{{"datasets":{},"crossovers":{},"currentApy":{:.4},"dark":{}}}"#,
        datasets, crossovers, current_apy, dark
    )
}

fn render_chart(json: &str) {
    let code = format!(
        r##"
        (function() {{
            var data = {json};
            var canvas = document.getElementById('costChart');
            if (!canvas || typeof Chart === 'undefined') return;

            if (window._costChart) {{
                window._costChart.destroy();
                window._costChart = null;
            }}

            var gridColor = data.dark ? 'rgba(255,255,255,0.1)' : 'rgba(0,0,0,0.1)';
            var textColor = data.dark ? '#8b949e' : '#4a5464';

            if (data.crossovers.length > 0) {{
                data.datasets.push({{
                    label: 'Crossover',
                    data: data.crossovers,
                    pointRadius: 6,
                    pointStyle: 'crossRot',
                    pointBorderWidth: 2,
                    pointBorderColor: data.dark ? '#e6edf3' : '#333',
                    backgroundColor: 'transparent',
                    showLine: false
                }});
            }}

            window._costChart = new Chart(canvas, {{
                type: 'scatter',
                data: {{ datasets: data.datasets }},
                options: {{
                    responsive: true,
                    maintainAspectRatio: false,
                    interaction: {{ mode: 'nearest', axis: 'x', intersect: false }},
                    scales: {{
                        x: {{
                            type: 'linear',
                            title: {{ display: true, text: 'APY (%)', color: textColor }},
                            min: 0, max: 30,
                            ticks: {{ color: textColor, callback: function(v) {{ return v + '%'; }} }},
                            grid: {{ color: gridColor }}
                        }},
                        y: {{
                            title: {{ display: true, text: 'Effective Cost ($)', color: textColor }},
                            ticks: {{
                                color: textColor,
                                callback: function(v) {{ return '$' + v.toLocaleString(); }}
                            }},
                            grid: {{ color: gridColor }}
                        }}
                    }},
                    plugins: {{
                        legend: {{
                            labels: {{
                                color: textColor,
                                filter: function(item) {{ return item.text !== 'Crossover'; }}
                            }}
                        }},
                        tooltip: {{
                            callbacks: {{
                                label: function(ctx) {{
                                    if (ctx.dataset.label === 'Crossover') return 'Crossover at ' + ctx.parsed.x.toFixed(1) + '%';
                                    return ctx.dataset.label + ': $' + ctx.parsed.y.toLocaleString(undefined, {{minimumFractionDigits:2, maximumFractionDigits:2}});
                                }},
                                title: function(items) {{
                                    return 'APY: ' + items[0].parsed.x.toFixed(1) + '%';
                                }}
                            }}
                        }},
                        annotation: {{
                            annotations: {{
                                currentApy: {{
                                    type: 'line',
                                    xMin: data.currentApy,
                                    xMax: data.currentApy,
                                    borderColor: data.dark ? 'rgba(255,255,255,0.5)' : 'rgba(0,0,0,0.4)',
                                    borderWidth: 2,
                                    borderDash: [6, 4],
                                    label: {{
                                        display: true,
                                        content: 'Your APY: ' + data.currentApy.toFixed(1) + '%',
                                        position: 'start',
                                        backgroundColor: data.dark ? 'rgba(30,40,50,0.9)' : 'rgba(255,255,255,0.9)',
                                        color: data.dark ? '#d1d5db' : '#333',
                                        font: {{ size: 11 }}
                                    }}
                                }}
                            }}
                        }}
                    }}
                }}
            }});
        }})()
        "##,
        json = json
    );
    js_sys::eval(&code).ok();
}

#[component]
pub fn CostChart(
    plans: Vec<PaymentPlan>,
    apy_value: Signal<f64>,
    dark: ReadSignal<bool>,
) -> impl IntoView {
    Effect::new(move || {
        let apy = apy_value.get();
        let is_dark = dark.get();
        let json = build_chart_json(&plans, apy, is_dark);
        // Use setTimeout(0) to ensure canvas is in the DOM before rendering
        let cb = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
            render_chart(&json);
        }) as Box<dyn Fn()>);
        let _ = web_sys::window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                cb.as_ref().unchecked_ref(),
                0,
            );
        cb.forget();
    });

    view! {
        <div class="card chart-card">
            <h2>"Effective Cost vs. APY"</h2>
            <div class="chart-container">
                <canvas id="costChart"></canvas>
            </div>
        </div>
    }
}
