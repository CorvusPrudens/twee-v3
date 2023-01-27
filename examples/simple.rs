use twee_v3::{ContentNode, Story};

const SAMPLE: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/sample/sample.twee"));

fn main() {
    let story = Story::try_from(SAMPLE).unwrap();
    let title = story.title().unwrap();
    let start = story.start().unwrap();

    println!("Let's tell the story [{title}]");
    for (count, node) in start.nodes().iter().enumerate() {
        match node {
            ContentNode::Text(text) => print!("{text}"),
            ContentNode::Link { text, target: _ } => {
                print!("{emoji} {text}", emoji = number_to_emoji(count));
            }
        }
    }
    println!();

    println!("{start}");

    for link in start.links() {
        let passage = story.get_passage(link.target);

        println!(
            "Does passage {title} exists? {exists}",
            title = link.target,
            exists = passage.is_some()
        );
    }
}

fn number_to_emoji(number: usize) -> &'static str {
    match number {
        0 => "0️⃣",
        1 => "1️⃣",
        2 => "2️⃣",
        3 => "3️⃣",
        4 => "4️⃣",
        5 => "5️⃣",
        _ => unreachable!(),
    }
}
