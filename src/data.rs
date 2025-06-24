use crate::types::*;
use std::fs;
use std::path::PathBuf;
use chrono::{Local, NaiveDate};
use csv::{ReaderBuilder, WriterBuilder};
use std::collections::HashMap;
use std::str::FromStr; // <----- ADD THIS LINE

pub fn fetch_exchange_rates_api(base: Currency, supported: &[Currency]) -> Result<HashMap<(Currency, Currency), f64>, String> {
    let base = base.as_str();
    let symbols: Vec<String> = supported.iter().map(|c| c.as_str().to_owned()).collect();
    let url = format!(
        "https://api.exchangerate.host/latest?base={}&symbols={}",
        base,
        symbols.join(",")
    );

    let resp = reqwest::blocking::get(&url).map_err(|e| format!("API error: {e}"))?;
    let json: serde_json::Value = resp.json().map_err(|e| format!("Invalid API JSON: {e}"))?;
    let mut rates = HashMap::new();

    if let Some(rates_json) = json.get("rates") {
        for c in supported {
            if let Some(rate) = rates_json.get(c.as_str()).and_then(|v| v.as_f64()) {
                rates.insert((Currency::from_str(base).unwrap_or(Currency::USD), *c), rate);
                rates.insert((*c, Currency::from_str(base).unwrap_or(Currency::USD)), 1.0 / rate);
            }
        }
    }
    Ok(rates)
}

impl AppState {
    pub fn load_or_default(file_path: Option<PathBuf>) -> Self {
        let data = if let Some(path) = &file_path {
            if let Ok(content) = fs::read_to_string(path) {
                serde_json::from_str(&content).unwrap_or_default()
            } else {
                BudgetAppData::default()
            }
        } else {
            BudgetAppData::default()
        };

        let mut exchange_rates = HashMap::new();
        // Example rates
        exchange_rates.insert((Currency::USD, Currency::USD), 1.0);
        exchange_rates.insert((Currency::EUR, Currency::USD), 1.1);
        exchange_rates.insert((Currency::USD, Currency::EUR), 0.91);
        exchange_rates.insert((Currency::GBP, Currency::USD), 1.25);
        exchange_rates.insert((Currency::USD, Currency::GBP), 0.8);
        exchange_rates.insert((Currency::JPY, Currency::USD), 0.0068);
        exchange_rates.insert((Currency::USD, Currency::JPY), 147.0);
        exchange_rates.insert((Currency::CHF, Currency::USD), 1.13);
        exchange_rates.insert((Currency::USD, Currency::CHF), 0.88);

        Self {
            data,
            input_desc: String::new(),
            input_amt: String::new(),
            input_cat: String::new(),
            input_date_str: Local::now().date_naive().to_string(),
            input_recurring: false,
            input_currency: Currency::USD,
            search_term: String::new(),
            file_path,
            selected_tx: None,
            theme: Theme::Light,
            show_import_modal: false,
            import_path: None,
            base_currency: Currency::USD,
            exchange_rates,
            editing_rates: false,
            rates_api_error: None,
        }
    }

    pub fn save(&self) {
        if let Some(path) = &self.file_path {
            if let Ok(json) = serde_json::to_string_pretty(&self.data) {
                let _ = fs::write(path, json);
            }
        }
    }

    pub fn load(&mut self, file_path: PathBuf) {
        self.file_path = Some(file_path.clone());
        if let Ok(content) = fs::read_to_string(&file_path) {
            self.data = serde_json::from_str(&content).unwrap_or_default();
        }
    }

    pub fn convert(&self, amount: f64, from: Currency, to: Currency) -> f64 {
        if from == to {
            amount
        } else if let Some(rate) = self.exchange_rates.get(&(from, to)) {
            amount * rate
        } else {
            amount
        }
    }

    pub fn fetch_exchange_rates(&mut self) {
        match fetch_exchange_rates_api(self.base_currency, Currency::all()) {
            Ok(rates) => {
                for ((f, t), v) in rates {
                    self.exchange_rates.insert((f, t), v);
                }
                self.rates_api_error = None;
            }
            Err(e) => {
                self.rates_api_error = Some(e);
            }
        }
    }

    pub fn export_csv(&self, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let mut wtr = WriterBuilder::new().from_path(path)?;
        wtr.write_record(&["date", "description", "amount", "currency", "category", "recurring"])?;
        for tx in &self.data.transactions {
            wtr.write_record(&[
                tx.date.to_string(),
                tx.description.clone(),
                tx.amount.to_string(),
                tx.currency.as_str().to_string(),
                tx.category.clone(),
                tx.recurring.to_string()
            ])?;
        }
        wtr.flush()?;
        Ok(())
    }

    pub fn import_csv(&mut self, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let mut rdr = ReaderBuilder::new().from_path(path)?;
        for result in rdr.records() {
            let record = result?;
            let date = chrono::NaiveDate::parse_from_str(&record[0], "%Y-%m-%d")?;
            let description = record[1].to_string();
            let amount: f64 = record[2].parse()?;
            let currency = Currency::all().iter().find(|c| c.as_str() == &record[3]).copied().unwrap_or(Currency::USD);
            let category = record[4].to_string();
            let recurring: bool = record[5].parse()?;
            self.data.transactions.push(Transaction {
                date, description, amount, category, recurring, currency
            });
        }
        Ok(())
    }
}