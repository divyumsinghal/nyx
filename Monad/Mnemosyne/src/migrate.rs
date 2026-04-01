use std::path::{Path, PathBuf};

use nun::Result;
use sqlx::{migrate::Migrator, PgPool};

pub fn schema_migration_path(base: impl AsRef<Path>, schema: &str) -> PathBuf {
    base.as_ref().join(schema)
}

pub async fn run_schema_migrations(
    pool: &PgPool,
    base: impl AsRef<Path>,
    schema: &str,
) -> Result<()> {
    let path = schema_migration_path(base, schema);
    if !path.exists() {
        return Ok(());
    }

    let migrator = Migrator::new(path.as_path())
        .await
        .map_err(nun::NyxError::internal)?;
    migrator.run(pool).await.map_err(nun::NyxError::internal)
}

pub fn discover_schema_migrations(base: impl AsRef<Path>) -> Result<Vec<String>> {
    let mut schemas = std::fs::read_dir(base)
        .map_err(nun::NyxError::internal)?
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            entry
                .file_type()
                .ok()
                .filter(|ty| ty.is_dir())
                .and_then(|_| entry.file_name().into_string().ok())
        })
        .collect::<Vec<_>>();

    schemas.sort();
    Ok(schemas)
}

pub async fn run_migrations_for_schemas<I, S>(
    pool: &PgPool,
    base: impl AsRef<Path>,
    schemas: I,
) -> Result<Vec<String>>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut applied = Vec::new();
    for schema in schemas {
        let schema = schema.as_ref();
        let path = schema_migration_path(base.as_ref(), schema);
        if !path.exists() {
            continue;
        }
        run_schema_migrations(pool, base.as_ref(), schema).await?;
        applied.push(schema.to_string());
    }

    Ok(applied)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_schema_path() {
        assert!(schema_migration_path("migrations", "Uzume")
            .ends_with(std::path::Path::new("migrations").join("Uzume")));
    }
}
