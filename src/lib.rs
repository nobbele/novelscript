use std::{collections::HashMap, io::BufRead};

pub mod parser;

#[derive(Debug, Clone, PartialEq)]
pub enum SceneNodeData {
    Text {
        speaker: Option<String>,
        content: String,
    },
    Choice(Vec<String>),
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

    pub fn iter<'a>(&'a self, starting_scene: &str) -> NovelIterator<'a> {
        NovelIterator {
            variables: HashMap::new(),
            scope: Scope::new(&self.scenes[starting_scene]),
        }
    }
}

#[derive(Debug, Clone)]
struct Scope<'a> {
    current: &'a Vec<SceneNode>,
    index: usize,
    parent: Option<Box<Scope<'a>>>,
}

impl<'a> Scope<'a> {
    pub fn new(data: &'a Vec<SceneNode>) -> Self {
        Scope {
            current: data,
            parent: None,
            index: 0,
        }
    }

    pub fn with_parent(data: &'a Vec<SceneNode>, parent: Scope<'a>) -> Self {
        Scope {
            current: data,
            parent: Some(Box::new(parent)),
            index: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct NovelIterator<'a> {
    variables: HashMap<String, i32>,
    scope: Scope<'a>,
}

impl<'a> std::iter::Iterator for NovelIterator<'a> {
    type Item = &'a SceneNodeData;
    fn next(&mut self) -> Option<&'a SceneNodeData> {
        let node = match self.scope.current.get(self.scope.index) {
            Some(node) => match node {
                SceneNode::Data(node) => Some(node),
                SceneNode::Control(node) => match node {
                    SceneNodeControl::If(cond, content) => {
                        if cond.check(&self.variables) {
                            self.scope = Scope::with_parent(&content, self.scope.clone());
                        } else {
                            self.scope.index += 1;
                        }
                        return self.next();
                    }
                },
            },
            None => {
                if let Some(parent) = self.scope.parent.clone() {
                    self.scope = *parent;
                    self.scope.index += 1;
                    return self.next();
                } else {
                    None
                }
            }
        };
        self.scope.index += 1;
        node
    }
}

impl<'a> NovelIterator<'a> {
    pub fn set_variable(&mut self, name: String, data: i32) {
        self.variables.insert(name, data);
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
            parser::Statement::Text { speaker, content } => SceneNode::Data(SceneNodeData::Text {
                speaker: speaker,
                content: content,
            }),
        });
    }
    nodes
}
