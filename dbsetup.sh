sqlx database create --database-url sqlite:data/archmage.db
sqlx migrate run --database-url sqlite:data/archmage.db
cargo sqlx prepare --database-url sqlite:data/archmage.db