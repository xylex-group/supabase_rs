use cruet::string::singularize::to_singular;
use std::collections::BTreeMap;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::str::Chars;
use tokio;
use tokio_postgres::{Config, NoTls, SimpleQueryMessage, SimpleQueryRow};

/// Generates Rust types for Supabase tables and RPC functions in the default 'public' schema.
///
/// This function connects to your Supabase Postgres database and generates Rust structs
/// for all tables and their columns, as well as argument structs for RPC functions.
///
/// # Arguments
///
/// * `user` - PostgreSQL username
/// * `password` - PostgreSQL password
/// * `singularize_struct_name` - If `true`, table names are singularized for struct names
/// * `included_tables` - List of table names to include (empty for all tables)
///
/// # Examples
///
/// ```rust,no_run
/// use supabase_rs::type_gen::generate_supabase_types;
///
/// #[tokio::main]
/// async fn main() {
///     generate_supabase_types(
///         "postgres",
///         "password",
///         true,
///         &["users", "posts"]
///     ).await;
/// }
/// ```
pub async fn generate_supabase_types(
    user: &str,
    password: &str,
    singularize_struct_name: bool,
    included_tables: &[&str],
) {
    generate_supabase_types_with_schema(
        user,
        password,
        singularize_struct_name,
        included_tables,
        "public",
    )
    .await
}

