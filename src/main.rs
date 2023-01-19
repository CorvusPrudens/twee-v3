use std::time::Instant;

use tweep::Story;

const SAMPLE: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/sample/sample.twee"));

fn main() {
    let start = Instant::now();
    let (story, _) = Story::from_string(SAMPLE.to_string()).take();
    let duration = start.elapsed();

    println!("Time elapsed in tweep() is: {:?}", duration);

    if let Ok(story) = story {
        println!("{:?}", story.title);
        let content = &story.passages["Start"].content;
        println!("Start: {}", content.content);
    }

    let start = Instant::now();
    let _story = twee_v3::story::Story::try_from(SAMPLE).unwrap();
    let duration = start.elapsed();

    println!("Time elapsed in twee-v3() is: {:?}", duration);
}
