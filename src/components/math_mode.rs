use leptos::prelude::*;
use leptos::*;
use wasm_bindgen::JsCast;

#[component]
pub fn MathMode() -> impl IntoView {
    let (open, set_open) = signal(false);
    let rendered = std::cell::Cell::new(false);

    let toggle = move |_| {
        let now = !open.get();
        set_open.set(now);
        if now && !rendered.get() {
            rendered.set(true);
            // Render KaTeX after DOM update
            request_animation_frame(move || {
                render_katex();
            });
        }
    };

    view! {
        <div class="card math-mode-card">
            <button class="math-toggle" on:click=toggle>
                <span class="math-toggle-icon">"\u{1D453}(x)"</span>
                <span>"Math Mode"</span>
                <span class="math-toggle-arrow" class:open=open>"\u{25B6}"</span>
            </button>
            <div class="math-body" style:display=move || if open.get() { "" } else { "none" }>
                <MathContent />
            </div>
        </div>
    }
}

fn render_katex() {
    use web_sys::wasm_bindgen::JsValue;
    let window = web_sys::window().unwrap();
    // Check if katex is loaded
    let katex = js_sys::Reflect::get(&window, &JsValue::from_str("katex"));
    match katex {
        Ok(k) if !k.is_undefined() => do_render_katex(),
        _ => {
            // Poll until katex loads
            let cb = wasm_bindgen::closure::Closure::wrap(Box::new(|| {
                render_katex();
            }) as Box<dyn Fn()>);
            window.set_timeout_with_callback_and_timeout_and_arguments_0(
                cb.as_ref().unchecked_ref(), 50
            ).ok();
            cb.forget();
        }
    }
}

fn do_render_katex() {
    // Use JS eval to render all math elements
    let code = r#"
        document.querySelectorAll('.math-block[data-tex]').forEach(el => {
            katex.render(el.dataset.tex, el, { displayMode: true, throwOnError: false });
        });
        document.querySelectorAll('.math-inline[data-tex]').forEach(el => {
            katex.render(el.dataset.tex, el, { displayMode: false, throwOnError: false });
        });
    "#;
    js_sys::eval(code).ok();
}

