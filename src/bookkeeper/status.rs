use bigdecimal::BigDecimal;
use nu_table::{draw_table, StyledString, Table, TextStyle, Theme};
use std::collections::HashMap;
use toml::value::Table as TomlTable;

use crate::{
    error::Result,
    parser::{Operation, OperationType},
};

#[allow(unused)]
pub(super) struct BookkeeperStatus {
    /// Total amount spent.
    pub take_total: BigDecimal,
    /// Total amount received.
    pub put_total: BigDecimal,
    /// List of all operations.
    pub all_operations: Vec<Operation>,
    /// List of put operations.
    pub put_operations: Vec<Operation>,
    /// List of take operations.
    pub take_operations: Vec<Operation>,
}

fn line_from_operation(operation: &Operation) -> Vec<StyledString> {
    let Operation { day, kind, amount, description } = operation;

    let (_, kind_symbol) = kind.name_and_symbol();

    let line: Vec<StyledString> = [
        format!("{day:2}"),
        format!("{kind_symbol}"),
        format!("{amount:8.2}"),
        description.clone(),
    ]
    .into_iter()
    .map(|x| StyledString::new(x, TextStyle::basic_left()))
    .collect();

    line
}

fn make_table_data(operations: &[Operation]) -> (Vec<StyledString>, Vec<Vec<StyledString>>) {
    let table_headers = ["day", "op", "amount", "description"]
        .iter()
        .take(4)
        .map(|column_name| StyledString::new(column_name, TextStyle::default_header()))
        .collect::<Vec<StyledString>>();

    let row_data = operations
        .into_iter()
        .map(|x| line_from_operation(x))
        .collect();

    (table_headers, row_data)
}

impl BookkeeperStatus {
    pub(super) fn display(&self) {
        let balance = &self.put_total - &self.take_total;

        println!("\tIncoming: R$ {:.2}", self.put_total);
        println!("\tOutgoing: R$ {:.2}", self.take_total);
        println!("\tBalance:  R$ {:.2}", balance);
        println!();
        println!("\tOperations:");

        let mut all_operations = self.all_operations.clone();
        all_operations.sort_by(|a, b| a.day.cmp(&b.day).then(a.kind.cmp(&b.kind)));

        for operation in &all_operations {
            let (_, kind_symbol) = operation.kind.name_and_symbol();
            let Operation { description, amount, day, .. } = &operation;

            println!("\t\t{day:2} {kind_symbol} {amount:8.2} {description}");
        }

        let width = 150;
        // The mocked up table data
        let (headers, rows) = make_table_data(&all_operations);
        // The table itself
        let table = Table::new(headers, rows, Theme::light());
        // FIXME: Config isn't available from here so just put these here to compile
        let color_hm: HashMap<String, nu_ansi_term::Style> = HashMap::new();
        // Capture the table as a string
        let output_table = draw_table(&table, width, &color_hm, false);
        // Draw the table
        println!("{}", output_table);
    }

    pub(super) fn from_toml_table(table: &TomlTable) -> Result<Self> {
        let (take, put) = (
            table["take"].as_array().unwrap(),
            table["put"].as_array().unwrap(),
        );

        let mut all_operations = vec![];
        let mut put_operations = vec![];
        let mut take_operations = vec![];

        for operation in take.iter().chain(put) {
            let operation = operation.as_str().unwrap();
            let operation = Operation::from_str(operation).unwrap();

            all_operations.push(operation.clone());

            match operation.kind {
                OperationType::Withdraw => take_operations.push(operation),
                OperationType::Deposit => put_operations.push(operation),
            }
        }

        let take_total: BigDecimal = take_operations.iter().map(|x| &x.amount).sum();
        let put_total: BigDecimal = put_operations.iter().map(|x| &x.amount).sum();

        Ok(Self {
            take_total,
            put_total,
            all_operations,
            take_operations,
            put_operations,
        })
    }
}
