use std::io::BufReader;
use novelscript;

fn setup(s: &str) -> Result<novelscript::Novel, Box<dyn std::error::Error>> {
    let mut novel = novelscript::Novel::new();
    novel.add_scene("test".into(), BufReader::new(s.as_bytes()))?;
    Ok(novel)
}

#[test]
fn test_text() -> Result<(), Box<dyn std::error::Error>> {
    let novel = setup(r#"
    
foo: test
: test

    "#)?;
    let it = novel.iter("test");

    let expected =vec![
        novelscript::SceneNodeData::Text {
            speaker: Some("foo".into()),
            content: "test".into(),
        },
        novelscript::SceneNodeData::Text {
            speaker: None,
            content: "test".into(),
        }
    ];

    assert_eq!(expected, it.cloned().collect::<Vec<_>>());
    Ok(())
}

#[test]
fn test_if() -> Result<(), Box<dyn std::error::Error>> {
    let novel = setup(r#"
    
if num = 13
    : first
end

    "#)?;
    let mut it = novel.iter("test");

    it.set_variable("num".into(), 13);

    assert_eq!(novelscript::SceneNodeData::Text { speaker: None, content: "first".into() }, it.next().unwrap().clone());
    assert_eq!(None, it.next().clone());

    Ok(())
}

#[test]
fn test_negative_if() -> Result<(), Box<dyn std::error::Error>> {
    let novel = setup(r#"
    
if num = 17
    : first
end

    "#)?;
    let mut it = novel.iter("test");

    it.set_variable("num".into(), 13);

    assert_eq!(None, it.next().clone());

    Ok(())
}

#[test]
fn test_nested_if() -> Result<(), Box<dyn std::error::Error>> {
    let novel = setup(r#"
    
if num = 13
    : first
    if num2 = 17
        : second
    end
end

    "#)?;
    let mut it = novel.iter("test");

    it.set_variable("num".into(), 13);
    it.set_variable("num2".into(), 17);

    assert_eq!(novelscript::SceneNodeData::Text { speaker: None, content: "first".into() }, it.next().unwrap().clone());
    assert_eq!(novelscript::SceneNodeData::Text { speaker: None, content: "second".into() }, it.next().unwrap().clone());
    assert_eq!(None, it.next().clone());

    Ok(())
}


#[test]
fn test_choice() -> Result<(), Box<dyn std::error::Error>> {
    let novel = setup(r#"
    
[x / y]
if choice = 1
    : first
end
if choice = 2
    : second
end

    "#)?;
    let mut it = novel.iter("test");

    assert_eq!(novelscript::SceneNodeData::Choice(vec![ "x".into(), "y".into() ]), it.next().unwrap().clone());
    it.set_variable("choice".into(), 1);
    assert_eq!(novelscript::SceneNodeData::Text { speaker: None, content: "first".into() }, it.next().unwrap().clone());
    assert_eq!(None, it.next().clone());

    Ok(())
}

#[test]
fn test_nested_choices() -> Result<(), Box<dyn std::error::Error>> {
    let novel = setup(r#"
    
[x / y]
if choice = 1
    : first
    [a / b]
end
if choice = 2
    : second
end

    "#)?;
    let mut it = novel.iter("test");

    assert_eq!(novelscript::SceneNodeData::Choice(vec![ "x".into(), "y".into() ]), it.next().unwrap().clone());
    it.set_variable("choice".into(), 1);
    assert_eq!(novelscript::SceneNodeData::Text { speaker: None, content: "first".into() }, it.next().unwrap().clone());
    assert_eq!(novelscript::SceneNodeData::Choice(vec![ "a".into(), "b".into() ]), it.next().unwrap().clone());
    it.set_variable("choice".into(), 2);
    assert_eq!(None, it.next().clone());

    Ok(())
}