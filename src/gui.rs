use crate::types::*;
use eframe::epi;
use egui_extras::{Column, TableBuilder};
use chrono::Local;
use std::collections::BTreeMap;

pub fn draw_main_window(app: &mut crate::app::BudgetApp, ctx: &egui::Context, _frame: &epi::Frame) {
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
                    match state.import_csv(&path) {
                        Ok(_) => {},
                        Err(e) => { /* TODO: Show error in GUI */ }
                    }
                }
            }
            if ui.button("Export CSV").clicked() {
                if let Some(path) = rfd::FileDialog::new().save_file() {
                    match state.export_csv(&path) {
                        Ok(_) => {},
                        Err(e) => { /* TODO: Show error in GUI */ }
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

    // Exchange Rates Modal
    if state.editing_rates {
        egui::Window::new("Exchange Rates")
            .open(&mut state.editing_rates)
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
                    state.fetch_exchange_rates();
                }
                if let Some(err) = &state.rates_api_error {
                    ui.colored_label(egui::Color32::RED, err);
                }
            });
    }

    egui::SidePanel::left("side").show(ctx, |ui| {
        ui.heading("Add Transaction");
        ui.horizontal(|ui| {
            ui.label("Date");
            ui.add(egui::widgets::DatePickerButton::new(&mut state.input_date));
        });
        ui.text_edit_singleline(&mut state.input_desc).hint_text("Description");
        ui.text_edit_singleline(&mut state.input_amt).hint_text("Amount");
        ui.text_edit_singleline(&mut state.input_cat).hint_text("Category");
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
                let transaction = Transaction {
                    date: state.input_date,
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
                state.input_date = Local::now().date_naive();
                state.input_recurring = false;
                state.save();
            }
        }
        if let Some(idx) = state.selected_tx {
            if ui.button("Delete Selected").clicked() {
                state.data.transactions.remove(idx);
                state.selected_tx = None;
                state.save();
            }
        }
        ui.separator();
        ui.heading("Budgets");
        // Multi-currency per-category budgets
        for cat in state.categories() {
            let cat_budget = state.data.budget.monthly_limits.entry(cat.clone())
                .or_insert(CategoryBudget { amount: 0.0, currency: state.base_currency });
            ui.horizontal(|ui| {
                ui.label(&cat);
                ui.add(egui::DragValue::new(&mut cat_budget.amount));
                egui::ComboBox::from_id_source(format!("budget_curr_{}", cat))
                    .selected_text(cat_budget.currency.as_str())
                    .show_ui(ui, |ui| {
                        for &c in Currency::all() {
                            ui.selectable_value(&mut cat_budget.currency, c, c.as_str());
                        }
                    });
                if ui.button("Set").clicked() {
                    state.save();
                }
                // Show budget in base currency
                let converted = state.convert(cat_budget.amount, cat_budget.currency, state.base_currency);
                if cat_budget.currency != state.base_currency {
                    ui.label(format!(
                        "≈ {:.2} {}",
                        converted, state.base_currency
                    ));
                }
            });
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

        // Multi-currency pie chart: toggle for original or base currency
        static mut SHOW_BASE: bool = true;
        let mut show_base = unsafe { SHOW_BASE };
        ui.horizontal(|ui| {
            ui.label("Chart currency:");
            ui.radio_value(&mut show_base, true, format!("{}", state.base_currency));
            ui.radio_value(&mut show_base, false, "Original");
        });
        unsafe { SHOW_BASE = show_base; }
        use egui::plot::{PieChart, PieSlice, Plot};
        let mut slices = Vec::new();
        if show_base {
            let cat_sums = state.category_sums_this_month();
            for (cat, sum) in &cat_sums {
                slices.push(PieSlice::new(cat.clone(), *sum));
            }
        } else {
            let mut sums: BTreeMap<(String, Currency), f64> = Default::default();
            for tx in state.filtered_transactions() {
                let cat = tx.category.clone();
                if tx.date.year() == Local::now().year() && tx.date.month() == Local::now().month() {
                    *sums.entry((cat, tx.currency)).or_insert(0.0) += tx.amount;
                }
            }
            for ((cat, curr), sum) in sums {
                slices.push(PieSlice::new(format!("{cat} ({curr})"), sum));
            }
        }
        if !slices.is_empty() {
            let pie = PieChart::new(slices).radius(80.0);
            Plot::new("categories_pie").show(ui, |plot_ui| {
                plot_ui.pie_chart(pie);
            });
        } else {
            ui.label("No data to display for this month.");
        }
    });

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading(format!(
            "Transactions (converted to base: {})",
            state.base_currency
        ));

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
                for (i, tx) in state.filtered_transactions().into_iter().enumerate() {
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
                                state.input_date = tx.date;
                                state.input_recurring = tx.recurring;
                                state.input_currency = tx.currency;
                            }
                        });
                    });
                }
            });
    });
}