/// Generates Rust types for Supabase tables and RPC functions with schema support.
///
/// This function extends `generate_supabase_types` with schema-aware type generation.
/// It generates:
/// 1. Primary and New<T> structs for each table
/// 2. Helper methods for column names and table names
/// 3. RPC function argument structs in a separate `rpc` module
///
/// RPC function argument structs are generated for all PostgreSQL functions in the
/// specified schema, with proper type mapping and parameter mode handling (IN, INOUT).
/// OUT parameters are excluded as they are not passed as arguments.
///
/// # Arguments
///
/// * `user` - PostgreSQL username
/// * `password` - PostgreSQL password
/// * `singularize_struct_name` - If `true`, table names are singularized for struct names
/// * `included_tables` - List of table names to include (empty for all tables)
/// * `schema` - PostgreSQL schema name (defaults to "public")
///
/// # Examples
///
/// ```rust,no_run
/// use supabase_rs::type_gen::generate_supabase_types_with_schema;
///
/// #[tokio::main]
/// async fn main() {
///     generate_supabase_types_with_schema(
///         "postgres",
///         "password",
///         true,
///         &["users", "posts"],
///         "public"
///     ).await;
/// }
/// ```
///
/// # Generated Output Example
///
/// For a function `create_user(name text, age integer)`:
/// ```rust
/// pub mod rpc {
///     use serde::Serialize;
///
///     #[derive(Debug, Serialize, Clone)]
///     pub struct CreateUserArgs {
///         pub name: String,
///         pub age: i32,
///     }
/// }
/// ```
pub async fn generate_supabase_types_with_schema(
    user: &str,
    password: &str,
    singularize_struct_name: bool,
    included_tables: &[&str],
    schema: &str,
) {
    // connect to your supabase Postgres pooler
    let mut config: Config = Config::new();
    config
        .host("aws-0-eu-central-1.pooler.supabase.com")
        .port(6543)
        .user(user)
        .password(password)
        .dbname("postgres");

    let (client, connection) = config
        .connect(NoTls)
        .await
        .expect("Failed to connect to database");

    // spawn the connection driver
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {e}");
        }
    });

    // fetch table schema info
    let table_query = format!(
        "SELECT
            table_name,
            column_name,
            data_type,
            is_nullable,
            column_default,
            is_identity
        FROM information_schema.columns
        WHERE table_schema = '{}'
        ORDER BY table_name, ordinal_position;",
        schema
    );

    let mut table_definitions: BTreeMap<String, Vec<(String, String, bool, bool)>> =
        BTreeMap::new();
    let mut all_columns: BTreeMap<String, Vec<String>> = BTreeMap::new();

    let table_rows: Vec<SimpleQueryRow> = client
        .simple_query(&table_query)
        .await
        .expect("simple_query for tables")
        .into_iter()
        .filter_map(|m| match m {
            SimpleQueryMessage::Row(r) => Some(r),
            _ => None,
        })
        .collect();

    for row in table_rows {
        let table_name: String = row
            .get::<usize>(0)
            .expect("table_name not found")
            .to_string();

        if !included_tables.contains(&table_name.as_ref()) {
            continue;
        }

        let column_name: String = row
            .get::<usize>(1)
            .expect("column_name not found")
            .to_string();

        let data_type: String = row
            .get::<usize>(2)
            .expect("data_type not found")
            .to_string();

        let is_nullable: String = row
            .get::<usize>(3)
            .expect("is_nullable not found")
            .to_string();

        let column_default: Option<String> = row.get::<usize>(4).and_then(|s| {
            if s.is_empty() {
                None
            } else {
                Some(s.to_string())
            }
        });

        let is_identity: String = row
            .get::<usize>(5)
            .expect("is_identity not found")
            .to_string();

        let base_rust_type: &'static str = match data_type.as_str() {
            "integer" => "i32",
            "bigint" => "i64",
            "smallint" => "i16",
            "text" | "varchar" | "char" => "String",
            "boolean" => "bool",
            "real" | "double precision" => "f64",
            "numeric" | "decimal" => "Decimal",
            "timestamp without time zone" => "NaiveDateTime",
            "timestamp with time zone" => "DateTime<Utc>",
            "date" => "NaiveDate",
            "uuid" => "Uuid",
            "json" | "jsonb" => "Value",
            _ => "String",
        };

        // only nullable columns become Option in the primary struct
        let nullable_flag: bool = is_nullable == "YES";
        // default_flag still needed for New<> below
        let default_flag: bool = is_identity == "YES" || column_default.is_some();
        // **Primary** type uses only nullable_flag
        let rust_type: String = if nullable_flag {
            format!("Option<{base_rust_type}>")
        } else {
            base_rust_type.to_string()
        };

        // stash both the primary type and flags for use in New<>
        table_definitions
            .entry(table_name.clone())
            .or_default()
            .push((column_name.clone(), rust_type, nullable_flag, default_flag));

        all_columns.entry(table_name).or_default().push(column_name);
    }

    // fetch RPC function info
    let rpc_query = format!(
        "SELECT
            r.routine_name,
            p.parameter_name,
            p.data_type,
            p.ordinal_position,
            p.parameter_mode
        FROM information_schema.routines r
        JOIN information_schema.parameters p 
            ON r.specific_name = p.specific_name
        WHERE r.routine_type = 'FUNCTION' 
          AND r.routine_schema = '{}'
        ORDER BY r.routine_name, p.ordinal_position;",
        schema
    );

    let mut rpc_definitions: BTreeMap<String, Vec<(String, String, String)>> = BTreeMap::new();

    let rpc_rows: Vec<SimpleQueryRow> = client
        .simple_query(&rpc_query)
        .await
        .expect("simple_query for rpc functions")
        .into_iter()
        .filter_map(|m| match m {
            SimpleQueryMessage::Row(r) => Some(r),
            _ => None,
        })
        .collect();

    for row in rpc_rows {
        let routine_name: String = row
            .get::<usize>(0)
            .expect("routine_name not found")
            .to_string();
        let parameter_name: String = row
            .get::<usize>(1)
            .expect("parameter_name not found")
            .to_string();
        let data_type: String = row
            .get::<usize>(2)
            .expect("data_type not found")
            .to_string();
        let _ordinal_position: String = row
            .get::<usize>(3)
            .expect("ordinal_position not found")
            .to_string();
        let parameter_mode: String = row
            .get::<usize>(4)
            .expect("parameter_mode not found")
            .to_string();

        // Only include IN and INOUT parameters (OUT parameters are not passed as arguments)
        if parameter_mode == "OUT" {
            continue;
        }

        let base_rust_type: &'static str = match data_type.as_str() {
            "integer" => "i32",
            "bigint" => "i64",
            "smallint" => "i16",
            "text" | "varchar" | "char" => "String",
            "boolean" => "bool",
            "real" | "double precision" => "f64",
            "numeric" | "decimal" => "Decimal",
            "timestamp without time zone" => "NaiveDateTime",
            "timestamp with time zone" => "DateTime<Utc>",
            "date" => "NaiveDate",
            "uuid" => "Uuid",
            "json" | "jsonb" => "Value",
            _ => "String",
        };

        // For RPC arguments, we don't have nullable information from information_schema.parameters
        // We'll assume they're not nullable (required) unless they have a default value
        // For simplicity, we'll generate non-Option types
        // In the future we could check pg_proc.proargdefaults to detect defaults
        let rust_type = base_rust_type.to_string();

        rpc_definitions.entry(routine_name).or_default().push((
            parameter_name,
            rust_type,
            parameter_mode,
        ));
    }

    let mut output: String = String::new();
    output.push_str("#![allow(dead_code)]\n\n");
    output.push_str("use serde::{Serialize, Deserialize};\n");
    output.push_str("use serde_json::Value;\n");
    output.push_str("use serde_with::skip_serializing_none;\n");
    output.push_str("use chrono::{DateTime, Utc, NaiveDate, NaiveDateTime};\n");
    output.push_str("use uuid::Uuid;\n");
    output.push_str("use rust_decimal::Decimal;\n\n");

    let mut all_tables: Vec<String> = Vec::new();
    let mut trait_methods: String = String::new();
    let mut impl_methods: String = String::new();

    // ensure tables are emitted in sorted order:
    let mut tables: Vec<_> = table_definitions.keys().cloned().collect();
    tables.sort();
    for table in &tables {
        let columns: &Vec<(String, String, bool, bool)> = &table_definitions[table];
        let struct_name: String = if singularize_struct_name {
            pascal_case(&to_singular(table))
        } else {
            pascal_case(table)
        };
        all_tables.push(table.clone());

        // — Primary struct
        output.push_str(&format!(
            "#[derive(Debug, Serialize, Deserialize, Clone)]\n\
             pub struct {struct_name} {{\n"
        ));
        for (col, ty, _, _) in columns {
            let field: String = safe_field_name(col);
            if &field != col {
                output.push_str(&format!("    #[serde(rename = \"{col}\")]\n"));
            }
            output.push_str(&format!("    pub {field}: {ty},\n"));
        }
        output.push_str("}\n\n");

        // — New<T> struct
        let new_name: String = format!("New{struct_name}");
        output.push_str(&format!(
            "#[derive(Debug, Serialize, Deserialize, Clone, Default)]\n\
             #[skip_serializing_none]\n\
             pub struct {new_name} {{\n"
        ));
        for (col, ty, nullable, default) in columns {
            let field: String = safe_field_name(col);
            // unwrap Option<…>
            let inner: &str = ty
                .strip_prefix("Option<")
                .and_then(|s| s.strip_suffix('>'))
                .unwrap_or(ty);
            if &field != col {
                output.push_str(&format!("    #[serde(rename = \"{col}\")]\n"));
            }
            if *nullable || *default {
                output.push_str(&format!("    pub {field}: Option<{inner}>,\n"));
            } else {
                output.push_str(&format!("    pub {field}: {inner},\n"));
            }
        }
        output.push_str("}\n\n");

        // — columns() fn
        output.push_str(&format!(
            "impl {struct_name} {{\n    pub fn columns() -> &'static [&'static str] {{\n"
        ));
        output.push_str("        &[\n");
        for col in &all_columns[table] {
            output.push_str(&format!("            \"{col}\",\n"));
        }
        output.push_str("        ]\n    }\n\n");
        output.push_str(&format!(
            "    pub fn table_name() -> &'static str {{ \"{}\" }}\n",
            table
        ));
        output.push_str("}\n\n");

        // — extension trait methods
        trait_methods.push_str(&format!("    fn select_{table}(&self) -> QueryBuilder;\n"));
        impl_methods.push_str(&format!(
            "    fn select_{table}(&self) -> QueryBuilder {{\n        QueryBuilder::new(self.clone(), \"{table}\")\n    }}\n"
        ));
    }

    // ALL_TABLES constant
    output.push_str("pub const ALL_TABLES: &[&str] = &[\n");
    for t in &all_tables {
        output.push_str(&format!("    \"{t}\",\n"));
    }
    output.push_str("];\n\n");

    // Generate RPC module if there are any functions
    if !rpc_definitions.is_empty() {
        output.push_str("pub mod rpc {\n");
        output.push_str("    use serde::Serialize;\n");
        output.push_str("    use serde_json::Value;\n");
        output.push_str("    use chrono::{DateTime, Utc, NaiveDate, NaiveDateTime};\n");
        output.push_str("    use uuid::Uuid;\n");
        output.push_str("    use rust_decimal::Decimal;\n\n");

        // ensure functions are emitted in sorted order:
        let mut functions: Vec<_> = rpc_definitions.keys().cloned().collect();
        functions.sort();
        for function in &functions {
            let parameters = &rpc_definitions[function];
            let struct_name = format!("{}Args", pascal_case(function));

            output.push_str(&format!("    #[derive(Debug, Serialize, Clone)]\n"));
            output.push_str(&format!("    pub struct {} {{\n", struct_name));

            for (param_name, rust_type, param_mode) in parameters {
                let field = safe_field_name(param_name);
                if &field != param_name {
                    output.push_str(&format!("        #[serde(rename = \"{param_name}\")]\n"));
                }
                // Add comment about parameter mode if it's INOUT
                if param_mode == "INOUT" {
                    output.push_str(&format!("        // INOUT parameter\n"));
                }
                output.push_str(&format!("        pub {field}: {rust_type},\n"));
            }
            output.push_str("    }\n\n");
        }

        output.push_str("}\n\n");
    }

    if singularize_struct_name {
        output.push_str("\n/// Map a singular resource name to its table\n");
        output.push_str("pub fn get_resource_table(resource_type: &str) -> Result<&'static str, std::io::Error> {\n");
        output.push_str("    match resource_type {\n");
        for table in &tables {
            let singular = to_singular(table);
            output.push_str(&format!(
                "        \"{singular}\" => Ok(\"{table}\"),\n",
                singular = singular,
                table = table
            ));
        }
        output.push_str(
            "        _ => Err(std::io::Error::new(\n\
             std::io::ErrorKind::InvalidInput,\n\
             format!(\"Unknown resource type: {}\", resource_type)\n\
             )),\n",
        );
        output.push_str("    }\n}\n");
    }

    if fs::metadata("src/lib.rs").is_ok() {
        let mut lib_rs: File = OpenOptions::new().read(true).open("src/lib.rs").unwrap();
        let mut contents: String = String::new();
        lib_rs.read_to_string(&mut contents).unwrap();
        if !contents.contains("pub mod supabase_types;") {
            let mut lib_rs: File = OpenOptions::new().append(true).open("src/lib.rs").unwrap();
            lib_rs.write_all(b"pub mod supabase_types;\n").unwrap();
        }
    } else if fs::metadata("src/mod.rs").is_ok() {
        let mut mod_rs: File = OpenOptions::new().read(true).open("src/mod.rs").unwrap();
        let mut contents: String = String::new();
        mod_rs.read_to_string(&mut contents).unwrap();
        if !contents.contains("pub mod supabase_types;") {
            let mut mod_rs: File = OpenOptions::new().append(true).open("src/mod.rs").unwrap();
            mod_rs.write_all(b"pub mod supabase_types;\n").unwrap();
        }
    }

    // write file
    if fs::metadata("src/supabase_types.rs").is_ok() {
        fs::remove_file("src/supabase_types.rs").unwrap();
    }
    let mut file: File = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("src/supabase_types.rs")
        .unwrap();
    file.write_all(output.as_bytes()).unwrap();
}

fn pascal_case(s: &str) -> String {
    s.split('_')
        .map(|w| {
            let mut c: Chars<'_> = w.chars();
            match c.next() {
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                None => String::new(),
            }
        })
        .collect()
}

fn snake_case(s: &str) -> String {
    let mut out: String = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            out.push('_');
        }
        out.push(c.to_lowercase().next().unwrap());
    }
    out
}

fn safe_field_name(col: &str) -> String {
    if col == "type" {
        "type_".into()
    } else if col.chars().all(|c| c.is_ascii_uppercase()) {
        col.to_lowercase()
    } else {
        snake_case(col)
    }
}
