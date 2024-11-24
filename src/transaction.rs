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

impl SimpleDate {
    pub fn from_string(date_string: &str) -> Result<SimpleDate, &str> {
        let splitter = if date_string.contains("-") { "-" } else { "." };
        let mut split_str = date_string.split(splitter);
        if let Some(year) = split_str.next() {
            if let Ok(year) = i32::from_str(year) {
                let year = if year < 100 { 2000 + year } else { year };
                if let Some(month) = split_str.next() {
                    if let Ok(month) = u8::from_str(month) {
                        if let Ok(m) = Month::try_from(month) {
                            if let Some(day) = split_str.next() {
                                if let Ok(day) = u8::from_str(day) {
                                    if let Ok(date) = Date::from_calendar_date(year, m, day) {
                                        return Ok(SimpleDate {
                                            year,
                                            month,
                                            day,
                                            date,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Err("failed to create date from string")
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
        SimpleDate::from_string(&s).map_err(serde::de::Error::custom)
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    pub date: SimpleDate,
    amount: f64,
    currency: String,
    details: String,
    category: String,
    method: String,
}

pub struct Column {
    name: String,
    pub width: u16,
}

impl Column {
    pub fn new(name: &str, width: u16) -> Self {
        Self {
            name: name.to_string(),
            width,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
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

    pub fn sort(a: &Transaction, b: &Transaction, column: &Column) -> Ordering {
        match column.name() {
            "Date" => a.date.cmp(&b.date),
            "Amount" => a.amount.partial_cmp(&b.amount).expect("amount not defined"),
            "Details" => a.details.cmp(&b.details),
            "Category" => a.category.cmp(&b.category),
            "Method" => a.method.cmp(&b.method),
            "Currency" => a.currency.cmp(&b.currency),
            &_ => a.date.cmp(&b.date), //warn("column not recognized")
        }
    }

    pub fn mutate_field(&mut self, column: &Column, input: &str) -> Result<(), &str> {
        match column.name() {
            "Date" => match SimpleDate::from_string(input) {
                Ok(date) => self.date = date,
                Err(_) => return Err(" failed to parse as date"),
            },
            "Amount" => match f64::from_str(input) {
                Ok(num) => self.amount = num,
                Err(_) => return Err(" failed to parse as number"),
            },
            "Details" => self.details = input.to_string(),
            "Category" => self.category = input.to_string(),
            "Method" => self.method = input.to_string(),
            "Currency" => self.currency = input.to_string(),
            &_ => return Err(" column not recognized"),
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
