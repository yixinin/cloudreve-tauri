fn get_json() -> &str {
    let s = r#""#;
    return s;
}
#[test]
fn test_decode() {
    let s = get_json();
    let res = serde_json::from_str::<Ack<GetSharesAck>>(s);
    println!("{:?}", res);
}
