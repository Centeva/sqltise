use wasm_bindgen::prelude::*;
use super::model::{ ColumnSelections, ColumnSelection, CsvErrors, ColumnType, ColumnHeader, ParseError, StatementType, StatementSelections};


#[wasm_bindgen]
pub fn generate_file(data: &str, statements: JsValue, columns: JsValue, corrections: JsValue) -> JsValue {
    let statements: StatementSelections = statements.into_serde().unwrap();
    let columns: ColumnSelections = columns.into_serde().unwrap();
    let corrections: CsvErrors = corrections.into_serde().unwrap();

    let mut reader = csv::ReaderBuilder::new()
	    .has_headers(true)
	    .from_reader(data.as_bytes());

    let mut headers = Vec::new();

	let header_value = match reader.headers() {
		Ok(h) => h,
		Err(_e) => {
			return JsValue::NULL;
		}
	};

	for (index, column) in header_value.iter().enumerate() {
		let header = ColumnHeader{
			name: column.to_string(),
			index
		};
		headers.push(header);
	}

    let mut results: Vec<String> = Vec::new();

    for statement in statements.value {
        let statement_columns: Vec<ColumnSelection> = columns.value.iter()
            .filter(|col| col.statement_id == statement.id)
            .cloned()
            .collect();

        let mut statement_lines: Vec<String> = Vec::new();

        for (index, record) in reader.records().enumerate() {
            let record = match record {
                Ok(r) => r,
                Err(e) => {
                    let error = &format!("{{ error: 'Not a proper csv file', kind: {} }}", e);
                    let json: ParseError = serde_json::from_str(error).unwrap();
                    return JsValue::from_serde(&json).unwrap();
                }
            };

            let mut output_columns: Vec<(String, String)> = Vec::new();

            for ref column in &statement_columns {
                let id = column.column;
                let mut value: String = record[id].to_string();

                if let Some(correction) = corrections.value.iter().find(|c| c.column_id == id && c.statement_id == statement.id && c.rows.iter().any(|&r| r == index)) {
                    value = correction.error_text.clone();
                }

                let name = match column.use_source {
                    true => {
                        headers.iter().find(|h| h.index == id).unwrap().name.clone()
                    },
                    false => column.name.clone().unwrap(),
                };

                // Format the values here to meet the requirement of the type
                match column.r#type{
                    ColumnType::VarChar => {
                        value = str::replace(&value, "'", "''");
                        value = format!("'{}'", value);
                    },
                    ColumnType::Date => {
                        value = format!("'{}'", value);
                    },
                    ColumnType::DateTime => {
                        value = format!("'{}'", value);
                    },
                    _ => {}
                }

                output_columns.push((name, value.to_string()));
            }

            
            let output: String = match statement.r#type {
                StatementType::Insert => {
                    let column_names = output_columns.iter().map(|(c, _)| c.clone())
                        .collect::<Vec<String>>().join(", ");
                    let values = output_columns.iter().map(|(_, v)| v.clone())
                        .collect::<Vec<String>>().join(", ");
                    format!("INSERT INTO {} ({}) VALUES ({});", statement.table, column_names, values)
                },
                StatementType::Update => {
                    // let sets = output_columns.iter()
                    //     .map(|(c, v)| format!("{} = {}", c, v))
                    //     .collect::<Vec<String>>().join(", ");
                    // format!("UPDATE {} SET {} WHERE 1 = 1", statement.table, sets)
                    format!("Oops, Update statements are not supported yet. sorry.")
                },
            };

            statement_lines.push(output);
        }

        results.push(statement_lines.join("\n"));
    }

    return JsValue::from_serde(&results).unwrap();
}