use color_eyre::Result;
use core::fmt;
use csv::{ReaderBuilder, WriterBuilder};
use eyre::bail;
use itertools::Itertools;
use ratatui::{
    text::Text,
    widgets::{Cell, Row},
};
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    fmt::Display,
    fs,
    path::PathBuf,
    slice::Iter,
    str::FromStr,
};
use time::{Date, Month};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct SimpleDate {
    pub year: i32,
    pub month: u8,
    pub day: u8,
    date: Date,
}

impl TryFrom<&str> for SimpleDate {
    type Error = String;

    fn try_from(date_string: &str) -> Result<SimpleDate, Self::Error> {
        let splitter = if date_string.contains('-') { "-" } else { "." };
        let mut parts = date_string.split(splitter);

        let year: i32 = parts
            .next()
            .ok_or_else(|| format!("Missing year in {}", date_string))
            .and_then(|y| y.parse::<i32>().map_err(|_| format!("Invalid year: {}", y)))
            .map(|y| if y < 100 { 2000 + y } else { y })?;

        let month: u8 = parts
            .next()
            .ok_or_else(|| format!("Missing month in {}", date_string))
            .and_then(|m| m.parse::<u8>().map_err(|_| format!("Invalid month: {}", m)))?;

        let monthy_month: Month =
            Month::try_from(month).map_err(|_| format!("Invalid month: {}", month))?;

        let day: u8 = parts
            .next()
            .ok_or_else(|| format!("Missing day in {}", date_string))
            .and_then(|d| d.parse::<u8>().map_err(|_| format!("Invalid day: {}", d)))?;

        let date = Date::from_calendar_date(year, monthy_month, day)
            .map_err(|_| format!("Invalid date: {}", date_string))?;

        Ok(SimpleDate {
            year,
            month,
            day,
            date,
        })
    }
}

impl Display for SimpleDate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:04}.{:02}.{:02}", self.year, self.month, self.day)
    }
}

impl Serialize for SimpleDate {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("{}", self))
    }
}

impl<'de> Deserialize<'de> for SimpleDate {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        SimpleDate::try_from(s.as_str()).map_err(serde::de::Error::custom)
    }
}

impl PartialOrd for SimpleDate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.date.cmp(&other.date))
    }
}

impl Ord for SimpleDate {
    fn cmp(&self, other: &Self) -> Ordering {
        self.date.cmp(&other.date)
    }
}

#[derive(Debug, Clone)]
pub enum TransactionField {
    Date,
    Amount,
    Details,
    Category,
    Method,
    Direction,
    Currency,
}

impl TransactionField {
    pub fn all_fields() -> Vec<Self> {
        vec![
            Self::Date,
            Self::Amount,
            Self::Details,
            Self::Category,
            Self::Method,
            Self::Direction,
            Self::Currency,
        ]
    }

    pub fn get(index: usize) -> Option<Self> {
        match index {
            0 => Some(Self::Date),
            1 => Some(Self::Amount),
            2 => Some(Self::Details),
            3 => Some(Self::Category),
            4 => Some(Self::Method),
            5 => Some(Self::Direction),
            6 => Some(Self::Currency),
            _ => None,
        }
    }

    pub fn widths() -> Vec<u16> {
        vec![11, 10, 100, 15, 9, 9, 9]
    }

    pub fn names() -> Vec<String> {
        vec![
            "Date",
            "Amount",
            "Details",
            "Category",
            "Method",
            "Direction",
            "Currency",
        ]
        .into_iter()
        .map(|s| s.to_string())
        .collect()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    pub date: SimpleDate,
    amount: f64,
    pub details: String,
    pub category: String,
    method: String,
    pub direction: String,
    currency: String,
}

impl Transaction {
    pub fn new(date: SimpleDate) -> Self {
        Transaction {
            date,
            amount: 0.0,
            details: "".to_string(),
            category: "".to_string(),
            method: "".to_string(),
            direction: "".to_string(),
            currency: "".to_string(),
        }
    }

