#[cfg(test)]
mod tests {
    use focus::tui::app::truncate_to;

    // T010/T011: truncate_to tests

    #[test]
    fn truncate_to_empty_string() {
        assert_eq!(truncate_to("", 5), "");
    }

    #[test]
    fn truncate_to_shorter_than_max() {
        assert_eq!(truncate_to("hello", 10), "hello");
    }

    #[test]
    fn truncate_to_exact_length() {
        assert_eq!(truncate_to("hello", 5), "hello");
    }

    #[test]
    fn truncate_to_longer_than_max() {
        let result = truncate_to("hello world", 8);
        // Should be 7 chars + ellipsis (1 char) = 8 chars total
        assert_eq!(result.chars().count(), 8);
        assert!(result.ends_with('…'));
        assert_eq!(result, "hello w…");
    }

    #[test]
    fn truncate_to_max_one() {
        // Max 1 char: take 0 chars + ellipsis
        let result = truncate_to("hello", 1);
        assert_eq!(result, "…");
    }

    #[test]
    fn truncate_to_max_zero() {
        assert_eq!(truncate_to("hello", 0), "");
    }

    #[test]
    fn truncate_to_unicode_multibyte() {
        // "café" has 4 Unicode chars
        assert_eq!(truncate_to("café", 4), "café");
        // Truncate to 3: "ca" + "…" = 3 chars
        let result = truncate_to("café", 3);
        assert_eq!(result.chars().count(), 3);
        assert!(result.ends_with('…'));
    }

    #[test]
    fn truncate_to_unicode_emoji() {
        // Each emoji is 1 Unicode scalar value
        let s = "hello🎉world";
        let result = truncate_to(s, 6);
        assert_eq!(result.chars().count(), 6);
        assert!(result.ends_with('…'));
    }
}
