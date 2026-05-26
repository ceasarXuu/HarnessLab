pub const CRATE_PURPOSE: &str = "report rendering";

#[cfg(test)]
mod tests {
    use super::CRATE_PURPOSE;

    #[test]
    fn exposes_crate_purpose() {
        assert_eq!(CRATE_PURPOSE, "report rendering");
    }
}
