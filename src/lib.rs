mod metadata;
pub mod passage;
mod utils;

#[cfg(test)]
mod tests {
    use crate::passage::parse_passage;

    const SAMPLE: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/sample/sample.twee"));

    #[test]
    fn test_full_story() {
        let input = SAMPLE;

        println!("{:?}", parse_passage(input));
    }
}
