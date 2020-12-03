use std::io::BufReader;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut novel = novelscript::Novel::new();

    let file = std::fs::File::open("test.ns")?;
    novel.add_scene("test".into(), BufReader::new(file))?;

    let mut state = novel.new_state("test");

    state.set_variable("number".into(), 0);
    state.set_variable("another_number".into(), 1);
    state.set_variable("another_another_number".into(), 0);

    while let Some(node) = novel.next(&mut state) {
        match node {
            novelscript::SceneNodeUser::Data(node) => match node {
                novelscript::SceneNodeData::Text { speaker, content } => {
                    println!("{}: {}", speaker.as_ref().unwrap_or(&"*".into()), content)
                }
                novelscript::SceneNodeData::Choice(choices) => {
                    println!("{:?}", choices);
                    state.set_choice(1);
                },
                novelscript::SceneNodeData::PlaySound {name, channel} => println!("Playing {} on {:?}", name, channel),
            },
            novelscript::SceneNodeUser::Load(node) => match node {
                novelscript::SceneNodeLoad::Character {
                    character,
                    expression,
                    placement,
                } => println!(
                    "Load {} with {} expression at {}",
                    character, expression, placement
                ),
                novelscript::SceneNodeLoad::Background { name } => {
                    println!("Load background {}", name)
                }
            },
        }
    }

    Ok(())
}