    pub fn mutate_field(&mut self, field_index: usize, input: &str) -> Result<(), String> {
        match TransactionField::get(field_index) {
            Some(field) => match field {
                TransactionField::Date => match SimpleDate::try_from(input) {
                    Ok(date) => self.date = date,
                    Err(e) => return Err(format!(" failed to parse as date: {}", e)),
                },
                TransactionField::Amount => match f64::from_str(input) {
                    Ok(num) => self.amount = num,
                    Err(e) => return Err(format!(" failed to parse as number: {}", e)),
                },
                TransactionField::Details => self.details = input.to_string(),
                TransactionField::Category => self.category = input.to_string(),
                TransactionField::Method => self.method = input.to_string(),
                TransactionField::Direction => self.direction = input.to_string(),
                TransactionField::Currency => self.currency = input.to_string(),
            },
            None => {}
        }
        Ok(())
    }

    fn get_field_text(&self, field: &TransactionField) -> String {
        match field {
            TransactionField::Date => format!("{}", self.date),
            TransactionField::Amount => {
                if self.amount == 0.0 {
                    "".to_string()
                } else {
                    format!("{:.2}", self.amount)
                }
            }
            TransactionField::Details => self.details.clone(),
            TransactionField::Category => self.category.clone(),
            TransactionField::Method => self.method.clone(),
            TransactionField::Direction => self.direction.clone(),
            TransactionField::Currency => self.currency.clone(),
        }
    }

    fn get_column_text(&self, field_index: usize) -> Option<String> {
        TransactionField::get(field_index).map(|field| self.get_field_text(&field))
    }

    pub fn generate_row(&self) -> Row {
        let cells: Vec<Cell> = TransactionField::all_fields()
            .into_iter()
            .map(|field| self.get_field_text(&field))
            .map(|text| Cell::from(Text::from(format!("\n{}\n", text))))
            .collect();
        Row::new(cells)
    }
}

impl PartialEq for Transaction {
    fn eq(&self, other: &Self) -> bool {
        let date_cmp = self.date == other.date;
        let amount_cmp = (self.amount - other.amount).abs() < 1e-6;
        let currency_cmp = self.currency == other.currency;
        let details_cmp = self.details == other.details;
        let category_cmp = self.category == other.category;
        let method_cmp = self.method == other.method;
        date_cmp && amount_cmp && currency_cmp && details_cmp && category_cmp && method_cmp
    }
}

impl Eq for Transaction {}

impl PartialOrd for Transaction {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.date.cmp(&other.date))
    }
}

impl Ord for Transaction {
    fn cmp(&self, other: &Self) -> Ordering {
        self.date.cmp(&other.date)
    }
}

pub enum FileType {
    Json,
    Csv,
    Unknown,
}

impl FileType {
    pub fn new(file_path: &PathBuf) -> FileType {
        let extension = file_path.extension().map(|ext| ext.to_str()).flatten();
        match extension {
            Some("json") => FileType::Json,
            Some("csv") => FileType::Csv,
            Some(_) => FileType::Unknown,
            None => FileType::Unknown,
        }
    }
}

pub type MonthInYear = (
    i32, // year
    u8,  //month
);

pub type DirectionAndCategory = (Option<String>, Option<String>);
type SummaryMap = HashMap<(DirectionAndCategory, MonthInYear), f64>;

const DEFAULT_CURRENCY: &str = "ILS";

pub struct TransactionsReport {
    pub months: Vec<MonthInYear>,
    categories: Vec<DirectionAndCategory>,
    category_summary: SummaryMap,
}

