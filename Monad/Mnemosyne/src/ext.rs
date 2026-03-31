pub fn build_bulk_insert_statement(
    table: &str,
    columns: &[&str],
    rows: usize,
    start_bind_index: usize,
) -> String {
    let cols = columns.join(", ");
    let mut bind = start_bind_index;

    let values = (0..rows)
        .map(|_| {
            let tuple = (0..columns.len())
                .map(|_| {
                    let token = format!("${bind}");
                    bind += 1;
                    token
                })
                .collect::<Vec<_>>()
                .join(", ");
            format!("({tuple})")
        })
        .collect::<Vec<_>>()
        .join(", ");

    format!("INSERT INTO {table} ({cols}) VALUES {values}")
}

pub fn build_cursor_pagination_query(
    table: &str,
    columns: &[&str],
    sort_column: &str,
    id_column: &str,
) -> String {
    let select = columns.join(", ");
    format!(
        "SELECT {select} FROM {table} WHERE ($1::timestamptz IS NULL OR ({sort_column}, {id_column}) < ($1, $2)) ORDER BY {sort_column} DESC, {id_column} DESC LIMIT $3"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_expected_insert_sql() {
        let sql = build_bulk_insert_statement("Uzume.stories", &["id", "author_id"], 2, 1);
        assert_eq!(
            sql,
            "INSERT INTO Uzume.stories (id, author_id) VALUES ($1, $2), ($3, $4)"
        );
    }
}
