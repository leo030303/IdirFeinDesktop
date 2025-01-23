fn get_auth_details() -> (String, String) {
    // TODO get from config
    (String::from("Leo"), String::from("12345"))
}

pub fn get_auth_pair() -> (String, String) {
    // TODO calculate properly
    let (client_id, totp_seed) = get_auth_details();
    (client_id, totp_seed)
}

pub fn calculate_totp(_client_secret: &[u8]) -> String {
    // TODO calculate properly
    String::from("12345")
}
