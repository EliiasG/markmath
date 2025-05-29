use crate::language::expression::DefinedUnit;
use crate::language::format::UnitLibrary;
use std::collections::{HashMap, HashSet};
use std::fmt::{format, Display};
use std::io::Write;
use std::mem;
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

    pub fn get_defined_unit(&self, name: &str) -> Option<&str> {
        self.defined_units.get(name).map(|x| x.as_str())
    }

    pub fn get_operator_result(
        &self,
        operator: String,
        a: String,
        b: String,
        associative: bool,
    ) -> Option<&str> {
        //FIXME use smarter map / hashing to avoid cloning and use &str...
        let k = (operator, a, b);
        if let Some(r) = self.operator_results.get(&k).map(|x| x.as_str()) {
            return Some(r);
        }
        if !associative {
            return None;
        }
        let k = (k.0, k.2, k.1);
        self.operator_results.get(&k).map(|x| x.as_str())
    }

    pub fn add_defined_unit(&mut self, name: String, unit: String) {
        self.defined_units.insert(name, unit);
    }

    pub fn add_operator_result(&mut self, operator: String, a: String, b: String, res: String) {
        self.operator_results.insert((operator, a, b), res);
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
            + "\n\n"
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

pub struct CLIUnitLib {
    collection: UnitCollection,
    cache: Vec<DefinedUnit>,
    interact: bool,
}

impl CLIUnitLib {
    pub fn new(collection: UnitCollection, interact: bool) -> Self {
        Self {
            collection,
            cache: Vec::new(),
            interact,
        }
    }

    pub fn finish(self) -> UnitCollection {
        self.collection
    }

    pub fn configure(&mut self) {
        loop {
            println!("\nUNIT CONFIG MODE");
            println!("[1]: Continue    [2]: List unit names    [3]: List operator results    [4]: Rename unit    [5]: Change operator result");
            match prompt("action: ", false).trim() {
                "1" => return,
                "2" => {
                    println!("Unit names:");
                    for (name, unit) in &self.collection.defined_units {
                        println!("{name}: \"{unit}\"");
                    }
                }
                "3" => {
                    println!("Operator results:");
                    for ((op, l, r), result) in &self.collection.operator_results {
                        println!("{l} {op} {r}: {result}");
                    }
                }
                "4" => {
                    let unit = prompt("Unit to rename: ", true);
                    if self.collection.get_defined_unit(&unit).is_none() {
                        println!("Unit does not exist: {unit}");
                        continue;
                    }
                    let name = prompt("Name: ", false);
                    self.collection.add_defined_unit(unit, name);
                }
                "5" => {
                    let op = prompt("Enter operator: ", false);
                    let l = prompt("Enter left unit: ", true);
                    let r = prompt("Enter right unit: ", true);
                    // FIXME shit
                    let assoc = "+*".contains(&op);
                    if self.collection.get_operator_result(op.clone(), l.clone(), r.clone(), assoc).is_none() {
                        println!("{l}{op}{r} does not exist");
                        continue;
                    }
                    let res = prompt("Enter resulting unit: ", true);
                    if self.collection.get_defined_unit(&res).is_none() {
                        let unit_name = prompt(&format!("Name unit {res}: "), false);
                        self.collection.add_defined_unit(res.clone(), unit_name);
                    }
                    self.collection.add_operator_result(op, l, r, res);
                }
                _ => {
                    println!("Please enter a number 1..5");
                    continue
                }
            }
        }
    }

    fn resolve_unit(&mut self, unit: DefinedUnit, missing_names: &mut HashSet<String>) -> String {
        match unit {
            DefinedUnit::Defined(name) => {
                if self.collection.get_defined_unit(&name).is_none() {
                    missing_names.insert(name.clone());
                }
                name
            }
            DefinedUnit::Implicit {
                operator,
                associative,
                left,
                right,
            } => {
                let l = self.resolve_unit(*left, missing_names);
                let r = self.resolve_unit(*right, missing_names);
                if let Some(res) = self.collection.get_operator_result(
                    operator.clone(),
                    l.clone(),
                    r.clone(),
                    associative,
                ) {
                    res.to_string()
                } else {
                    let msg = format!("Enter result of {l} {operator} {r}: ");
                    let res = prompt(&msg, true);
                    self.collection
                        .add_operator_result(operator, l, r, res.clone());
                    if self.collection.get_defined_unit(&res).is_none() {
                        missing_names.insert(res.clone());
                    }
                    res
                }
            }
        }
    }

    fn get_internal_unit<'a>(&'a self, unit: &'a DefinedUnit) -> &'a str {
        match unit {
            DefinedUnit::Defined(name) => name,
            DefinedUnit::Implicit {
                operator,
                associative,
                left,
                right,
            } => {
                let l = self.get_internal_unit(left).to_string();
                let r = self.get_internal_unit(right).to_string();
                self.collection
                    .get_operator_result(operator.clone(), l, r, *associative)
                    .unwrap()
            }
        }
    }
}

impl UnitLibrary for CLIUnitLib {
    fn cache_defined_unit(&mut self, unit: &DefinedUnit) {
        // could maybe be done smarter, but that would require some system of mapping undefined units...
        self.cache.push(unit.clone());
    }

    fn resolve_units(&mut self) {
        if !self.interact {
            return;
        }
        let mut missing = HashSet::new();
        for unit in mem::take(&mut self.cache) {
            self.resolve_unit(unit, &mut missing);
        }
        for m in missing {
            let msg = format!("Name unit {m}: ");
            self.collection.add_defined_unit(m, prompt(&msg, false));
        }
    }

    fn get_defined_unit(&self, unit: &DefinedUnit) -> String {
        if !self.interact {
            match unit {
                DefinedUnit::Defined(d) => d.clone(),
                DefinedUnit::Implicit { operator, left, right, .. } => format!("{} {} {}", self.get_defined_unit(left), operator, self.get_defined_unit(right)),
            }
        } else {
            self.collection
                .get_defined_unit(self.get_internal_unit(unit))
                .unwrap().to_string()
        }
    }
}

fn prompt(message: &str, name: bool) -> String {
    print!("{}", message);
    std::io::stdout().flush().unwrap();
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    input = input.trim().to_string();
    let starts_with_num = input.chars().next().map(char::is_numeric).unwrap_or(true);
    let valid_chars = input.chars().all(|c| c.is_alphanumeric() || c == '_');
    if name && (starts_with_num || !valid_chars) {
        println!(
            "please enter a valid name (must be alphanumeric + '_', and cannot start with digit)"
        );
        prompt(message, true)
    } else {
        input
    }
}
