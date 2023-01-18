use twee_v3::parse_tags;

fn main() {
    let tag_string = r"[hel\\lo w\o\\rld no-copy]";

    let (_, tags) = parse_tags(tag_string).unwrap();

    let tag_names = tags.iter().map(|tag| tag.as_ref()).collect::<Vec<_>>();
    println!("{}", tag_names.join(", "));
    println!("{tags:?}");
}
