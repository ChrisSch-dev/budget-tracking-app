use chrono::NaiveDate;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Currency {
    USD,
    EUR,
    GBP,
    JPY,
    CHF,
}

impl Currency {
    pub fn as_str(&self) -> &'static str {
        match self {
            Currency::USD => "USD",
            Currency::EUR => "EUR",
            Currency::GBP => "GBP",
            Currency::JPY => "JPY",
            Currency::CHF => "CHF",
        }
    }
    pub fn all() -> &'static [Currency] {
        &[Currency::USD, Currency::EUR, Currency::GBP, Currency::JPY, Currency::CHF]
    }
}

impl std::fmt::Display for Currency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Default for Currency {
    fn default() -> Self {
        Currency::USD
    }
}

impl FromStr for Currency {
    type Err = ();

    fn from_str(s: &str) -> Result<Currency, ()> {
        match s {
            "USD" => Ok(Currency::USD),
            "EUR" => Ok(Currency::EUR),
            "GBP" => Ok(Currency::GBP),
            "JPY" => Ok(Currency::JPY),
            "CHF" => Ok(Currency::CHF),
            _ => Err(()),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Transaction {
    pub date: NaiveDate,
    pub description: String,
    pub amount: f64,
    pub category: String,
    pub recurring: bool,
    pub currency: Currency,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CategoryBudget {
    pub amount: f64,
    pub currency: Currency,
}

#[derive(Serialize, Deserialize, Default)]
pub struct Budget {
    pub monthly_limits: HashMap<String, CategoryBudget>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct BudgetAppData {
    pub transactions: Vec<Transaction>,
    pub budget: Budget,
    pub last_profile: Option<String>,
}

#[derive(PartialEq)]
pub enum Theme {
    Light,
    Dark,
}

pub struct AppState {
    pub data: BudgetAppData,
    pub input_desc: String,
    pub input_amt: String,
    pub input_cat: String,
    pub input_date_str: String, // changed from NaiveDate to user-editable string
    pub input_recurring: bool,
    pub input_currency: Currency,
    pub search_term: String,
    pub file_path: Option<std::path::PathBuf>,
    pub selected_tx: Option<usize>,
    pub theme: Theme,
    pub show_import_modal: bool,
    pub import_path: Option<std::path::PathBuf>,
    pub base_currency: Currency,
    pub exchange_rates: HashMap<(Currency, Currency), f64>,
    pub editing_rates: bool,
    pub rates_api_error: Option<String>,
}