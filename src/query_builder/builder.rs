use crate::query::{Filter, Query, QueryBuilder, Sort};
use crate::SupabaseClient;

use serde_json::Value;

impl QueryBuilder {
    /// Constructs a new `QueryBuilder` for a specified table.
    ///
    /// # Arguments
    /// * `client` - A `SupabaseClient` instance to be used for the query.
    /// * `table_name` - A string slice that specifies the table name for the query.
    ///
    /// # Returns
    /// Returns a new instance of `QueryBuilder`.
    pub fn new(client: SupabaseClient, table_name: &str) -> Self {
        QueryBuilder {
            client,
            query: Query::new(),
            table_name: table_name.to_owned(),
        }
    }

    pub fn columns(mut self, columns: Vec<&str>) -> QueryBuilder {
        // add query params &select=column1,column2
        let columns_str: String = columns.join(",");
        self.query.add_param("select", &columns_str);
        self
    }

    /// Adds a filter to the query to check if the column is equal to a specified value.
    ///
    /// # Arguments
    /// * `column` - The column name to apply the filter.
    /// * `value` - The value to compare against the column.
    ///
    /// # Returns
    /// Returns the `QueryBuilder` instance to allow for method chaining.
    pub fn eq(mut self, column: &str, value: &str) -> Self {
        self.query.add_param(column, &format!("eq.{value}"));
        self
    }

    /// Adds a filter to the query to check if the column is not equal to a specified value.
    ///
    /// # Arguments
    /// * `column` - The column name to apply the filter.
    /// * `value` - The value to compare against the column.
    ///
    /// # Returns
    /// Returns the `QueryBuilder` instance to allow for method chaining.
    pub fn neq(mut self, column: &str, value: &str) -> Self {
        self.query.add_param(column, &format!("neq.{value}"));
        self
    }

    /// Adds a filter to the query to check if the column is greater than a specified value.
    ///
    /// # Arguments
    /// * `column` - The column name to apply the filter.
    /// * `value` - The value to compare against the column.
    ///
    /// # Returns
    /// Returns the `QueryBuilder` instance to allow for method chaining.
    pub fn gt(mut self, column: &str, value: &str) -> Self {
        self.query.add_param(column, &format!("gt.{value}"));
        self
    }

    /// Adds a filter to the query to check if the column is less than a specified value.
    ///
    /// # Arguments
    /// * `column` - The column name to apply the filter.
    /// * `value` - The value to compare against the column.
    ///
    /// # Returns
    /// Returns the `QueryBuilder` instance to allow for method chaining.
    pub fn lt(mut self, column: &str, value: &str) -> Self {
        self.query.add_param(column, &format!("lt.{value}"));
        self
    }

    /// Adds a filter to the query to check if the column is greater than or equal to a specified value.
    ///
    /// # Arguments
    /// * `column` - The column name to apply the filter.
    /// * `value` - The value to compare against the column.
    ///
    /// # Returns
    /// Returns the `QueryBuilder` instance to allow for method chaining.
    pub fn gte(mut self, column: &str, value: &str) -> Self {
        self.query.add_param(column, &format!("gte.{value}"));
        self
    }

    /// Adds a filter to the query to check if the column is less than or equal to a specified value.
    ///
    /// # Arguments
    /// * `column` - The column name to apply the filter.
    /// * `value` - The value to compare against the column.
    ///
    /// # Returns
    /// Returns the `QueryBuilder` instance to allow for method chaining.
    pub fn lte(mut self, column: &str, value: &str) -> Self {
        self.query.add_param(column, &format!("lte.{value}"));
        self
    }

    /// Adds a parameter to the query to count the exact number of rows that match the query.
    ///
    /// # Returns
    /// Returns the `QueryBuilder` instance to allow for method chaining.
    pub fn count(mut self) -> Self {
        self.query.add_param("count", "exact");
        self
    }

    /// Adds a limit to the number of rows returned by the query.
    ///
    /// # Arguments
    /// * `limit` - The maximum number of rows to return.
    ///
    /// # Returns
    /// Returns the `QueryBuilder` instance to allow for method chaining.
    pub fn limit(mut self, limit: usize) -> Self {
        self.query.add_param("limit", &limit.to_string());
        self
    }

