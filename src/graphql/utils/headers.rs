use crate::SupabaseClient;
use std::collections::HashMap;

/// ### Headers for GraphQL requests
/// This will populate a headers HashMap with the required headers for a GraphQL request
///
/// #### Arguments
/// * `client` - The Supabase client
///
///
/// #### Example
/// ```rust,ignore
/// use supabase::SupabaseClient;
/// use supabase::graphql::utils::headers::headers;
///
/// let client = SupabaseClient::new("https://myapp.supabase.co", "anonkey");
///
/// let headers = headers(&client);
///
/// println!("{:#?}", headers);
///
/// >> {
/// >>  "apiKey": "YOUR_SUPABASE_KEY",
/// >>  "Content-Type": "application/json"
/// >> }
/// ```
///
pub fn headers(client: &SupabaseClient) -> HashMap<String, String> {
    // init the header hashmap
    let mut headers: HashMap<String, String> = HashMap::new();

    // insert the headers
    headers.insert("apiKey".to_owned(), client.api_key.clone());
    headers.insert("Content-Type".to_owned(), "application/json".to_owned());

    // return the headers
    headers
}
