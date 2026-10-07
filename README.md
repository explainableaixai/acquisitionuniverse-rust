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

<!--expanded-->
## Why a Rust toolkit for deal teams

Most deal tooling is written in languages chosen for convenience. Some teams, especially those that build data platforms, write their pipelines in Rust for speed and safety. If your screened lists flow through such a platform, you do not want to call out to a script for the scoring. This crate puts the same logic inside your binary, with no dependencies and no network.

The crate has a very small surface: a brief type, a company type, a weights type and four functions. That is deliberate. Scoring logic that you can read in one sitting is scoring logic you can defend in a meeting.

## Reading the scoring rules in code

```rust
pub fn composite(c: &Company, w: &Weights) -> f64 {
    let outreach = if c.group_owned { 0.0 } else { c.outreach_suitability };
    let total = w.mandate_fit + w.outreach_suitability + w.transition_context;
    let v = (c.mandate_fit * w.mandate_fit + outreach * w.outreach_suitability + c.transition_context * w.transition_context) / total;
    (v * 10.0).round() / 10.0
}
```

That is the whole rule. Fit has weight 0.7, suitability 0.2 and context 0.1 by default. If the company belongs to a group, suitability is zero before it is weighted. The result is rounded to one decimal. Because the weights are divided by their sum, you can pass any positive numbers and get a score on the same 0 to 100 scale.

## Building a ranked report

A typical binary reads a CSV, builds `Company` values, ranks them and writes a report. With the `csv` crate, the whole thing is short:

```rust
use acquisitionuniverse::{rank, Company, DEFAULT_WEIGHTS};

let mut rdr = csv::Reader::from_path("screen.csv")?;
let mut companies = Vec::new();
for row in rdr.records() {
    let row = row?;
    companies.push(Company {
        id: row[0].to_string(),
        mandate_fit: row[1].parse()?,
        outreach_suitability: row[2].parse()?,
        transition_context: row[3].parse()?,
        group_owned: &row[4] == "yes",
    });
}

for (c, score) in rank(&companies, &DEFAULT_WEIGHTS).into_iter().take(25) {
    println!("{:<10} {:>5.1}", c.id, score);
}
```

The crate does not depend on `csv` or `serde`, so your own binary decides which to bring in.

## Properties worth testing

Because the functions are pure, they are easy to test thoroughly. Four properties catch most mistakes.

1. For inputs between 0 and 100, the composite is between 0 and 100.
2. Marking a company group owned never increases its composite.
3. Raising any single score never lowers the composite when the weights are positive.
4. Ranking is stable: companies with equal composites keep their input order.

A property testing crate can generate random companies and check these in milliseconds.

## Memory and speed

A list of a million companies fits comfortably in memory, and ranking it is a sort. On a laptop it takes well under a second. There is no reason to push this logic into a database unless the rest of your analysis lives there. If it does, port the formula to SQL and test it against this crate with a handful of fixtures.

## About the signals

The 15 common signals are exported as a constant array of names. They describe what a company publishes about itself: ownership character, leadership bench, history, thesis fit, footprint, business model, recurring revenue language, end markets, group membership, management depth, hiring, site activity, partners, certifications and digital maturity. `signal_coverage` accepts pairs of a name and an optional value and returns how many are visible and how many were checked. Names outside the fifteen are ignored, which lets you add vertical specific signals, such as an accreditation scope, without distorting the count.

## Industry context

The best way to understand how these pieces fit together is to read how a screening is used. The page on [thesis universe mapping](https://www.acquisitionuniverse.com/use-cases/thesis-universe-mapping.php) describes counting every company in a thesis, and the guide to [IAB audience taxonomy data](https://www.cookielessaudiences.com/features/iab-audience-taxonomy.php) shows how a different kind of dataset is organised around a standard vocabulary. For people data, the [job title normalization API](https://www.resumereaderapi.com/normalization/job-titles.php) shows how messy labels become canonical ones.

<!--extra-->
## Vertical signals

Screenings add extra signals for the trade in question, for example NADCAP accreditation for coatings or OEM dealer authorizations for material handling services. Keep them in a separate collection from the fifteen common signals. `signal_coverage` counts only the known fifteen, which keeps the measure comparable across verticals, and you can write a second small function for the extras with the same shape.

A closing thought: write the report generator first, even before the importer. Seeing the ranked table with weights and coverage on one page tells you immediately what data you still need, and it keeps the project honest about what the numbers can and cannot say.

<!--further-->
## Further reading and practical notes

The [Rust language site](https://www.rust-lang.org/) is the best starting point for the toolchain and the ecosystem. For anyone who combines screened lists with outreach, the [Federal Trade Commission](https://www.ftc.gov/) publishes guidance on business communications and advertising rules in the United States, which is worth reading before you design a campaign on top of a target list.

A few practical notes for Rust users. Make the weights a configuration value that you read at start up, and print them in the first line of every report. A report without its weights is hard to interpret later. Keep the company type close to your own data model by writing a `From` implementation from your row type, so that adding a field to your table does not touch the scoring code. And write the ranking function so that it never panics on bad input: replace `NaN` with zero at the boundary, because a single `NaN` in a sort can produce an order that looks plausible and is wrong.

Because the crate has no dependencies, it is a good citizen in workspaces with strict dependency policies. Security teams that review third party code can read the entire crate in an afternoon, and that is a feature of a library whose job is to be understood.

## Build and release notes

The crate compiles on stable Rust with edition 2021. It has no features and no build script. Version 0.1.0 is the first release, and the 0.x series may adjust names as the toolkit grows. Pin the version in your manifest and read the changelog before you upgrade.

## Questions

**Why are scores `f64`?** They are percentages with one decimal. Floats are the simplest honest representation.

**Why no serde?** To keep the crate dependency free. Derive your own row type.

**Can I add my own signals?** Yes, pass them to `signal_coverage`. They are ignored in the count unless they are among the fifteen, so keep your own tally for them.

## FAQ

**Why f64?** Scores are percentages with one decimal.

**Can it read the delivered CSV?** Not directly. Read it with the `csv` crate and map columns.

**Edition?** 2021. MIT license. info@alpha-quantum.com
