use crate::types::*;
use chrono::{Datelike, Local};
use std::collections::HashMap;

impl AppState {
    pub fn filtered_transactions(&self) -> Vec<&Transaction> {
        self.data.transactions.iter().filter(|tx| {
            self.search_term.is_empty() ||
                tx.description.to_lowercase().contains(&self.search_term.to_lowercase()) ||
                tx.category.to_lowercase().contains(&self.search_term.to_lowercase())
        }).collect()
    }

    pub fn total(&self) -> f64 {
        self.filtered_transactions()
            .iter()
            .map(|t| self.convert(t.amount, t.currency, self.base_currency))
            .sum()
    }

    /// Sums per category for current month, in base currency
    pub fn category_sums_this_month(&self) -> HashMap<String, f64> {
        let now = Local::now().naive_local();
        let mut sums = HashMap::new();
        for tx in &self.data.transactions {
            if tx.date.year() == now.year() && tx.date.month() == now.month() {
                let converted = self.convert(tx.amount, tx.currency, self.base_currency);
                *sums.entry(tx.category.clone()).or_insert(0.0) += converted;
            }
        }
        sums
    }

    pub fn categories(&self) -> Vec<String> {
        let mut cats: Vec<String> = self.data.transactions.iter().map(|t| t.category.clone()).collect();
        cats.sort();
        cats.dedup();
        cats
    }
}