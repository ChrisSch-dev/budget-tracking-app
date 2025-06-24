use crate::types::*;
use eframe::egui;
use egui_extras::{Column, TableBuilder};
use chrono::Local;

pub fn draw_main_window(app: &mut crate::app::BudgetApp, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    let state = &mut app.state;
    state.set_theme(ctx);

    egui::TopBottomPanel::top("menu").show(ctx, |ui| {
        ui.horizontal(|ui| {
            if ui.button("New").clicked() {
                state.data = crate::types::BudgetAppData::default();
                state.file_path = None;
            }
            if ui.button("Save As...").clicked() {
                if let Some(path) = rfd::FileDialog::new().save_file() {
                    state.file_path = Some(path.clone());
                    state.save();
                }
            }
            if ui.button("Load...").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    state.load(path);
                }
            }
            if ui.button("Import CSV").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    // TODO: Show error in GUI
                    match state.import_csv(&path) {
                        Ok(_) => {
                            state.rates_api_error = Some("CSV imported successfully.".to_string());
                        },
                        Err(e) => {
                            state.rates_api_error = Some(format!("CSV import failed: {e}"));
                        }
                    }
                }
            }
            if ui.button("Export CSV").clicked() {
                if let Some(path) = rfd::FileDialog::new().save_file() {
                    // TODO: Show error in GUI
                    match state.export_csv(&path) {
                        Ok(_) => {
                            state.rates_api_error = Some("CSV exported successfully.".to_string());
                        },
                        Err(e) => {
                            state.rates_api_error = Some(format!("CSV export failed: {e}"));
                        }
                    }
                }
            }
            if ui.button("Edit Exchange Rates").clicked() {
                state.editing_rates = true;
            }
            if ui.button("Theme").clicked() {
                state.theme = if state.theme == Theme::Light { Theme::Dark } else { Theme::Light };
            }
            ui.label("Base currency:");
            egui::ComboBox::from_id_source("base_currency")
                .selected_text(state.base_currency.as_str())
                .show_ui(ui, |ui| {
                    for c in Currency::all() {
                        ui.selectable_value(&mut state.base_currency, *c, c.as_str());
                    }
                });
            ui.label(format!(
                "Profile: {}",
                state.file_path
                    .as_ref()
                    .and_then(|p| p.file_name())
                    .and_then(|n| n.to_str())
                    .unwrap_or("None")
            ));
        });
    });

    // Exchange Rates Modal: snapshot editing_rates before window, update after
    let mut editing_rates = state.editing_rates;
    if editing_rates {
        egui::Window::new("Exchange Rates")
            .open(&mut editing_rates)
            .show(ctx, |ui| {
                ui.label("Edit exchange rates relative to base currency");
                for &from in Currency::all() {
                    for &to in Currency::all() {
                        if from != to {
                            let key = (from, to);
                            let mut val = *state.exchange_rates.get(&key).unwrap_or(&1.0);
                            ui.horizontal(|ui| {
                                ui.label(format!("{} -> {}", from.as_str(), to.as_str()));
                                if ui.add(egui::DragValue::new(&mut val).speed(0.001)).changed() {
                                    state.exchange_rates.insert(key, val);
                                }
                            });
                        }
                    }
                }
                if ui.button("Update from API").clicked() {
                    // Schedule update after window closes to avoid borrow error
                    state.rates_api_error = Some("fetch".to_owned());
                }
                if let Some(err) = &state.rates_api_error {
                    if err != "fetch" { // don't show fake error
                        ui.colored_label(egui::Color32::RED, err);
                    }
                }
            });
    }
    state.editing_rates = editing_rates;
    if state.rates_api_error.as_deref() == Some("fetch") {
        state.fetch_exchange_rates();
        state.rates_api_error = Some("Exchange rates updated from API.".to_string());
    }

    egui::SidePanel::left("side").show(ctx, |ui| {
        ui.heading("Add Transaction");
        ui.label("Date (YYYY-MM-DD):");
        ui.text_edit_singleline(&mut state.input_date_str);
        ui.label("Description:");
        ui.text_edit_singleline(&mut state.input_desc);
        ui.label("Amount:");
        ui.text_edit_singleline(&mut state.input_amt);
        ui.label("Category:");
        ui.text_edit_singleline(&mut state.input_cat);
        egui::ComboBox::from_id_source("input_currency")
            .selected_text(state.input_currency.as_str())
            .show_ui(ui, |ui| {
                for c in Currency::all() {
                    ui.selectable_value(&mut state.input_currency, *c, c.as_str());
                }
            });
        ui.checkbox(&mut state.input_recurring, "Recurring");
        if ui.button("Add").clicked() {
            if let Ok(amount) = state.input_amt.parse::<f64>() {
                if let Ok(date) = chrono::NaiveDate::parse_from_str(&state.input_date_str, "%Y-%m-%d") {
                    let transaction = Transaction {
                        date,
                        description: state.input_desc.clone(),
                        amount,
                        category: state.input_cat.clone(),
                        recurring: state.input_recurring,
                        currency: state.input_currency,
                    };
                    state.data.transactions.push(transaction);
                    state.input_desc.clear();
                    state.input_amt.clear();
                    state.input_cat.clear();
                    state.input_date_str = Local::now().date_naive().to_string();
                    state.input_recurring = false;
                    state.save();
                    // Show a status message for success
                    state.rates_api_error = Some("Transaction added.".to_string());
                } else {
                    // Show a status message for date parse failure
                    state.rates_api_error = Some("Failed to parse date. Use YYYY-MM-DD.".to_string());
                }
            } else {
                // Show a status message for amount parse failure
                state.rates_api_error = Some("Failed to parse amount (must be a number).".to_string());
            }
        }
        if let Some(idx) = state.selected_tx {
            if ui.button("Delete Selected").clicked() {
                state.data.transactions.remove(idx);
                state.selected_tx = None;
                state.save();
                // Show a status message for delete
                state.rates_api_error = Some("Transaction deleted.".to_string());
            }
        }
        ui.separator();
        ui.heading("Budgets");

        // FIX: Avoid borrow checker error by operating on copies and writing back if changed.
        let categories = state.categories();
        for cat in categories {
            // Get current values
            let (mut amount, mut currency) = {
                let entry = state.data.budget.monthly_limits
                    .get(&cat)
                    .cloned()
                    .unwrap_or(CategoryBudget { amount: 0.0, currency: state.base_currency });
                (entry.amount, entry.currency)
            };

            let mut changed = false;
            ui.horizontal(|ui| {
                ui.label(&cat);
                changed |= ui.add(egui::DragValue::new(&mut amount)).changed();
                egui::ComboBox::from_id_source(format!("budget_curr_{}", cat))
                    .selected_text(currency.as_str())
                    .show_ui(ui, |ui| {
                        for &c in Currency::all() {
                            if ui.selectable_value(&mut currency, c, c.as_str()).changed() {
                                changed = true;
                            }
                        }
                    });
                if ui.button("Set").clicked() || changed {
                    state.data.budget.monthly_limits.insert(cat.clone(), CategoryBudget { amount, currency });
                    state.save();
                    // Show a status message for budget update
                    state.rates_api_error = Some(format!("Budget updated for category '{}'.", cat));
                }
                let converted = state.convert(amount, currency, state.base_currency);
                if currency != state.base_currency {
                    ui.label(format!("≈ {:.2} {}", converted, state.base_currency));
                }
            });
        }
        // Show status/error message (used for TODOs above)
        if let Some(msg) = &state.rates_api_error {
            ui.separator();
            ui.colored_label(egui::Color32::LIGHT_BLUE, msg);
        }
    });

    egui::TopBottomPanel::bottom("stats").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.label("Search:");
            ui.text_edit_singleline(&mut state.search_term);
            ui.label(format!(
                "Total: {:.2} {}",
                state.total(),
                state.base_currency
            ));
        });
    });

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading(format!(
            "Transactions (converted to base: {})",
            state.base_currency
        ));

        // Borrow checker fix: collect filtered indices first
        let filtered_indices: Vec<usize> = state.data.transactions.iter().enumerate()
            .filter(|(_, tx)| {
                state.search_term.is_empty() ||
                    tx.description.to_lowercase().contains(&state.search_term.to_lowercase()) ||
                    tx.category.to_lowercase().contains(&state.search_term.to_lowercase())
            })
            .map(|(i, _)| i)
            .collect();

        TableBuilder::new(ui)
            .striped(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .columns(Column::auto(), 7)
            .header(20.0, |mut header| {
                header.col(|ui| { ui.label("Date"); });
                header.col(|ui| { ui.label("Description"); });
                header.col(|ui| { ui.label("Amount"); });
                header.col(|ui| { ui.label("Currency"); });
                header.col(|ui| { ui.label("Category"); });
                header.col(|ui| { ui.label("Recurring"); });
                header.col(|ui| { ui.label("Select"); });
            })
            .body(|mut body| {
                for &i in &filtered_indices {
                    let tx = &state.data.transactions[i];
                    body.row(18.0, |mut row| {
                        row.col(|ui| { ui.label(tx.date.to_string()); });
                        row.col(|ui| { ui.label(&tx.description); });
                        row.col(|ui| {
                            ui.label(format!("{:.2}", tx.amount));
                        });
                        row.col(|ui| { ui.label(tx.currency.as_str()); });
                        row.col(|ui| { ui.label(&tx.category); });
                        row.col(|ui| { if tx.recurring { ui.label("Yes"); } else { ui.label("No"); } });
                        row.col(|ui| {
                            if ui.button("Select").clicked() {
                                state.selected_tx = Some(i);
                                state.input_desc = tx.description.clone();
                                state.input_amt = format!("{}", tx.amount);
                                state.input_cat = tx.category.clone();
                                state.input_date_str = tx.date.to_string();
                                state.input_recurring = tx.recurring;
                                state.input_currency = tx.currency;
                            }
                        });
                    });
                }
            });
    });
}