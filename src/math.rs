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