    /// Adds an offset to the query to skip a specified number of rows.
    ///
    /// # Arguments
    /// * `offset` - The number of rows to skip from the beginning of the result set.
    ///
    /// # Returns
    /// Returns the `QueryBuilder` instance to allow for method chaining.
    pub fn offset(mut self, offset: usize) -> Self {
        self.query.add_param("offset", &offset.to_string());
        self
    }

    /// Adds a range to the query for pagination using PostgREST range syntax.
    ///
    /// # Arguments
    /// * `from` - The starting index (0-based) of the range.
    /// * `to` - The ending index (inclusive) of the range.
    ///
    /// # Returns
    /// Returns the `QueryBuilder` instance to allow for method chaining.
    ///
    /// # Examples
    /// ```rust,no_run
    /// # use supabase_rs::SupabaseClient;
    /// # async fn example(client: SupabaseClient) -> Result<(), String> {
    /// // Get rows 10-19 (10 rows starting from index 10)
    /// let rows = client
    ///     .from("users")
    ///     .range(10, 19)
    ///     .execute()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn range(mut self, from: usize, to: usize) -> Self {
        self.query.set_range(from, to);
        self
    }

    /// Adds a filter to the query to check if the column is null.
    ///
    /// # Arguments
    /// * `column` - The column name to apply the filter.
    ///
    /// # Returns
    /// Returns the `QueryBuilder` instance to allow for method chaining.
    pub fn order(mut self, column: &str, ascending: bool) -> Self {
        let order_value: &str = if ascending { "asc" } else { "desc" };
        self.query
            .add_param("order", &format!("{column}.{order_value}"));
        self
    }

    /// Adds a full-text search filter to the query.
    ///
    /// # Arguments
    /// * `column` - The column name to perform the text search on.
    /// * `value` - The value to search for within the column.
    ///
    /// # Returns
    /// Returns the `QueryBuilder` instance to allow for method chaining.
    pub fn text_search(mut self, column: &str, value: &str) -> Self {
        self.query.add_param(column, &format!("fts.{value}"));
        self
    }

    /// Filters results where `column` is in the given list.
    ///
    /// # Arguments
    /// * `column` - The column name to apply the filter.
    /// * `values` - A slice of values to check against the column.
    ///
    /// # Returns
    /// Returns the `QueryBuilder` instance to allow for method chaining.
    pub fn in_<T>(mut self, column: &str, values: &[T]) -> Self
    where
        T: ToString,
    {
        let list = values
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(",");
        self.query.add_param(column, &format!("in.({})", list));
        self
    }

    /// Executes the constructed query against the database.
    ///
    /// # Returns
    /// Returns a `Result` containing either a vector of `Value` representing the fetched records, or a `String` error message.
    pub async fn execute(self) -> Result<Vec<Value>, String> {
        self.client
            .execute_with_query(&self.table_name, &self.query)
            .await
    }

    /// Executes the constructed query against the database.
    /// Note: Results are not guaranteed deterministic unless you call order(...)).
    ///
    /// # Returns
    /// - `Ok(Vec<Value>)` with the fetched records when the request succeeds.
    /// - `Err(String)` with an error message when the request fails.
    pub async fn first(self) -> Result<Option<Value>, String> {
        // ask for 1 row for efficiency
        let rows = self.limit(1).execute().await?;
        Ok(rows.into_iter().next())
    }

    /// Executes the constructed query and returns the single matching row, or an error if there are 0 or >1 matches.
    /// Note: Results are not guaranteed deterministic unless you call order(...)).
    ///
    /// # Returns
    /// - Ok(Value) when exactly one row is found.
    /// - Err(String) when no rows match.
    /// - Err(String) when more than one row matches.
    /// - Err(String) for other request failures.
    pub async fn single(self) -> Result<Value, String> {
        // ask for up to 2 rows to detect multiples
        let rows = self.limit(2).execute().await?;
        match rows.len() {
            1 => Ok(rows.into_iter().next().expect("Expected at least 1 row")),
            0 => Err("NotFound: no rows matched the query".into()),
            _ => Err("MultipleRows: expected exactly one row but found multiple".into()),
        }
    }
}

