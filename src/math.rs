const BILLING_MONTHS: f64 = 13.0;
const BILLING_YEARS: f64 = BILLING_MONTHS / 12.0;

pub fn effective_cost(payments: &[(f64, f64)], apy: f64) -> f64 {
    payments.iter().map(|&(t, amt)| {
        let remaining = BILLING_YEARS - t / 12.0;
        amt * (1.0 + apy).powf(remaining)
    }).sum()
}

pub fn total_dollars(payments: &[(f64, f64)]) -> f64 {
    payments.iter().map(|&(_, a)| a).sum()
}

pub fn weighted_avg_month(payments: &[(f64, f64)]) -> f64 {
    let tot = total_dollars(payments);
    if tot == 0.0 {
        return 0.0;
    }
    payments.iter().map(|&(t, a)| t * a).sum::<f64>() / tot
}

pub fn breakeven_apy(plan_a: &[(f64, f64)], plan_b: &[(f64, f64)]) -> Option<f64> {
    let mut lo = -0.5_f64;
    let mut hi = 20.0_f64;
    let tol = 1e-9;

    let diff = |r: f64| effective_cost(plan_a, r) - effective_cost(plan_b, r);

    let mut fa = diff(lo);
    let fb = diff(hi);
    if fa * fb > 0.0 {
        return None;
    }

    for _ in 0..200 {
        let mid = (lo + hi) / 2.0;
        let fm = diff(mid);
        if fm == 0.0 || (hi - lo) / 2.0 < tol {
            return Some(mid);
        }
        if fa * fm < 0.0 {
            hi = mid;
        } else {
            lo = mid;
            fa = fm;
        }
    }
    Some((lo + hi) / 2.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::build_plans;

    const EPS_DOLLAR: f64 = 0.01;
    const EPS_APY: f64 = 0.001;

    // ── effective_cost ───────────────────────────────────────────────

    #[test]
    fn effective_cost_zero_apy_equals_total() {
        let payments = vec![(0.0, 5000.0), (3.0, 3000.0), (6.0, 2000.0)];
        let ec = effective_cost(&payments, 0.0);
        let td = total_dollars(&payments);
        assert!((ec - td).abs() < EPS_DOLLAR, "ec={ec}, td={td}");
    }

    #[test]
    fn effective_cost_single_payment_t0_5pct() {
        let amt = 1000.0;
        let payments = vec![(0.0, amt)];
        // remaining = 13/12, FV = 1000 * 1.05^(13/12)
        let expected = amt * 1.05_f64.powf(13.0 / 12.0);
        let ec = effective_cost(&payments, 0.05);
        assert!(
            (ec - expected).abs() < EPS_DOLLAR,
            "ec={ec}, expected={expected}"
        );
    }

    #[test]
    fn effective_cost_single_payment_t13() {
        let amt = 7500.0;
        let payments = vec![(13.0, amt)];
        // remaining = 0, FV = amt * (1.05)^0 = amt
        let ec = effective_cost(&payments, 0.05);
        assert!(
            (ec - amt).abs() < EPS_DOLLAR,
            "ec={ec}, expected={amt}"
        );
    }

    #[test]
    fn effective_cost_multiple_payments_sum_of_fvs() {
        let payments = vec![(0.0, 1000.0), (6.0, 2000.0), (12.0, 500.0)];
        let apy: f64 = 0.08;
        let expected: f64 = payments.iter().map(|&(t, a)| {
            a * (1.0_f64 + apy).powf(13.0 / 12.0 - t / 12.0)
        }).sum();
        let ec = effective_cost(&payments, apy);
        assert!(
            (ec - expected).abs() < EPS_DOLLAR,
            "ec={ec}, expected={expected}"
        );
    }

    // ── total_dollars ────────────────────────────────────────────────

    #[test]
    fn total_dollars_sums_correctly() {
        let payments = vec![(1.0, 100.0), (2.0, 200.0), (3.0, 300.0)];
        assert!((total_dollars(&payments) - 600.0).abs() < EPS_DOLLAR);
    }

    #[test]
    fn total_dollars_empty() {
        let payments: Vec<(f64, f64)> = vec![];
        assert!((total_dollars(&payments) - 0.0).abs() < EPS_DOLLAR);
    }

    // ── weighted_avg_month ───────────────────────────────────────────

    #[test]
    fn weighted_avg_month_single_payment() {
        let payments = vec![(5.0, 1000.0)];
        assert!(
            (weighted_avg_month(&payments) - 5.0).abs() < EPS_DOLLAR
        );
    }

    #[test]
    fn weighted_avg_month_equal_payments_at_2_and_8() {
        let payments = vec![(2.0, 500.0), (8.0, 500.0)];
        assert!(
            (weighted_avg_month(&payments) - 5.0).abs() < EPS_DOLLAR
        );
    }

    #[test]
    fn weighted_avg_month_empty_returns_zero() {
        let payments: Vec<(f64, f64)> = vec![];
        assert!((weighted_avg_month(&payments) - 0.0).abs() < EPS_DOLLAR);
    }

    #[test]
    fn weighted_avg_month_zero_amounts_returns_zero() {
        let payments = vec![(3.0, 0.0), (7.0, 0.0)];
        assert!((weighted_avg_month(&payments) - 0.0).abs() < EPS_DOLLAR);
    }

    // ── breakeven_apy ────────────────────────────────────────────────

    #[test]
    fn breakeven_apy_identical_plans_returns_some() {
        // diff(r) = 0 for all r. fa=0, fb=0, fa*fb=0 which is NOT > 0,
        // so we enter the loop. fm=0 on the first iteration → return Some(mid).
        let plan = vec![(2.0, 10000.0)];
        let result = breakeven_apy(&plan, &plan);
        assert!(result.is_some());
    }

    #[test]
    fn breakeven_apy_may1_vs_july1_near_12_89_pct() {
        let tuition = 20500.0;
        let plan_a = vec![(0.0, tuition * 0.98)]; // May 1, 2% discount
        let plan_b = vec![(2.0, tuition)];         // July 1, full price
        let result = breakeven_apy(&plan_a, &plan_b).expect("should find breakeven");
        // Analytically: (1+r)^(1/6) = 20500/20090 → r = (20500/20090)^6 - 1 ≈ 0.12886
        let expected = (20500.0_f64 / 20090.0).powi(6) - 1.0;
        assert!(
            (result - expected).abs() < EPS_APY,
            "result={result}, expected={expected}"
        );
        // Also confirm it rounds to 12.89%
        let pct = result * 100.0;
        assert!(
            (pct - 12.89).abs() < 0.01,
            "breakeven pct={pct}, expected ~12.89"
        );
    }

    #[test]
    fn breakeven_apy_always_cheaper_returns_none() {
        // Same timing, different amounts → diff never crosses zero
        let plan_a = vec![(2.0, 1000.0)];
        let plan_b = vec![(2.0, 2000.0)];
        assert!(
            breakeven_apy(&plan_a, &plan_b).is_none(),
            "one plan always cheaper → should be None"
        );
    }

    // ── build_plans invariants ────────────────────────────────────────

    #[test]
    fn build_plans_pay_in_full_may() {
        let tuition = 20500.0;
        let plans = build_plans(tuition);
        let may = &plans[0];
        let td = total_dollars(&may.payments);
        let expected = tuition * 0.98;
        assert!(
            (td - expected).abs() < EPS_DOLLAR,
            "May total={td}, expected={expected}"
        );
    }

    #[test]
    fn build_plans_pay_in_full_july() {
        let tuition = 20500.0;
        let plans = build_plans(tuition);
        let july = &plans[1];
        let td = total_dollars(&july.payments);
        assert!(
            (td - tuition).abs() < EPS_DOLLAR,
            "July total={td}, expected={tuition}"
        );
    }

    #[test]
    fn build_plans_semi_annual_total_equals_tuition_plus_fees() {
        let tuition = 20500.0;
        let plans = build_plans(tuition);
        let semi = &plans[2];
        let td = total_dollars(&semi.payments);
        let expected = tuition + semi.fees;
        assert!(
            (td - expected).abs() < EPS_DOLLAR,
            "Semi total={td}, expected={expected}"
        );
    }

    #[test]
    fn build_plans_quarterly_total_equals_tuition_plus_fees() {
        let tuition = 20500.0;
        let plans = build_plans(tuition);
        let quarterly = &plans[3];
        let td = total_dollars(&quarterly.payments);
        let expected = tuition + quarterly.fees;
        assert!(
            (td - expected).abs() < EPS_DOLLAR,
            "Quarterly total={td}, expected={expected}"
        );
    }

    #[test]
    fn build_plans_monthly_total_equals_tuition_plus_fees() {
        let tuition = 20500.0;
        let plans = build_plans(tuition);
        let monthly = &plans[4];
        let td = total_dollars(&monthly.payments);
        let expected = tuition + monthly.fees;
        assert!(
            (td - expected).abs() < EPS_DOLLAR,
            "Monthly total={td}, expected={expected}"
        );
    }

    #[test]
    fn build_plans_monthly_has_10_payments_starting_at_month_2() {
        let tuition = 20500.0;
        let plans = build_plans(tuition);
        let monthly = &plans[4];
        assert_eq!(monthly.payments.len(), 10);
        for (i, &(t, _)) in monthly.payments.iter().enumerate() {
            let expected_t = 2.0 + i as f64;
            assert!(
                (t - expected_t).abs() < EPS_DOLLAR,
                "payment {i}: month={t}, expected={expected_t}"
            );
        }
    }

    #[test]
    fn build_plans_quarterly_has_4_payments_at_correct_months() {
        let tuition = 20500.0;
        let plans = build_plans(tuition);
        let quarterly = &plans[3];
        assert_eq!(quarterly.payments.len(), 4);
        let expected_months = [2.0, 5.0, 8.0, 11.0];
        for (i, &(t, _)) in quarterly.payments.iter().enumerate() {
            assert!(
                (t - expected_months[i]).abs() < EPS_DOLLAR,
                "payment {i}: month={t}, expected={}",
                expected_months[i]
            );
        }
    }
}
