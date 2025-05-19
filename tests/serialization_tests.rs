#[cfg(test)]
mod serialization_tests {
    use word_tally::{Format, Serialization};

    #[test]
    fn test_serialization_new_text() {
        let serialization = Serialization::new(Format::Text, " ").expect("create serialization");
        assert_eq!(serialization.format(), Format::Text);
        assert_eq!(serialization.delimiter(), " ");
    }

    #[test]
    fn test_serialization_new_json() {
        let serialization = Serialization::new(Format::Json, ",").expect("create serialization");
        assert_eq!(serialization.format(), Format::Json);
        assert_eq!(serialization.delimiter(), ",");
    }

    #[test]
    fn test_serialization_new_csv() {
        let serialization = Serialization::new(Format::Csv, ",").expect("create serialization");
        assert_eq!(serialization.format(), Format::Csv);
        assert_eq!(serialization.delimiter(), ",");
    }

    #[test]
    fn test_serialization_with_format() {
        let serialization = Serialization::with_format(Format::Json);
        assert_eq!(serialization.format(), Format::Json);
    }

    #[test]
    fn test_serialization_with_delimiter() {
        let serialization = Serialization::with_delimiter("\t").expect("create delimiter");
        assert_eq!(serialization.delimiter(), "\t");
    }

    #[test]
    fn test_serialization_with_format_setting() {
        let serialization = Serialization::default().with_format_setting(Format::Csv);
        assert_eq!(serialization.format(), Format::Csv);
    }

    #[test]
    fn test_serialization_default() {
        let serialization = Serialization::default();
        assert_eq!(serialization.format(), Format::Text);
        assert_eq!(serialization.delimiter(), " ");
    }

    #[test]
    fn test_serialization_builder_chain() {
        let serialization = Serialization::default().with_format_setting(Format::Csv);
        assert_eq!(serialization.format(), Format::Csv);
    }
}
