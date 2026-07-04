#[derive(Clone, Debug)]
pub struct Program {
    pub id: &'static str,
    pub label: &'static str,
    pub desc: &'static str,
    pub options: &'static [TuitionOption],
}

#[derive(Clone, Debug, Copy)]
pub struct TuitionOption {
    pub id: &'static str,
    pub label: &'static str,
    pub amount: f64,
}

#[derive(Clone, Debug)]
pub struct PaymentPlan {
    pub name: String,
    pub payments: Vec<(f64, f64)>, // (month, amount)
    pub fees: f64,
    pub discount: f64,
}

pub const PROGRAMS: &[Program] = &[
    Program {
        id: "twos",
        label: "Twos Program",
        desc: "2 year olds",
        options: &[
            TuitionOption { id: "t1", label: "3 Full Days", amount: 15300.0 },
            TuitionOption { id: "t2", label: "4 Full Days", amount: 18200.0 },
            TuitionOption { id: "t3", label: "5 Full Days", amount: 20500.0 },
            TuitionOption { id: "t4", label: "3/4/5 Half Days", amount: 17850.0 },
        ],
    },
    Program {
        id: "preschool",
        label: "Preschool",
        desc: "3 & 4 year olds",
        options: &[
            TuitionOption { id: "p1", label: "3 Full Days", amount: 14900.0 },
            TuitionOption { id: "p2", label: "4 Full Days", amount: 17450.0 },
            TuitionOption { id: "p3", label: "5 Full Days", amount: 19700.0 },
            TuitionOption { id: "p4", label: "3/4/5 Half Days", amount: 17220.0 },
        ],
    },
    Program {
        id: "kinder",
        label: "Kindergarten",
        desc: "5 yrs old by Sep 1",
        options: &[
            TuitionOption { id: "k1", label: "5 Full Days", amount: 19900.0 },
        ],
    },
    Program {
        id: "grade",
        label: "Grade School",
        desc: "1st through 4th",
        options: &[
            TuitionOption { id: "g1", label: "5 Full Days", amount: 20150.0 },
        ],
    },
    Program {
        id: "middle",
        label: "Middle School",
        desc: "5th through 8th",
        options: &[
            TuitionOption { id: "m1", label: "5 Full Days", amount: 20150.0 },
        ],
    },
];

pub fn build_plans(tuition: f64) -> Vec<PaymentPlan> {
    let semi_each = (tuition + 2.0 * 90.0) / 2.0;
    let quarterly_each = (tuition + 4.0 * 70.0) / 4.0;
    let monthly_each = (tuition + 10.0 * 30.0) / 10.0;

    vec![
        PaymentPlan {
            name: "Pay in full May 1 (2% discount)".into(),
            payments: vec![(0.0, tuition * 0.98)],
            fees: 0.0,
            discount: tuition * 0.02,
        },
        PaymentPlan {
            name: "Pay in full July 1 (no fee)".into(),
            payments: vec![(2.0, tuition)],
            fees: 0.0,
            discount: 0.0,
        },
        PaymentPlan {
            name: "Semi-annual (2 pmts, $90/invoice)".into(),
            payments: vec![(2.0, semi_each), (8.0, semi_each)],
            fees: 2.0 * 90.0,
            discount: 0.0,
        },
        PaymentPlan {
            name: "Quarterly (4 pmts, $70/invoice)".into(),
            payments: vec![
                (2.0, quarterly_each),
                (5.0, quarterly_each),
                (8.0, quarterly_each),
                (11.0, quarterly_each),
            ],
            fees: 4.0 * 70.0,
            discount: 0.0,
        },
        PaymentPlan {
            name: "Monthly (10 pmts, $30/invoice)".into(),
            payments: (0..10).map(|i| (2.0 + i as f64, monthly_each)).collect(),
            fees: 10.0 * 30.0,
            discount: 0.0,
        },
    ]
}