impl TransactionsReport {
    pub fn new(transactions: &Vec<Transaction>) -> Self {
        let mut months: HashSet<MonthInYear> = HashSet::new();
        let mut categories: HashSet<DirectionAndCategory> = HashSet::new();
        let mut category_summary: SummaryMap = HashMap::default();
        transactions.iter().for_each(|transaction| {
            if transaction.currency == DEFAULT_CURRENCY {
                let month_in_year = (transaction.date.year, transaction.date.month);
                months.insert(month_in_year);
                categories.insert((
                    Some(transaction.direction.clone()),
                    Some(transaction.category.clone()),
                ));
                categories.insert((Some(transaction.direction.clone()), None));
                *category_summary
                    .entry((
                        (
                            Some(transaction.direction.clone()),
                            Some(transaction.category.clone()),
                        ),
                        month_in_year,
                    ))
                    .or_insert(0.0) += transaction.amount;
                *category_summary
                    .entry(((Some(transaction.direction.clone()), None), month_in_year))
                    .or_insert(0.0) += transaction.amount;
            }
        });
        let months: Vec<MonthInYear> = months.into_iter().sorted().rev().collect();
        let categories: Vec<DirectionAndCategory> = categories.into_iter().sorted().collect();
        TransactionsReport {
            months,
            categories,
            category_summary,
        }
    }

    pub fn rows_len(&self) -> usize {
        self.months.len()
    }

    pub fn get_month_rows(
        &self,
        direction_and_category: &DirectionAndCategory,
    ) -> Vec<(String, f64)> {
        self.months
            .iter()
            .map(|month| {
                let month_str = format!("\n{:04}.{:02}", month.0, month.1);
                let category_amount: f64 = *self
                    .category_summary
                    .get(&(direction_and_category.clone(), *month))
                    .unwrap_or(&0.0);
                (month_str, category_amount)
            })
            .collect()
    }

    pub fn get_month_at_index(&self, index: usize) -> Option<&MonthInYear> {
        self.months.get(index)
    }

    /** Returns a vector of categories that have a non-0 value for the given month */
    pub fn get_categories_for_month_by_index(&self, index: usize) -> Vec<DirectionAndCategory> {
        if let Some(month) = self.get_month_at_index(index) {
            self.categories
                .iter()
                .filter(|direction_and_category| {
                    self.category_summary
                        .get(&((*direction_and_category).clone(), *month))
                        .map_or(false, |amount| amount != &0.0)
                })
                .map(|dac| dac.clone())
                .collect()
        } else {
            vec![]
        }
    }

    /** Returns the category label for a month index and a category index */
    pub fn get_category_by_index_for_month_at_index(
        &self,
        month_index: usize,
        category_index: usize,
    ) -> DirectionAndCategory {
        let categories = self.get_categories_for_month_by_index(month_index);
        categories
            .get(category_index)
            .map_or((None, None), |category| category.clone())
    }

    pub fn get_category_rows_for_month_by_index(&self, index: usize) -> Vec<Vec<String>> {
        if let Some(month) = self.get_month_at_index(index) {
            self.categories
                .iter()
                .map(|direction_and_category| {
                    self.category_summary
                        .get(&(direction_and_category.clone(), *month))
                        .map(|sum| (direction_and_category, sum))
                })
                .filter_map(|category_sum| category_sum)
                .map(|(direction_and_category, sum)| {
                    vec![
                        format!(
                            "\n{} - {}",
                            direction_and_category
                                .0
                                .as_ref()
                                .unwrap_or(&"*".to_string()),
                            direction_and_category
                                .1
                                .as_ref()
                                .unwrap_or(&"*".to_string())
                        ),
                        format!("\n{:02.2}\n", sum),
                    ]
                })
                .collect()
        } else {
            vec![]
        }
    }
}

pub struct TransactionsTable {
    transactions: Vec<Transaction>,
    recommended_input: Option<String>,
    file_path: PathBuf,
    file_type: FileType,
}

impl TransactionsTable {
    pub fn new(file_path: PathBuf) -> Self {
        let file_type = FileType::new(&file_path);
        Self {
            transactions: Vec::new(),
            recommended_input: None,
            file_path,
            file_type,
        }
    }

    pub fn load(&mut self) -> Result<()> {
        match self.file_type {
            FileType::Json => self.load_from_json(),
            FileType::Csv => self.load_from_csv(),
            FileType::Unknown => bail!("File type unknown"),
        }
    }

    pub fn load_from_json(&mut self) -> Result<()> {
        let file_string = fs::read_to_string(&self.file_path)?;
        self.transactions = serde_json::from_str(&file_string)?;
        self.transactions.sort();
        Ok(())
    }

