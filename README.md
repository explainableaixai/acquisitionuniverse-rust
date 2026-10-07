# acquisitionuniverse for Rust

A tiny crate for deal teams that keep their target lists in Rust tools. It scores companies with a weighted composite, ranks them, validates a thesis brief and counts which of the 15 common signals are visible.

No dependencies. No network. Compiles in a second.

```toml
[dependencies]
acquisitionuniverse = "0.1"
```

## What a screen gives you

A [thesis universe mapping](https://www.acquisitionuniverse.com/use-cases/thesis-universe-mapping.php) run returns every company in scope with evidence for each signal. This crate is the layer you put on top when you want to sort, filter and brief people from your own code.

## Types and functions

| Item | Description |
|---|---|
| `ThesisBrief` | struct with `validate()` and `to_text()` |
| `Company` | id, three scores and a `group_owned` flag |
| `Weights`, `DEFAULT_WEIGHTS` | 0.7, 0.2, 0.1 |
| `composite(&Company, &Weights)` | f64 on a 0 to 100 scale |
| `rank(&[Company], &Weights)` | `Vec<(Company, f64)>`, best first |
| `signal_coverage(&[(&str, Option<&str>)])` | `(visible, checked)` |
| `SIGNALS` | array of 15 names |
| `PILOT_URL` | page to request a pilot |

## Rank a list

```rust
use acquisitionuniverse::{rank, Company, DEFAULT_WEIGHTS};

let list = vec![
    Company { id: "M-01".into(), mandate_fit: 96.0, outreach_suitability: 85.0, transition_context: 70.0, group_owned: false },
    Company { id: "M-02".into(), mandate_fit: 98.0, outreach_suitability: 90.0, transition_context: 80.0, group_owned: true },
];

for (c, score) in rank(&list, &DEFAULT_WEIGHTS) {
    println!("{} {score}", c.id);
}
```

M-02 fits slightly better but is group-owned. Its outreach score is treated as zero, so M-01 is first.

## Validate and print a brief

```rust
use acquisitionuniverse::ThesisBrief;

let brief = ThesisBrief {
    name: "Precision machining, Southeast".into(),
    vertical: "Precision machining".into(),
    regions: vec!["Georgia".into(), "Alabama".into()],
    employees_min: Some(15),
    employees_max: Some(120),
    exclusions: vec!["group-owned".into()],
    ..Default::default()
};

assert!(brief.validate().is_empty());
println!("{}", brief.to_text());
```

## Coverage

```rust
use acquisitionuniverse::{signal_coverage, SIGNALS};

let found = [(SIGNALS[0], Some("family associated")), (SIGNALS[5], None), (SIGNALS[13], Some("AS9100D"))];
let (visible, checked) = signal_coverage(&found);
println!("{visible} of {checked} visible");
```

Names outside the 15 are ignored, which lets you pass extra industry signals without noise.

## Pair it with serde

The crate has no serde dependency on purpose. Derive your own row type, map it into `Company`, and call `rank`. That keeps compile times short for CLI tools.

## Other Alpha Quantum data products

Ad tech teams may want the [IAB audience taxonomy data](https://www.cookielessaudiences.com/features/iab-audience-taxonomy.php) behind cookieless planning. HR software teams may prefer a [job title normalization API](https://www.resumereaderapi.com/normalization/job-titles.php).

## FAQ

**Why f64?** Scores are percentages with one decimal.

**Can it read the delivered CSV?** Not directly. Read it with the `csv` crate and map columns.

**Edition?** 2021. MIT license. info@alpha-quantum.com
