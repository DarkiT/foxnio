//! Tests for promo code service

#[cfg(test)]
mod tests {
    use foxnio::entity::promo_codes::PromoCodeStatus;
    use chrono::Utc;

    #[test]
    fn test_promo_code_status() {
        assert_eq!(PromoCodeStatus::Active.as_str(), "active");
        assert_eq!(PromoCodeStatus::Disabled.as_str(), "disabled");

        assert_eq!(PromoCodeStatus::parse("active"), PromoCodeStatus::Active);
        assert_eq!(PromoCodeStatus::parse("disabled"), PromoCodeStatus::Disabled);
        assert_eq!(PromoCodeStatus::parse("unknown"), PromoCodeStatus::Active);
    }
}
