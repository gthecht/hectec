use color_eyre::Result;
use core::fmt;
use ratatui::{
    text::Text,
    widgets::{Cell, Row},
};
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, fmt::Display, str::FromStr};
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
    Currency,
}

impl TransactionField {
    pub fn get(index: usize) -> Option<Self> {
        match index {
            0 => Some(Self::Date),
            1 => Some(Self::Amount),
            2 => Some(Self::Details),
            3 => Some(Self::Category),
            4 => Some(Self::Method),
            5 => Some(Self::Currency),
            _ => None,
        }
    }

    pub fn widths() -> Vec<u16> {
        vec![11, 10, 100, 15, 11, 9]
    }

    pub fn names() -> Vec<String> {
        vec![
            "Date", "Amount", "Details", "Category", "Method", "Currency",
        ]
        .into_iter()
        .map(|s| s.to_string())
        .collect()
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    pub date: SimpleDate,
    amount: f64,
    currency: String,
    pub details: String,
    category: String,
    method: String,
}

impl Transaction {
    pub fn new(date: SimpleDate) -> Self {
        Transaction {
            date,
            amount: 0.0,
            currency: "".to_string(),
            details: "".to_string(),
            category: "".to_string(),
            method: "".to_string(),
        }
    }

    pub fn mutate_field(&mut self, column_index: usize, input: &str) -> Result<(), String> {
        match TransactionField::get(column_index) {
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
                TransactionField::Currency => self.currency = input.to_string(),
            },
            None => {}
        }
        Ok(())
    }

    pub fn generate_row_text(&self) -> [String; 6] {
        [
            format!("{}", self.date),
            format!("{}", self.amount),
            self.details.clone(),
            self.category.clone(),
            self.method.clone(),
            self.currency.clone(),
        ]
    }

    pub fn generate_row(&self) -> Row {
        let cells: Vec<Cell> = self
            .generate_row_text()
            .iter()
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
