use std::io::BufReader;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut novel = novelscript::Novel::new();

    let file = std::fs::File::open("test.ns")?;
    novel.add_scene("test".into(), BufReader::new(file))?;

    let mut iter = novel.iter("test");

    iter.set_variable("number".into(), 0);
    iter.set_variable("another_number".into(), 1);
    iter.set_variable("another_another_number".into(), 0);

    while let Some(node) = iter.next() {
        match node {
            novelscript::SceneNodeData::Text { speaker, content } => {
                println!("{:?}: {}", speaker, content)
            }
            novelscript::SceneNodeData::Choice(choices) => {
                println!("{:?}", choices);
                iter.set_variable("choice".into(), 1);
            }
        }
    }

    Ok(())
}
