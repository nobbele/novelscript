use std::{collections::HashMap, io::BufRead, rc::Rc};

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
    If(parser::Condition, Rc<Vec<SceneNode>>),
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
    current: Rc<Vec<SceneNode>>,
    index: usize,
    parent: Option<Box<Scope>>,
    choice: i32,
}

#[derive(Debug, Clone)]
pub struct NovelState {
    variables: HashMap<String, i32>,
    scope: Scope,
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
            variables: HashMap::new(),
            scope: Scope::new(&Rc::new(self.scenes[starting_scene].clone())),
        }
    }

    pub fn next(&self, state: &mut NovelState) -> Option<SceneNodeData> {
        let node = match state.scope.current.get(state.scope.index).cloned() {
            Some(node) => match node {
                SceneNode::Data(node) => Some(node),
                SceneNode::Control(node) => match node {
                    SceneNodeControl::If(cond, content) => {
                        // Hacky fix for scoped choices
                        state.variables.insert("choice".into(), state.scope.choice);
                        println!("adding choice {}", state.scope.choice);
                        if cond.check(&state.variables) {
                            state.scope = Scope::with_parent(&content, state.scope.clone());
                        } else {
                            state.scope.index += 1;
                        }
                        return self.next(state);
                    }
                },
            },
            None => {
                if let Some(parent) = state.scope.parent.clone() {
                    state.scope = *parent;
                    state.scope.index += 1;
                    return self.next(state);
                } else {
                    None
                }
            }
        };
        state.scope.index += 1;
        node
    }
}

impl Scope {
    pub fn new(data: &Rc<Vec<SceneNode>>) -> Self {
        Scope {
            current: data.clone(),
            parent: None,
            index: 0,
            choice: 0,
        }
    }

    pub fn with_parent(data: &Rc<Vec<SceneNode>>, parent: Scope) -> Self {
        Scope {
            current: data.clone(),
            parent: Some(Box::new(parent)),
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
        self.scope.choice = choice; 
    }
}

fn parse(iter: &mut impl Iterator<Item = parser::Statement>) -> Vec<SceneNode> {
    let mut nodes = Vec::new();

    while let Some(statement) = iter.next() {
        nodes.push(match statement {
            parser::Statement::End => break,
            parser::Statement::If(cond) => {
                SceneNode::Control(SceneNodeControl::If(cond, Rc::new(parse(iter))))
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
