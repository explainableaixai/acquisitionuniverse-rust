//! Local toolkit for acquisition target screening. No network calls.

pub const PILOT_URL: &str = "https://www.acquisitionuniverse.com/free-pilot.php";

/// The 15 common signals, each recorded with a verbatim quote and a source URL, or "not visible".
pub const SIGNALS: [&str; 15] = [
    "Founder or family association",
    "Visible leadership bench depth",
    "Operating history and continued independence",
    "Strategic fit to thesis",
    "Geographic and branch footprint",
    "Service-led vs product-led model",
    "Recurring-offering indicators",
    "Vertical specialization and end-market exposure",
    "Acquisition-program or roll-up readiness",
    "Management professionalization",
    "Hiring posture and functional investment",
    "Website and news activity trajectory",
    "Partner and channel ecosystem position",
    "Compliance and regulated-market readiness",
    "Digital-commercial maturity",
];

#[derive(Debug, Clone, Copy)]
pub struct Weights {
    pub mandate_fit: f64,
    pub outreach_suitability: f64,
    pub transition_context: f64,
}

pub const DEFAULT_WEIGHTS: Weights = Weights { mandate_fit: 0.7, outreach_suitability: 0.2, transition_context: 0.1 };

#[derive(Debug, Clone, Default)]
pub struct ThesisBrief {
    pub name: String,
    pub vertical: String,
    pub subverticals: Vec<String>,
    pub regions: Vec<String>,
    pub employees_min: Option<u32>,
    pub employees_max: Option<u32>,
    pub must_have: Vec<String>,
    pub exclusions: Vec<String>,
    pub notes: String,
}

impl ThesisBrief {
    /// Problems that would make the brief ambiguous to a screening team.
    pub fn validate(&self) -> Vec<&'static str> {
        let mut out = Vec::new();
        if self.vertical.is_empty() {
            out.push("vertical is empty");
        }
        if self.regions.is_empty() {
            out.push("no regions listed");
        }
        if let (Some(a), Some(b)) = (self.employees_min, self.employees_max) {
            if a > b {
                out.push("employees_min is above employees_max");
            }
        }
        if self.exclusions.is_empty() {
            out.push("no exclusions listed, group-owned companies are the usual first one");
        }
        out
    }

    /// Plain text brief, ready to paste into a pilot request.
    pub fn to_text(&self) -> String {
        let join = |v: &Vec<String>, sep: &str, empty: &str| if v.is_empty() { empty.to_string() } else { v.join(sep) };
        let size = match (self.employees_min, self.employees_max) {
            (None, None) => "not specified".to_string(),
            (a, b) => format!(
                "{} to {} employees",
                a.map_or("?".to_string(), |v| v.to_string()),
                b.map_or("?".to_string(), |v| v.to_string())
            ),
        };
        let mut lines = vec![
            format!("Thesis: {}", self.name),
            format!("Vertical: {}", self.vertical),
            format!("Subverticals: {}", join(&self.subverticals, ", ", "all")),
            format!("Regions: {}", join(&self.regions, ", ", "not specified")),
            format!("Size: {size}"),
            format!("Must have: {}", join(&self.must_have, "; ", "none")),
            format!("Exclude: {}", join(&self.exclusions, "; ", "none")),
        ];
        if !self.notes.is_empty() {
            lines.push(format!("Notes: {}", self.notes));
        }
        lines.push(String::new());
        lines.push(format!("Request a pilot: {PILOT_URL}"));
        lines.join("\n")
    }
}

#[derive(Debug, Clone, Default)]
pub struct Company {
    pub id: String,
    pub mandate_fit: f64,
    pub outreach_suitability: f64,
    pub transition_context: f64,
    pub group_owned: bool,
}

/// Weighted composite on a 0 to 100 scale. A group-owned company scores zero on outreach suitability.
pub fn composite(c: &Company, w: &Weights) -> f64 {
    let outreach = if c.group_owned { 0.0 } else { c.outreach_suitability };
    let total = w.mandate_fit + w.outreach_suitability + w.transition_context;
    let v = (c.mandate_fit * w.mandate_fit + outreach * w.outreach_suitability + c.transition_context * w.transition_context) / total;
    (v * 10.0).round() / 10.0
}

/// Companies with their composite, highest first.
pub fn rank(companies: &[Company], w: &Weights) -> Vec<(Company, f64)> {
    let mut out: Vec<(Company, f64)> = companies.iter().map(|c| (c.clone(), composite(c, w))).collect();
    out.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    out
}

/// (visible, checked) over the 15 signals; `None` and "not visible" count as silent.
pub fn signal_coverage(signals: &[(&str, Option<&str>)]) -> (usize, usize) {
    let known: Vec<_> = signals.iter().filter(|(n, _)| SIGNALS.contains(n)).collect();
    let visible = known.iter().filter(|(_, v)| matches!(v, Some(s) if *s != "not visible")).count();
    (visible, known.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn group_owned_zeroes_outreach() {
        let full = Company { id: "a".into(), mandate_fit: 100.0, outreach_suitability: 100.0, transition_context: 100.0, group_owned: false };
        assert_eq!(composite(&full, &DEFAULT_WEIGHTS), 100.0);
        let owned = Company { group_owned: true, transition_context: 0.0, ..full };
        assert_eq!(composite(&owned, &DEFAULT_WEIGHTS), 70.0);
    }

    #[test]
    fn rank_and_coverage() {
        let a = Company { id: "a".into(), mandate_fit: 50.0, ..Default::default() };
        let b = Company { id: "b".into(), mandate_fit: 90.0, outreach_suitability: 50.0, ..Default::default() };
        assert_eq!(rank(&[a, b], &DEFAULT_WEIGHTS)[0].0.id, "b");
        assert_eq!(signal_coverage(&[(SIGNALS[0], Some("x")), (SIGNALS[1], None)]), (1, 2));
    }

    #[test]
    fn brief_text_has_pilot_link() {
        let b = ThesisBrief { name: "T".into(), vertical: "Water".into(), regions: vec!["US".into()], ..Default::default() };
        assert_eq!(b.validate().len(), 1);
        assert!(b.to_text().contains("free-pilot.php"));
    }
}