    pub fn load_from_csv(&mut self) -> Result<()> {
        let mut reader = ReaderBuilder::new()
            .has_headers(true)
            .from_path(&self.file_path)?;
        self.transactions = Vec::new();
        for result in reader.deserialize() {
            let transaction = result?;
            self.transactions.push(transaction);
        }
        self.transactions.sort();
        Ok(())
    }

    pub fn save_transactions(&mut self) -> Result<()> {
        match self.file_type {
            FileType::Json => self.save_to_json(),
            FileType::Csv => self.save_to_csv(),
            FileType::Unknown => bail!("File type unknown"),
        }
    }

    pub fn save_to_json(&mut self) -> Result<()> {
        self.transactions.sort();
        fs::write(
            &self.file_path,
            serde_json::to_string_pretty(&self.transactions)?,
        )?;
        Ok(())
    }

    pub fn save_to_csv(&mut self) -> Result<()> {
        self.transactions.sort();
        // let file = File::create("transactions.csv")?;
        let mut writer = WriterBuilder::new()
            .delimiter(b',')
            .from_path(&self.file_path)?;
        for transaction in &self.transactions {
            writer.serialize(transaction)?;
        }
        writer.flush()?;
        Ok(())
    }

    pub fn new_transaction(&mut self) {
        let last_transaction_date = self.transactions.last().unwrap().date;
        self.transactions
            .push(Transaction::new(last_transaction_date));
    }

    pub fn delete_transaction(&mut self, i: usize) {
        if i < self.len() {
            self.transactions.remove(i);
        }
    }

    pub fn update_transaction(
        &mut self,
        row: usize,
        column: usize,
        input: &str,
    ) -> Result<(), String> {
        if let Some(transaction) = self.transactions.get_mut(row) {
            let input = self
                .recommended_input
                .as_ref()
                .map_or(input, |r| r.as_str());
            transaction.mutate_field(column, input)?
        }
        Ok(())
    }

    pub fn get_cell_text(&mut self, row: usize, column: usize) -> Option<String> {
        self.transactions
            .get(row)
            .map(|transaction| transaction.get_column_text(column))
            .flatten()
    }

    fn find_recommended_transactions_by_field(
        &self,
        row: usize,
        field: &TransactionField,
        input: &str,
    ) -> Option<&Transaction> {
        self.transactions
            .iter()
            .take(row)
            .rev()
            .find(|transaction| transaction.get_field_text(field).starts_with(input))
    }

    pub fn update_recommended_input(&mut self, row: usize, column: usize, input: &str) {
        if let Some(field) = TransactionField::get(column) {
            if input.chars().count() > 0 {
                // look for a previous input of the same field that starts with the given input
                self.recommended_input = self
                    .find_recommended_transactions_by_field(row, &field, input)
                    .map(|transaction| transaction.get_field_text(&field));
            } else {
                // look for a transaction with the same details and provide the relevant column
                let input_details = self
                    .transactions
                    .get(row)
                    .map_or("".to_string(), |transaction| {
                        transaction.get_field_text(&TransactionField::Details)
                    });
                self.recommended_input = self
                    .find_recommended_transactions_by_field(
                        row,
                        &TransactionField::Details,
                        &input_details,
                    )
                    .map(|transaction| transaction.get_field_text(&field))
            }
        }
    }

    pub fn get_recommended_input(&self, input: &str) -> &str {
        self.recommended_input
            .as_ref()
            .map(|recommended_input| {
                let input_len = input.chars().count();
                if input_len > recommended_input.chars().count() {
                    ""
                } else {
                    &recommended_input[input_len..]
                }
            })
            .unwrap_or("")
    }

    pub fn clear_recommended_input(&mut self) {
        self.recommended_input = None;
    }

    pub fn iter(&self) -> Iter<'_, Transaction> {
        self.transactions.iter()
    }

    pub fn len(&self) -> usize {
        self.transactions.len()
    }

    pub fn generate_report(&self) -> TransactionsReport {
        TransactionsReport::new(&self.transactions)
    }
}
