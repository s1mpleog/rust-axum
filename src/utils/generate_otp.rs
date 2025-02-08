pub fn create_otp() -> i32 {
    let otp = rand::random_range(100_000..=999_999);
    otp
}
