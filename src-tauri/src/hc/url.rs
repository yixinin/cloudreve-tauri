use serde::Serialize;

pub fn get_api_query_url<T>(addr: &str, path: &str, query: Option<T>) -> String
where
    T: Serialize,
{
    if let Some(query) = query {
        let query_str = serde_urlencoded::to_string(query).unwrap_or_default();
        return format!("{}/api/v4{}?{}", addr, path, query_str);
    }
    return format!("{}/api/v4{}", addr, path);
}

pub fn get_api_url(addr: &str, path: &str) -> String {
    return get_api_query_url::<String>(addr, path, None);
}
