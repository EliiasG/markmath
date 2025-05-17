use std::collections::HashMap;
use std::fmt::Display;
use std::str::FromStr;

pub struct UnitCollection {
    defined_units: HashMap<String, String>,
    /// op, unit_a, unit_b -> unit_res
    operator_results: HashMap<(String, String, String), String>,
}

impl UnitCollection {
    pub fn new() -> Self {
        Self {
            defined_units: HashMap::new(),
            operator_results: HashMap::new(),
        }
    }
}

impl Display for UnitCollection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = self
            .defined_units
            .iter()
            .map(|(a, b)| format!("{a};{b}"))
            .collect::<Vec<_>>()
            .join("\n")
            + "\n"
            + &self
                .operator_results
                .iter()
                .map(|((op, a, b), r)| format!("{a};{op};{b};{r}"))
                .collect::<Vec<_>>()
                .join("\n");
        write!(f, "{}", str)
    }
}

impl FromStr for UnitCollection {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut defined_units = HashMap::new();
        let mut operator_results = HashMap::new();
        let mut lines = s.lines();
        while let Some(line) = lines.next() {
            let line = line.trim();
            if line.is_empty() {
                break;
            }
            let [k, v] = line.split(';').collect::<Vec<&str>>()[..] else {
                return Err(format!("Invalid defined line: {}", line));
            };
            defined_units.insert(k.to_string(), v.to_string());
        }
        while let Some(line) = lines.next() {
            let line = line.trim();
            if line.is_empty() {
                break;
            }
            let [a, op, b, r] = line.split(';').collect::<Vec<&str>>()[..] else {
                return Err(format!("Invalid operator line: {}", line));
            };
            operator_results.insert(
                (op.to_string(), a.to_string(), b.to_string()),
                r.to_string(),
            );
        }
        Ok(Self {
            defined_units,
            operator_results,
        })
    }
}