impl Query {
    /// Constructs a new `Query` instance using the default settings.
    ///
    /// # Examples
    ///
    /// ```
    /// # use supabase_rs::query::Query;
    /// let query = Query::new();
    /// ```
    pub fn new() -> Query {
        Query::default()
    }

    /// Adds a key-value pair to the query parameters.
    ///
    /// # Arguments
    /// * `key` - A string slice that holds the key of the parameter.
    /// * `value` - A string slice that holds the value of the parameter.
    ///
    /// # Examples
    ///
    /// ```
    /// # use supabase_rs::query::Query;
    /// let mut query = Query::new();
    /// query.add_param("name", "John Doe");
    /// ```
    pub fn add_param(&mut self, key: &str, value: &str) {
        let key_value_pair = (key.to_owned(), value.to_owned());
        if !self.params.contains(&key_value_pair) {
            self.params.push(key_value_pair);
        }
    }

    /// Adds a filter to the query.
    ///
    /// # Arguments
    /// * `filter` - A `Filter` struct containing the column name, operator, and value for the filter.
    ///
    /// # Examples
    /// ```
    /// # use supabase_rs::query::{Query, Filter, Operator};
    /// let mut query = Query::new();
    /// let filter = Filter {
    ///     column: "age".to_string(),
    ///     operator: Operator::GreaterThan,
    ///     value: "30".to_string(),
    /// };
    /// query.add_filter(filter);
    /// ```
    pub fn add_filter(&mut self, filter: Filter) {
        self.filters.push(filter);
    }

    /// Adds a sorting criterion to the query.
    ///
    /// # Arguments
    /// * `sort` - A `Sort` struct containing the column name and the sorting order.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # use supabase_rs::query::{Query, Sort, SortOrder};
    /// let mut query = Query::new();
    /// let sort = Sort {
    ///     column: "name".to_string(),
    ///     order: SortOrder::Ascending,
    /// };
    /// query.add_sort(sort);
    /// ```
    pub fn add_sort(&mut self, sort: Sort) {
        self.sorts.push(sort);
    }

    /// Sets the range for pagination.
    ///
    /// # Arguments
    /// * `from` - The starting index (0-based) of the range.
    /// * `to` - The ending index (inclusive) of the range.
    pub fn set_range(&mut self, from: usize, to: usize) {
        self.range = Some((from, to));
    }

    /// Gets the range if set.
    ///
    /// # Returns
    /// Returns an `Option<(usize, usize)>` containing the range if set.
    pub fn get_range(&self) -> Option<(usize, usize)> {
        self.range
    }

    /// Builds and returns the query string from the current state of the query parameters.
    ///
    /// # Returns
    /// A `String` that represents the URL-encoded query string.
    ///
    /// # Examples
    ///
    /// ```
    /// # use supabase_rs::query::Query;
    /// let mut query = Query::new();
    /// query.add_param("name", "John Doe");
    /// query.add_param("age", "30");
    /// let query_string = query.build();
    /// assert_eq!(query_string, "name=John Doe&age=30");
    /// ```
    pub fn build(&self) -> String {
        self.params
            .iter()
            .map(|(key, value)| format!("{key}={value}&"))
            .collect::<Vec<String>>()
            .join("");

        let mut query_string: String = String::new();

        // add params
        query_string.push_str(
            self.params
                .iter()
                .map(|(key, value)| format!("{key}={value}"))
                .collect::<Vec<String>>()
                .join("&")
                .as_str(),
        );

        if !self.filters.is_empty() {
            // add filters
            if !query_string.is_empty() {
                query_string.push('&');
            }
            query_string.push_str(
                self.filters
                    .iter()
                    .map(|filter| filter.to_string())
                    .collect::<Vec<String>>()
                    .join("&")
                    .as_str(),
            );
        }
        if !self.sorts.is_empty() {
            // add sorts
            if !query_string.is_empty() {
                query_string.push('&');
            }

            query_string.push_str(
                self.sorts
                    .iter()
                    .map(|sort| sort.to_string())
                    .collect::<Vec<String>>()
                    .join("&")
                    .as_str(),
            );
        }
        query_string
    }
}
