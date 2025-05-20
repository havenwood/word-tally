#[cfg(test)]
mod processing_tests {
    use word_tally::Processing;

    #[test]
    fn test_processing_display() {
        assert_eq!(Processing::Sequential.to_string(), "sequential");
        assert_eq!(Processing::Parallel.to_string(), "parallel");
    }

    #[test]
    fn test_processing_default() {
        assert_eq!(Processing::default(), Processing::Sequential);
    }

    #[test]
    fn test_processing_traits() {
        use std::collections::HashSet;

        // Test Debug trait
        assert_eq!(format!("{:?}", Processing::Sequential), "Sequential");
        assert_eq!(format!("{:?}", Processing::Parallel), "Parallel");

        // Test Clone trait
        let processing = Processing::Parallel;
        let cloned = processing;
        assert_eq!(processing, cloned);

        // Test Copy trait (implicitly by not moving)
        let processing = Processing::Sequential;
        let copy1 = processing;
        let copy2 = processing;
        assert_eq!(copy1, copy2);

        // Test Ord trait
        assert!(Processing::Sequential < Processing::Parallel);

        let mut set = HashSet::new();
        set.insert(Processing::Sequential);
        set.insert(Processing::Parallel);
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_processing_serde() {
        use serde_json;

        // Test serialization
        let sequential = Processing::Sequential;
        let json = serde_json::to_string(&sequential).expect("serialize JSON");
        assert_eq!(json, "\"Sequential\"");

        let parallel = Processing::Parallel;
        let json = serde_json::to_string(&parallel).expect("serialize JSON");
        assert_eq!(json, "\"Parallel\"");

        // Test deserialization
        let deserialized: Processing =
            serde_json::from_str("\"Sequential\"").expect("deserialize JSON");
        assert_eq!(deserialized, Processing::Sequential);

        let deserialized: Processing =
            serde_json::from_str("\"Parallel\"").expect("deserialize JSON");
        assert_eq!(deserialized, Processing::Parallel);
    }
}
