pub const CRATE_PURPOSE: &str = "infrastructure adapters";

#[cfg(test)]
mod tests {
    use super::CRATE_PURPOSE;

    #[test]
    fn exposes_crate_purpose() {
        assert_eq!(CRATE_PURPOSE, "infrastructure adapters");
    }
}
