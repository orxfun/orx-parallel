#[derive(Debug)]
pub struct Table {
    input_factors: Vec<String>,
    rows: Vec<Vec<String>>,
}

impl Table {
    pub fn new(input_factors: Vec<String>) -> Self {
        let mut header = input_factors.clone();
        header.extend(["method", "num_threads", "time (ns)"].map(|x| x.to_string()));

        let rows = vec![header];
        Self {
            input_factors,
            rows,
        }
    }

    /// `output` is like `method:orx-once__size:small-10k__19087815`
    pub fn append(&mut self, output: String) {
        let parts: Vec<String> = output.split("__").map(|x| x.to_string()).collect();

        let method = prop_value(&parts, "method");
        let input_values = self
            .input_factors
            .iter()
            .map(|prop| prop_value(&parts, prop));

        let mut row = vec![];
        row.push(method);
        for value in input_values {
            row.push(value);
        }

        self.rows.push(row);
    }
}

fn prop_value(parts: &[String], prop: &str) -> String {
    parts
        .iter()
        .filter(|x| x.starts_with(&format!("{prop}:")))
        .map(|x| x.split(":").last().unwrap().to_string())
        .next()
        .unwrap()
}