#[component]
fn MathContent() -> impl IntoView {
    view! {
        <section class="math-section">
            <h3>"Overview"</h3>
            <p>"This calculator compares the " <em>"true economic cost"</em> " of each payment plan by accounting for the "
                <strong>"time value of money"</strong>
                ". If you can earn interest (APY) on uninvested tuition dollars, paying later is worth more than the invoice amount suggests. The question is: does the interest earned by delaying payment outweigh any fees charged for installment plans (or the discount lost by not paying early)?"
            </p>
        </section>

        <section class="math-section">
            <h3>"Timeline"</h3>
            <p>"All payments are mapped onto a 13-month timeline:"</p>
            <div class="math-block" data-tex=r"t = 0 \;\text{(May 1, 2026)} \quad\text{through}\quad t = 13 \;\text{months (Jun 1, 2027)}"></div>
            <p>"Each payment occurs at month " <span class="math-inline" data-tex="t_i"></span>
                ". The billing period spans " <span class="math-inline" data-tex="T = 13"></span>
                " months, or " <span class="math-inline" data-tex="T/12"></span>
                " years. We evaluate all costs as of " <span class="math-inline" data-tex="t = T"></span>
                " (the end of the school year)."
            </p>
        </section>

        <section class="math-section">
            <h3>"1. Effective Cost (Future Value)"</h3>
            <p>"Each payment of " <span class="math-inline" data-tex="X_i"></span>
                " dollars at month " <span class="math-inline" data-tex="t_i"></span>
                " has a " <strong>"future value"</strong>
                " at the end of the billing period. This represents what that money "
                <em>"could have grown to"</em> " if left invested:"
            </p>
            <div class="math-block" data-tex=r"\text{Effective Cost} = \sum_{i} X_i \cdot (1 + r)^{(T - t_i)/12}"></div>
            <p>"where " <span class="math-inline" data-tex="r"></span>
                " is the annual percentage yield (APY). The exponent "
                <span class="math-inline" data-tex="(T - t_i)/12"></span>
                " converts the remaining months into years for the annual rate."
            </p>
            <div class="math-example">
                <strong>"Example:"</strong> " Pay $20,090 in full on May 1 ("
                <span class="math-inline" data-tex="t=0"></span> ") at 5% APY:"
                <br />
                <div class="math-block" data-tex=r"20{,}090 \times (1.05)^{13/12} = 20{,}090 \times 1.05406 = \$21{,}176.09"></div>
                <p>"The effective cost is higher than the invoice because you gave up 13 months of potential investment returns."</p>
            </div>
        </section>

        <section class="math-section">
            <h3>"2. Payment Plan Structures"</h3>
            <p>"Each plan generates a specific set of "
                <span class="math-inline" data-tex="(t_i, X_i)"></span> " pairs:"
            </p>
            <table class="math-plan-table">
                <thead><tr><th>"Plan"</th><th>"Payments"</th><th>"Formula"</th></tr></thead>
                <tbody>
                    <tr>
                        <td>"May 1 (2% disc.)"</td>
                        <td>"1 payment at " <span class="math-inline" data-tex="t{=}0"></span></td>
                        <td><span class="math-inline" data-tex=r"X = 0.98 \cdot C"></span></td>
                    </tr>
                    <tr>
                        <td>"July 1 (no fee)"</td>
                        <td>"1 payment at " <span class="math-inline" data-tex="t{=}2"></span></td>
                        <td><span class="math-inline" data-tex="X = C"></span></td>
                    </tr>
                    <tr>
                        <td>"Semi-annual"</td>
                        <td>"2 payments at " <span class="math-inline" data-tex=r"t{=}2,\,8"></span></td>
                        <td><span class="math-inline" data-tex=r"X = (C + 2 \times 90)\,/\,2"></span></td>
                    </tr>
                    <tr>
                        <td>"Quarterly"</td>
                        <td>"4 payments at " <span class="math-inline" data-tex=r"t{=}2,\,5,\,8,\,11"></span></td>
                        <td><span class="math-inline" data-tex=r"X = (C + 4 \times 70)\,/\,4"></span></td>
                    </tr>
                    <tr>
                        <td>"Monthly"</td>
                        <td>"10 payments at " <span class="math-inline" data-tex=r"t{=}2,\,3,\,\ldots,\,11"></span></td>
                        <td><span class="math-inline" data-tex=r"X = (C + 10 \times 30)\,/\,10"></span></td>
                    </tr>
                </tbody>
            </table>
            <p>"where " <span class="math-inline" data-tex="C"></span>
                " is the base tuition. Each installment amount "
                <span class="math-inline" data-tex="X"></span>
                " includes the per-invoice fee distributed evenly."
            </p>
        </section>

        <section class="math-section">
            <h3>"3. Total Paid & Weighted Average Month"</h3>
            <p><strong>"Total paid"</strong> " is simply the sum of all payment amounts (the cash that actually leaves your account):"</p>
            <div class="math-block" data-tex=r"\text{Total Paid} = \sum_{i} X_i"></div>
            <p>"The " <strong>"weighted average payment month"</strong> " tells you when, on average, your money leaves:"</p>
            <div class="math-block" data-tex=r"\bar{t} = \frac{\sum_{i}\, t_i \cdot X_i}{\sum_{i}\, X_i}"></div>
            <p>"A higher " <span class="math-inline" data-tex=r"\bar{t}"></span>
                " means money stays invested longer, earning more interest before being spent on tuition."
            </p>
        </section>

        <section class="math-section">
            <h3>"4. Breakeven APY (Bisection Method)"</h3>
            <p>"The breakeven APY is the rate " <span class="math-inline" data-tex="r^*"></span>
                " at which two plans have " <em>"identical"</em> " effective cost:"
            </p>
            <div class="math-block" data-tex=r"\sum_{i} X_i^{(A)} \cdot (1+r^*)^{(T-t_i^{(A)})/12} \;=\; \sum_{j} X_j^{(B)} \cdot (1+r^*)^{(T-t_j^{(B)})/12}"></div>
            <p>"Equivalently, we find the root of:"</p>
            <div class="math-block" data-tex=r"f(r) = \text{EffCost}_A(r) - \text{EffCost}_B(r) = 0"></div>
            <p>"Since " <span class="math-inline" data-tex="f(r)"></span>
                " is continuous and monotonic over the relevant range, we use the "
                <strong>"bisection method"</strong> ":"
            </p>
            <ol class="math-steps">
                <li>"Start with bounds " <span class="math-inline" data-tex=r"[lo, hi] = [-0.5,\; 20]"></span> " (i.e., \u{2212}50% to 2000% APY)"</li>
                <li>"Compute midpoint " <span class="math-inline" data-tex=r"m = (lo + hi)\,/\,2"></span></li>
                <li>"Evaluate " <span class="math-inline" data-tex="f(m)"></span></li>
                <li>"If " <span class="math-inline" data-tex="f(lo)"></span> " and " <span class="math-inline" data-tex="f(m)"></span> " have opposite signs, the root is in " <span class="math-inline" data-tex="[lo, m]"></span> "; otherwise it\u{2019}s in " <span class="math-inline" data-tex="[m, hi]"></span></li>
                <li>"Repeat for up to 200 iterations or until " <span class="math-inline" data-tex="|hi - lo| < 10^{-9}"></span></li>
            </ol>
            <p>"This converges to a precision of about " <span class="math-inline" data-tex="10^{-9}"></span>
                " (displayed as 2 decimal places of percent)."
            </p>
        </section>

        <section class="math-section">
            <h3>"5. Interpretation"</h3>
            <p>"At any given APY, the plan with the " <strong>"lowest effective cost"</strong>
                " is the best choice. The breakeven APY between two plans tells you the switching point:"
            </p>
            <ul class="math-list">
                <li><strong>"Below"</strong> " the breakeven: the plan that pays " <em>"earlier"</em> " (getting discount or avoiding fees) wins"</li>
                <li><strong>"Above"</strong> " the breakeven: the plan that pays " <em>"later"</em> " (keeping money invested) wins"</li>
            </ul>
            <div class="math-example">
                <strong>"Example:"</strong> " For $20,500 tuition (Twos, 5 Full Days), the breakeven between May 1 (2% discount) and July 1 (no fee) is:"
                <div class="math-block" data-tex=r"r^* = 12.89\%"></div>
                <p>"If your savings account/investment earns more than 12.89% APY, skip the early-pay discount and invest until July. Below 12.89%, the 2% discount is worth more than two months of investment returns."</p>
            </div>
        </section>

        <section class="math-section">
            <h3>"6. Assumptions & Limitations"</h3>
            <ul class="math-list">
                <li>"The $1,000 non-refundable deposit is excluded (same for all plans, so it doesn\u{2019}t affect comparison)"</li>
                <li>"APY is assumed constant over the 13-month period"</li>
                <li>"No tax effects are modeled (interest income may be taxable)"</li>
                <li>"Payments are assumed to occur on the 1st of each month"</li>
                <li>"No risk premium is considered \u{2014} the APY you enter should reflect your actual risk-free return (e.g., HYSA or T-bill rate)"</li>
            </ul>
        </section>
    }
}
