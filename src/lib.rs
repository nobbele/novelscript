use std::{collections::HashMap, io::BufRead};

pub mod parser;

#[derive(Debug, Clone, PartialEq)]
pub enum SceneNodeData {
    Text {
        speaker: Option<String>,
        content: String,
    },
    Choice(Vec<String>),
    LoadCharacter {
        character: String,
        expression: String,
        placement: String,
    },
    LoadBackground {
        name: String
    },
}

#[derive(Debug, Clone)]
pub enum SceneNodeControl {
    If(parser::Condition, Vec<SceneNode>),
}

#[derive(Debug, Clone)]
pub enum SceneNode {
    Data(SceneNodeData),
    Control(SceneNodeControl),
}

#[derive(Debug, Clone, Default)]
pub struct Novel {
    scenes: HashMap<String, Vec<SceneNode>>,
}

#[derive(Debug, Clone)]
struct Scope {
    index: usize,
    choice: i32,
}

#[derive(Debug, Clone)]
pub struct NovelState {
    scene: String,
    variables: HashMap<String, i32>,
    scopes: Vec<Scope>,
}

impl Novel {
    pub fn new() -> Self {
        Novel::default()
    }

    pub fn add_scene(
        &mut self,
        name: String,
        reader: impl BufRead,
    ) -> Result<(), parser::ParseErrColl> {
        self.scenes
            .insert(name, parse(&mut parser::parse(reader)?.into_iter()));
        Ok(())
    }

    pub fn new_state(&self, starting_scene: &str) -> NovelState {
        NovelState {
            scene: starting_scene.to_owned(),
            variables: HashMap::new(),
            scopes: vec![Scope::new()],
        }
    }

    pub fn next(&self, state: &mut NovelState) -> Option<SceneNodeData> {
        println!("{:?}", state.scopes);
        let active_scope = state.scopes.last()?;
        let mut active_node = self.scenes[&state.scene].get(state.scopes[0].index);
        for scope in &state.scopes[1..] {
            if let Some(SceneNode::Control(SceneNodeControl::If(_, content))) = active_node.map(| n| n) {
                active_node = content.get(scope.index)
            }
        }
        println!("{:?}", active_node);
        let node = match active_node.cloned() {
            Some(node) => match node {
                SceneNode::Data(node) => Some(node),
                SceneNode::Control(node) => match node {
                    SceneNodeControl::If(cond, _) => {
                        // Hacky fix for scoped choices
                        state.variables.insert("choice".into(), active_scope.choice);
                        if cond.check(&state.variables) {
                            state.scopes.push(Scope::new());
                        } else {
                            state.scopes.last_mut().unwrap().index += 1;
                        }
                        return self.next(state);
                    }
                },
            },
            None => {
                if state.scopes.len() > 0 {
                    state.scopes.remove(state.scopes.len() - 1);
                    state.scopes.last_mut().map(|n| n.index += 1);
                    return self.next(state);
                } else {
                    None
                }
            }
        };
        state.scopes.last_mut().unwrap().index += 1;
        node
    }
}

impl Scope {
    pub fn new() -> Self {
        Scope {
            index: 0,
            choice: 0,
        }
    }
}

impl NovelState {
    pub fn set_variable(&mut self, name: String, data: i32) {
        if name.as_str() == "choice" {
            panic!("Don't use set choice with set_variable, use set_choice instead");
        }
        self.variables.insert(name, data);
    }

    pub fn set_choice(&mut self, choice: i32) {
        println!("set choice to {}", choice);
        self.scopes.last_mut().unwrap().choice = choice; 
    }
}

fn parse(iter: &mut impl Iterator<Item = parser::Statement>) -> Vec<SceneNode> {
    let mut nodes = Vec::new();

    while let Some(statement) = iter.next() {
        nodes.push(match statement {
            parser::Statement::End => break,
            parser::Statement::If(cond) => {
                SceneNode::Control(SceneNodeControl::If(cond, parse(iter)))
            }
            parser::Statement::Else => panic!("Else is currently unsupported"),
            parser::Statement::Choice(choices) => SceneNode::Data(SceneNodeData::Choice(choices)),
            parser::Statement::LoadCharacter {
                character,
                expression,
                placement,
            } => SceneNode::Data(SceneNodeData::LoadCharacter {
                character,
                expression,
                placement,
            }),
            parser::Statement::LoadBackground {
                name
            } => SceneNode::Data(SceneNodeData::LoadBackground {
                name,
            }),
            parser::Statement::Text { speaker, content } => {
                SceneNode::Data(SceneNodeData::Text { speaker, content })
            }
        });
    }
    nodes
}
