#![cfg(feature = "serde")]
#![cfg(feature = "serde_json")]

use std::io::BufReader;

#[test]
fn test_save_load() {
    let serialized = {

        let mut novel = novelscript::Novel::new();
        let s = r#"
        
            foo: test
            : test
            Bar: test

        "#;
        novel.add_scene("test".into(), BufReader::new(s.as_bytes())).unwrap();

        let mut state = novel.new_state("test");

        assert_eq!(
            novelscript::SceneNodeData::Text {
                speaker: Some("foo".into()),
                content: "test".into(),
            },
            novel.next(&mut state).unwrap()
        );

        assert_eq!(
            novelscript::SceneNodeData::Text {
                speaker: None,
                content: "test".into(),
            },
            novel.next(&mut state).unwrap()
        );

        serde_json::to_string(&state).unwrap()
    };

    let mut state = serde_json::from_str(&serialized).unwrap();

    let mut novel = novelscript::Novel::new();
    let s = r#"
    
        foo: test
        : test
        Bar: test

    "#;
    novel.add_scene("test".into(), BufReader::new(s.as_bytes())).unwrap();

    assert_eq!(
        novelscript::SceneNodeData::Text {
            speaker: Some("Bar".into()),
            content: "test".into(),
        },
        novel.next(&mut state).unwrap()
    );
}