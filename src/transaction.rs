use color_eyre::Result;
use ratatui::{
    text::Text,
    widgets::{Cell, Row},
};
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, str::FromStr};
use time::{format_description, macros::format_description, OffsetDateTime};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    #[serde(with = "time::serde::iso8601")]
    pub date: OffsetDateTime,
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
    pub fn new(date: OffsetDateTime) -> Self {
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

    fn try_parse_date(&self, input: &str) -> Result<OffsetDateTime> {
        let format = format_description!(
            "[year]-[month]-[day] [hour]:[minute]:[second] [offset_hour \
                 sign:mandatory]"
        );
        Ok(OffsetDateTime::parse(
            &format!("{} 00:08:00 +02", input),
            &format,
        )?)
    }

    pub fn mutate_field(&mut self, column: &Column, input: &str) -> Result<(), &str> {
        match column.name() {
            "Date" => match self.try_parse_date(input) {
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
            self.date
                .format(&format_description::parse("[year]-[month]-[day]").unwrap())
                .unwrap(),
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
