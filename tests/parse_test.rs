use novelscript;
use std::io::BufReader;

fn setup(s: &str) -> Result<novelscript::Novel, Box<dyn std::error::Error>> {
    let mut novel = novelscript::Novel::new();
    novel.add_scene("test".into(), BufReader::new(s.as_bytes()))?;
    Ok(novel)
}

#[test]
fn test_text() -> Result<(), Box<dyn std::error::Error>> {
    let novel = setup(
        r#"
    
foo: test
: test

    "#,
    )?;
    let mut state = novel.new_state("test");

    assert_eq!(
        &novelscript::SceneNodeUser::Data(novelscript::SceneNodeData::Text {
            speaker: Some("foo".into()),
            content: "test".into(),
        }),
        novel.next(&mut state).unwrap()
    );

    assert_eq!(
        &novelscript::SceneNodeUser::Data(novelscript::SceneNodeData::Text {
            speaker: None,
            content: "test".into(),
        }),
        novel.next(&mut state).unwrap()
    );

    Ok(())
}

#[test]
fn test_if() -> Result<(), Box<dyn std::error::Error>> {
    let novel = setup(
        r#"
    
if num = 13
    : first
end

    "#,
    )?;
    let mut state = novel.new_state("test");

    state.set_variable("num".into(), 13);

    assert_eq!(
        &novelscript::SceneNodeUser::Data(novelscript::SceneNodeData::Text {
            speaker: None,
            content: "first".into()
        }),
        novel.next(&mut state).unwrap()
    );
    assert_eq!(None, novel.next(&mut state));

    Ok(())
}

#[test]
fn test_negative_if() -> Result<(), Box<dyn std::error::Error>> {
    let novel = setup(
        r#"
    
if num = 17
    : first
end

    "#,
    )?;
    let mut state = novel.new_state("test");

    state.set_variable("num".into(), 13);

    assert_eq!(None, novel.next(&mut state));

    Ok(())
}

#[test]
fn test_nested_if() -> Result<(), Box<dyn std::error::Error>> {
    let novel = setup(
        r#"
    
if num = 13
    : first
    if num2 = 17
        : second
    end
end

    "#,
    )?;
    let mut state = novel.new_state("test");

    state.set_variable("num".into(), 13);
    state.set_variable("num2".into(), 17);

    assert_eq!(
        &novelscript::SceneNodeUser::Data(novelscript::SceneNodeData::Text {
            speaker: None,
            content: "first".into()
        }),
        novel.next(&mut state).unwrap()
    );
    assert_eq!(
        &novelscript::SceneNodeUser::Data(novelscript::SceneNodeData::Text {
            speaker: None,
            content: "second".into()
        }),
        novel.next(&mut state).unwrap()
    );
    assert_eq!(None, novel.next(&mut state));

    Ok(())
}

#[test]
fn test_choice() -> Result<(), Box<dyn std::error::Error>> {
    let novel = setup(
        r#"
    
[x / y]
if choice = 1
    : first
end
if choice = 2
    : second
end

    "#,
    )?;
    let mut state = novel.new_state("test");

    assert_eq!(
        &novelscript::SceneNodeUser::Data(novelscript::SceneNodeData::Choice(vec![
            "x".into(),
            "y".into()
        ])),
        novel.next(&mut state).unwrap()
    );
    state.set_choice(1);
    assert_eq!(
        &novelscript::SceneNodeUser::Data(novelscript::SceneNodeData::Text {
            speaker: None,
            content: "first".into()
        }),
        novel.next(&mut state).unwrap()
    );
    assert_eq!(None, novel.next(&mut state));

    Ok(())
}

#[test]
fn test_nested_choices() -> Result<(), Box<dyn std::error::Error>> {
    let novel = setup(
        r#"
    
[x / y]
if choice = 1
    : first
    [a / b]
end
if choice = 2
    : second
end

    "#,
    )?;
    let mut state = novel.new_state("test");

    assert_eq!(
        &novelscript::SceneNodeUser::Data(novelscript::SceneNodeData::Choice(vec![
            "x".into(),
            "y".into()
        ])),
        novel.next(&mut state).unwrap()
    );
    state.set_choice(1);
    assert_eq!(
        &novelscript::SceneNodeUser::Data(novelscript::SceneNodeData::Text {
            speaker: None,
            content: "first".into()
        }),
        novel.next(&mut state).unwrap()
    );
    assert_eq!(
        &novelscript::SceneNodeUser::Data(novelscript::SceneNodeData::Choice(vec![
            "a".into(),
            "b".into()
        ])),
        novel.next(&mut state).unwrap()
    );
    state.set_choice(1);
    assert_eq!(None, novel.next(&mut state));

    Ok(())
}

#[test]
fn test_load_character_and_background() -> Result<(), Box<dyn std::error::Error>> {
    let novel = setup(
        r#"
    
Foo: Hello Bar
load Bar Normal at Center
Bar: Hello Foo
scene Night
Foo: It is now night

    "#,
    )?;
    let mut state = novel.new_state("test");

    assert_eq!(
        &novelscript::SceneNodeUser::Data(novelscript::SceneNodeData::Text {
            speaker: Some("Foo".into()),
            content: "Hello Bar".into(),
        }),
        novel.next(&mut state).unwrap()
    );
    assert_eq!(
        &novelscript::SceneNodeUser::Load(novelscript::SceneNodeLoad::Character {
            character: "Bar".into(),
            expression: "Normal".into(),
            placement: "Center".into(),
        }),
        novel.next(&mut state).unwrap()
    );
    assert_eq!(
        &novelscript::SceneNodeUser::Data(novelscript::SceneNodeData::Text {
            speaker: Some("Bar".into()),
            content: "Hello Foo".into(),
        }),
        novel.next(&mut state).unwrap()
    );
    assert_eq!(
        &novelscript::SceneNodeUser::Load(novelscript::SceneNodeLoad::Background {
            name: "Night".into(),
        }),
        novel.next(&mut state).unwrap()
    );
    assert_eq!(
        &novelscript::SceneNodeUser::Data(novelscript::SceneNodeData::Text {
            speaker: Some("Foo".into()),
            content: "It is now night".into(),
        }),
        novel.next(&mut state).unwrap()
    );

    Ok(())
}

#[test]
fn test_sound() -> Result<(), Box<dyn std::error::Error>> {
    let novel = setup(
        r#"
    
play test
play noise sfx
play relax music

    "#,
    )?;
    let mut state = novel.new_state("test");

    assert_eq!(
        &novelscript::SceneNodeUser::Data(novelscript::SceneNodeData::PlaySound {
            name: "test".into(),
            channel: None,
        }),
        novel.next(&mut state).unwrap()
    );

    assert_eq!(
        &novelscript::SceneNodeUser::Data(novelscript::SceneNodeData::PlaySound {
            name: "noise".into(),
            channel: Some("sfx".into()),
        }),
        novel.next(&mut state).unwrap()
    );

    assert_eq!(
        &novelscript::SceneNodeUser::Data(novelscript::SceneNodeData::PlaySound {
            name: "relax".into(),
            channel: Some("music".into()),
        }),
        novel.next(&mut state).unwrap()
    );

    Ok(())
}