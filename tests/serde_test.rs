#[test]
fn test_save_load() {
    let s = r#"
        
            foo: test
            _: test
            bar: test

        "#;
    let serialized = {
        let mut novel = novelscript::Novel::new();

        novel.add_scene("test".into(), s);

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

        serde_json::to_string(&state).unwrap()
    };

    let mut state = serde_json::from_str(&serialized).unwrap();

    let mut novel = novelscript::Novel::new();

    novel.add_scene("test".into(), s);

    assert_eq!(
        &novelscript::SceneNodeUser::Data(novelscript::SceneNodeData::Text {
            speaker: Some("bar".into()),
            content: "test".into(),
        }),
        novel.next(&mut state).unwrap()
    );
}
