use twee_v3::Story;

const SAMPLE: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/sample/sample.twee"));

fn main() {
    let story = Story::try_from(SAMPLE).unwrap();
    let title = story.title().unwrap();
    let start = story.start().unwrap();
    println!("Let's tell the story [{title}]");

    println!("It all starts with {start:?}");
}
