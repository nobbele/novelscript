use novelscript;

fn setup(s: &str) -> Result<novelscript::Novel, Box<dyn std::error::Error>> {
    let mut novel = novelscript::Novel::new();
    novel.add_scene("test".into(), s)?;
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
