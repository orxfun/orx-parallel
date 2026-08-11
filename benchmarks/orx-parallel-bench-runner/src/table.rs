use csv::Writer;
use std::fs::File;

#[derive(Debug)]
pub struct Table {
    input_factors: Vec<String>,
    rows: Vec<Vec<String>>,
}

impl Table {
    pub fn new(input_factors: Vec<String>) -> Self {
        let mut header = vec!["method".to_string(), "num_threads".to_string()];
        header.extend(input_factors.clone());
        header.extend(["time (ns)"].map(|x| x.to_string()));

        let rows = vec![header];
        Self {
            input_factors,
            rows,
        }
    }

    /// `output` is like `method:orx-once__size:small-10k__19087815`
    pub fn append(&mut self, outputs: String, threads: usize) {
        let output_rows: Vec<_> = outputs
            .trim()
            .split("\n")
            .map(|x| x.trim().to_string())
            .collect();

        for output in output_rows {
            let parts: Vec<String> = output.split("__").map(|x| x.to_string()).collect();

            let method = prop_value(&parts, "method");
            let time_ns = parts
                .last()
                .unwrap()
                .parse::<u64>()
                .expect("failed to parse time")
                .to_string();
            let input_values = self
                .input_factors
                .iter()
                .map(|prop| prop_value(&parts, prop));

            let mut row = vec![];
            row.push(method);
            row.push(threads.to_string());
            for value in input_values {
                row.push(value);
            }
            row.push(time_ns);

            self.rows.push(row);
        }
    }

    pub fn write_csv(&self, path: &str) {
        let file = File::create(path).expect("Failed to create CSV file");
        let mut wtr = Writer::from_writer(file);

        for row in &self.rows {
            wtr.write_record(row).expect("Failed to write CSV row");
        }

        wtr.flush().expect("Failed to flush CSV writer");
    }

    pub fn print(&self) {
        let mut table = comfy_table::Table::new();

        for row in &self.rows {
            table.add_row(row);
        }

        println!("\n{table}\n");
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
