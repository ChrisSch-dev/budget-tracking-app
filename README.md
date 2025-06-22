# Fintrack

**Fintrack** is a modern, modular, cross-platform finance and budget tracking desktop application written in Rust.  
It features a sleek GUI (built with `eframe`/`egui`), robust file persistence (JSON), CSV import/export, analytics, multi-currency support, monthly budgets, and more.

## Features

- 📊 **Modern Native GUI** (eframe/egui)
- 💸 **Add, edit, and delete transactions (with per-transaction currency)**
- 🌎 **Multi-currency support** (per-transaction currency, base currency selection, automatic conversion in analytics and totals)
- 💱 **Editable exchange rates** (edit in GUI; fetch live rates from API)
- 🗂 **Categories, search/filter, and recurring expenses**
- 🏦 **Multi-currency monthly budget limits and progress tracking per category**
- 📈 **Analytics: Pie charts for category spending (in base or original currency)**
- 💾 **Save/load profiles (JSON)**
- 🗃 **CSV import/export (with currency support)**
- 🌗 **Light/dark theme toggle**
- 🎯 **Cross-platform:** Windows, macOS, Linux

## Folder Structure

```
fintrack/
├── Cargo.toml
└── src/
    ├── main.rs           # Entry point
    ├── app.rs            # App struct and core logic
    ├── gui.rs            # All GUI rendering (with pie chart, currency toggles, exchange editing)
    ├── data.rs           # Persistence, currency conversion, exchange rates (with API fetching), CSV import/export
    ├── analytics.rs      # Filtering, stats, charts (totals in base and other currencies)
    ├── utils.rs          # Theme and helpers
    └── types.rs          # Data types, state, Currency enum
```

## Multi-Currency Support

- Each transaction records its own currency.
- The application maintains a **base currency** (user-selectable, e.g. USD/EUR/GBP/JPY/CHF).
- All totals, analytics, and charts are shown in this base currency, using exchange rates (editable in GUI & fetchable from API).
- Per-category budgets can be set in any currency; analytics display conversions to base currency.
- When adding a transaction, select the appropriate currency from a dropdown.

## Editable Exchange Rates

- Click "Edit Exchange Rates" in the top bar to open the rate editor.
- Edit any rate, or click "Update from API" for live rates.

## Multi-currency CSV Import/Export

- CSV import/export supports a `currency` column for each transaction.

## Multi-currency Charts

- Pie chart analytics can be toggled between base currency (converted) and original transaction currencies.

## Usage

### Prerequisites

- [Rust](https://rustup.rs/) (stable toolchain)
- Add the following dependencies to `Cargo.toml` if not present:
    - eframe
    - egui_extras
    - serde, serde_json
    - chrono
    - rfd
    - csv
    - reqwest = { version = "0.12", features = ["blocking", "json"] }

### Run

```
cargo run
```

### Build

```
cargo build --release
```

### Binary will be found in `target/release/fintrack`

## License

MIT

---

Made with ❤️ in Rust.