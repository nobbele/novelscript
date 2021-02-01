use novelscript;

fn setup(s: &str) -> Result<novelscript::Novel, Box<dyn std::error::Error>> {
    let mut novel = novelscript::Novel::new();
    novel.add_scene("test".into(), s);
    Ok(novel)
}

#[test]
fn test_text() -> Result<(), Box<dyn std::error::Error>> {
    let novel = setup(
        r#"

foo: test
_: test

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
fn test_special_text() -> Result<(), Box<dyn std::error::Error>> {
    let novel = setup(
        r#"

foo: "test"
_: test? what!
foo: hmm... what if, you say test

    "#,
    )?;
    let mut state = novel.new_state("test");

    assert_eq!(
        &novelscript::SceneNodeUser::Data(novelscript::SceneNodeData::Text {
            speaker: Some("foo".into()),
            content: "\"test\"".into(),
        }),
        novel.next(&mut state).unwrap()
    );

    assert_eq!(
        &novelscript::SceneNodeUser::Data(novelscript::SceneNodeData::Text {
            speaker: None,
            content: "test? what!".into(),
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
    _: first
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
    _: first
end

    "#,
    )?;
    let mut state = novel.new_state("test");

    state.set_variable("num".into(), 13);

    assert_eq!(None, novel.next(&mut state));

    Ok(())
}

#[test]
fn test_remove() -> Result<(), Box<dyn std::error::Error>> {
    let novel = setup(
        r#"

remove Foo

    "#,
    )?;
    let mut state = novel.new_state("test");

    state.set_variable("num".into(), 13);

    assert_eq!(
        &novelscript::SceneNodeUser::Load(novelscript::SceneNodeLoad::RemoveCharacter {
            name: "Foo".into()
        }),
        novel.next(&mut state).unwrap()
    );

    Ok(())
}

#[test]
fn test_nested_if() -> Result<(), Box<dyn std::error::Error>> {
    let novel = setup(
        r#"

if num = 13
    _: first
    if num2 = 17
        _: second
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
    _: first
end
if choice = 2
    _: second
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
    _: first
    [a / b]
end
if choice = 2
    _: second
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
load Bar {
    expression Normal
    placement Center
}
Bar: Hello Foo
scene Night
set Bar expression Cold
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
            expression: Some("Normal".into()),
            placement: Some("Center".into()),
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
        &novelscript::SceneNodeUser::Load(novelscript::SceneNodeLoad::Character {
            character: "Bar".into(),
            expression: Some("Cold".into()),
            placement: None,
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

play test on sfx
play noise on sfx
play relax on music
play test on music

    "#,
    )?;
    let mut state = novel.new_state("test");

    assert_eq!(
        &novelscript::SceneNodeUser::Load(novelscript::SceneNodeLoad::PlaySound {
            name: "test".into(),
            channel: "sfx".into(),
        }),
        novel.next(&mut state).unwrap()
    );

    assert_eq!(
        &novelscript::SceneNodeUser::Load(novelscript::SceneNodeLoad::PlaySound {
            name: "noise".into(),
            channel: "sfx".into(),
        }),
        novel.next(&mut state).unwrap()
    );

    assert_eq!(
        &novelscript::SceneNodeUser::Load(novelscript::SceneNodeLoad::PlaySound {
            name: "relax".into(),
            channel: "music".into(),
        }),
        novel.next(&mut state).unwrap()
    );

    assert_eq!(
        &novelscript::SceneNodeUser::Load(novelscript::SceneNodeLoad::PlaySound {
            name: "test".into(),
            channel: "music".into(),
        }),
        novel.next(&mut state).unwrap()
    );

    Ok(())
}

#[test]
fn test_jump() -> Result<(), Box<dyn std::error::Error>> {
    let mut novel = novelscript::Novel::new();
    novel.add_scene(
        "test".into(),
        r#"

foo: test
_: test
jump test2

    "#,
    );
    novel.add_scene(
        "test2".into(),
        r#"

foo: it is test2
_: indeed

    "#,
    );
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

    assert_eq!(
        &novelscript::SceneNodeUser::Data(novelscript::SceneNodeData::Text {
            speaker: Some("foo".into()),
            content: "it is test2".into(),
        }),
        novel.next(&mut state).unwrap()
    );

    assert_eq!(
        &novelscript::SceneNodeUser::Data(novelscript::SceneNodeData::Text {
            speaker: None,
            content: "indeed".into(),
        }),
        novel.next(&mut state).unwrap()
    );

    Ok(())
}
