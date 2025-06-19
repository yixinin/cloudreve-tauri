use std::collections::HashMap;

use reqwest::Request;

pub struct Client {
    h3_pool: HashMap<String, Request::Client>,
}
