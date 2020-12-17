use pest::Parser;
use pest_derive::Parser;
use std::collections::HashMap;
use vec1::Vec1;

#[derive(Debug, Clone, PartialEq)]
pub enum SceneNodeData {
    Text {
        speaker: Option<String>,
        content: String,
    },
    Choice(Vec<String>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum SceneNodeLoad {
    Character {
        character: String,
        expression: String,
        placement: String,
    },
    Background {
        name: String,
    },
    PlaySound {
        name: String,
        channel: Option<String>,
    },
    RemoveCharacter {
        name: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Comparison {
    Equals,
    NotEquals,
    MoreThan,
    LessThan,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CompareableData {
    Variable(String),
    Number(i32),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Condition {
    first: CompareableData,
    compare: Comparison,
    second: CompareableData,
}

impl Condition {
    pub fn check(&self, map: &HashMap<String, i32>) -> bool {
        let first = match &self.first {
            CompareableData::Number(n) => Some(n),
            CompareableData::Variable(s) => map.get(s),
        };
        let second = match &self.second {
            CompareableData::Number(n) => Some(n),
            CompareableData::Variable(s) => map.get(s),
        };
        match (first, second) {
            (Some(first), Some(second)) => match self.compare {
                Comparison::Equals => first == second,
                Comparison::NotEquals => first != second,
                Comparison::MoreThan => first > second,
                Comparison::LessThan => first < second,
            },
            (None, None) => panic!(
                "No variables{:?} or {:?} are invalid",
                self.first, self.second
            ),
            (None, _) => panic!("No variable {:?} is invalid", self.first),
            (_, None) => panic!("No variable {:?} is invalid", self.second),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SceneNodeControl {
    If {
        cond: Condition,
        else_ifs: Vec<(Condition, Vec<SceneNode>)>,
        else_content: Option<Vec<SceneNode>>,
        content: Vec<SceneNode>,
    },
    Jump(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum SceneNodeUser {
    Data(SceneNodeData),
    Load(SceneNodeLoad),
}

#[derive(Debug, Clone, PartialEq)]
pub enum SceneNode {
    User(SceneNodeUser),
    Control(SceneNodeControl),
}

#[derive(Debug, Clone, Copy)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum Branch {
    First,
    Middle(usize),
    Last,
}

#[derive(Debug, Clone, Default)]
#[derive(serde::Serialize, serde::Deserialize)]
struct Scope {
    /// This is the index of the node that was next'd. it's None when nothing has been loaded.
    index: Option<usize>,
    choice: i32,
    branch: Option<Branch>,
}

impl Scope {
    pub fn inc(&mut self) {
        if let Some(idx) = &mut self.index {
            *idx += 1;
        } else {
            self.index = Some(0);
        }
    }
}

#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct NovelState {
    scene: String,
    variables: HashMap<String, i32>,
    scopes: Vec1<Scope>,
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
        self.scopes.last_mut().choice = choice;
    }
}

#[derive(Debug, Clone, Default)]
pub struct Novel {
    scenes: HashMap<String, Vec<SceneNode>>,
}

impl Novel {
    pub fn new() -> Self {
        Novel::default()
    }

    pub fn new_state(&self, starting_scene: &str) -> NovelState {
        NovelState {
            scene: starting_scene.to_owned(),
            variables: HashMap::new(),
            scopes: Vec1::new(Scope::default()),
        }
    }

    pub fn add_scene(&mut self, name: String, data: &str) {
        self.add_nodes(name, parse(data));
    }

    pub fn add_nodes(&mut self, name: String, data: Vec<SceneNode>) {
        self.scenes.insert(name, data);
    }

    pub fn next<'a>(&'a self, state: &mut NovelState) -> Option<&'a SceneNodeUser> {
        state.scopes.last_mut().inc();
        self.current(state)
    }

    pub fn current<'a>(&'a self, state: &mut NovelState) -> Option<&'a SceneNodeUser> {
        let active_node = {
            let active_scene = &self.scenes.get(&state.scene).unwrap_or_else(|| panic!("Couldn't find scene '{}'", state.scene));

            let mut prev_scope = &state.scopes[0];
            let mut active_node = active_scene.get(prev_scope.index.expect("Expected a scope index"));
            for scope in &state.scopes[1..] {
                if let Some(SceneNode::Control(SceneNodeControl::If {
                    cond: _,
                    content,
                    else_ifs,
                    else_content,
                })) = active_node
                {
                    if let Some(branch) = prev_scope.branch {
                        active_node = match branch {
                            Branch::First => content.get(scope.index.expect("Expected a scope index")),
                            Branch::Middle(n) => else_ifs
                                .get(n)
                                .map(|o| o.1.get(scope.index.expect("Expected a scope index")))
                                .flatten(),
                            Branch::Last => else_content
                                .as_ref()
                                .map(|c| c.get(scope.index.expect("Expected a scope index")))
                                .flatten(),
                        }
                    }
                }
                prev_scope = scope;
            }
            active_node
        };
        let node = match active_node {
            Some(node) => match node {
                SceneNode::User(node) => Some(node),
                SceneNode::Control(node) => match node {
                    SceneNodeControl::If {
                        cond,
                        content: _,
                        else_ifs,
                        else_content,
                    } => {
                        // Hacky fix for scoped choices
                        state
                            .variables
                            .insert("choice".into(), state.scopes.last().choice);
                        if cond.check(&state.variables) {
                            state.scopes.last_mut().branch = Some(Branch::First);
                            state.scopes.push(Scope::default())
                        } else if let Some((n, _)) = else_ifs
                            .iter()
                            .enumerate()
                            .find(|(_, (else_if_cond, _))| else_if_cond.check(&state.variables))
                        {
                            state.scopes.last_mut().branch = Some(Branch::Middle(n));
                            state.scopes.push(Scope::default())
                        } else if let Some(_) = else_content {
                            state.scopes.last_mut().branch = Some(Branch::Last);
                            state.scopes.push(Scope::default())
                        }

                        return self.next(state);
                    },
                    SceneNodeControl::Jump(target) => {
                        state.scopes = Vec1::new(Scope::default());
                        state.scene = target.clone();

                        return self.next(state);
                    }
                },
            },
            None => {
                if let Ok(..) = state.scopes.try_pop() {
                    state.scopes.last_mut().branch = None;
                    return self.next(state);
                } else {
                    None
                }
            }
        };
        node
    }
}

#[derive(Parser)]
#[grammar = "novelscript.pest"]
struct NovelscriptParser;

fn parse_if(mut pair_it: pest::iterators::Pairs<Rule>) -> (Condition, Vec<SceneNode>) {
    let condition = {
        let mut cond_it = pair_it.next().unwrap().into_inner();
        let first = cond_it.next().unwrap().as_str();
        let compare = match cond_it.next().unwrap().as_str() {
            "=" => Comparison::Equals,
            "!=" => Comparison::NotEquals,
            ">" => Comparison::MoreThan,
            "<" => Comparison::LessThan,
            c => panic!("{}", c),
        };
        let second = cond_it.next().unwrap().as_str();

        Condition {
            first: match first.parse() {
                Ok(n) => CompareableData::Number(n),
                Err(_) => CompareableData::Variable(first.to_owned()),
            },
            compare,
            second: match second.parse() {
                Ok(n) => CompareableData::Number(n),
                Err(_) => CompareableData::Variable(second.to_owned()),
            },
        }
    };
    let statement_list = pair_it
        .next()
        .unwrap()
        .into_inner()
        .map(|statement| parse_statement(statement.into_inner().next().unwrap()))
        .collect::<Vec<_>>();

    (condition, statement_list)
}

fn parse_statement<'a>(pair: pest::iterators::Pair<'a, Rule>) -> SceneNode {
    match pair.as_rule() {
        Rule::choice_statement => {
            let choices = pair
                .into_inner()
                .map(|choice| choice.as_str().trim().to_owned())
                .collect::<Vec<_>>();
            SceneNode::User(SceneNodeUser::Data(SceneNodeData::Choice(choices)))
        }
        Rule::if_statement => {
            let mut pairs_it = pair.into_inner();
            let (if_cond, if_content) = { parse_if(pairs_it.next().unwrap().into_inner()) };
            let mut else_content = None;
            let mut else_ifs = Vec::new();
            for case in pairs_it {
                match case.as_rule() {
                    Rule::else_if_case => {
                        let pair_it = case.into_inner().next().unwrap().into_inner();
                        else_ifs.push(parse_if(pair_it));
                    }
                    Rule::else_case => {
                        let statement_it = case.into_inner().next().unwrap().into_inner();
                        else_content = Some(
                            statement_it
                                .map(|statement| {
                                    parse_statement(statement.into_inner().next().unwrap())
                                })
                                .collect::<Vec<_>>(),
                        );
                    }
                    _ => unreachable!(),
                }
            }
            SceneNode::Control(SceneNodeControl::If {
                cond: if_cond,
                else_ifs,
                else_content,
                content: if_content,
            })
        }
        Rule::dialogue_statement => {
            let mut diag_it = pair.into_inner();
            let speaker = diag_it.next().unwrap().as_str().to_owned();
            let content = diag_it.next().unwrap().as_str().to_owned();
            SceneNode::User(SceneNodeUser::Data(SceneNodeData::Text {
                speaker: if speaker == "_" { None } else { Some(speaker) },
                content,
            }))
        }
        Rule::scene_statement => {
            let mut scene_it = pair.into_inner();
            let name = scene_it.next().unwrap().as_str().to_owned();
            SceneNode::User(SceneNodeUser::Load(SceneNodeLoad::Background { name }))
        }
        Rule::load_statement => {
            let mut load_it = pair.into_inner();
            let character = load_it.next().unwrap().as_str().to_owned();
            let expression = load_it.next().unwrap().as_str().to_owned();
            let placement = load_it.next().unwrap().as_str().to_owned();
            SceneNode::User(SceneNodeUser::Load(SceneNodeLoad::Character {
                character,
                expression,
                placement,
            }))
        }
        Rule::sound_statement => {
            let mut sound_it = pair.into_inner();
            let name = sound_it.next().unwrap().as_str().to_owned();
            let channel = sound_it.next().unwrap().as_str().to_owned();
            SceneNode::User(SceneNodeUser::Load(SceneNodeLoad::PlaySound {
                name,
                channel: Some(channel),
            }))
        }
        Rule::remove_statement => {
            let mut remove_it = pair.into_inner();
            let name = remove_it.next().unwrap().as_str().to_owned();
            SceneNode::User(SceneNodeUser::Load(SceneNodeLoad::RemoveCharacter { name }))
        },
        Rule::jump_statement => {
            let mut jump_it = pair.into_inner();
            let target = jump_it.next().unwrap().as_str().to_owned();
            SceneNode::Control(SceneNodeControl::Jump(target))
        },
        _ => unreachable!(),
    }
}

pub fn parse(data: &str) -> Vec<SceneNode> {
    let mut nodes = Vec::new();

    let parse = NovelscriptParser::parse(Rule::file, &data)
        .unwrap()
        .next()
        .unwrap();

    for line in parse.into_inner() {
        match line.as_rule() {
            Rule::statement => {
                nodes.push(parse_statement(line.into_inner().next().unwrap()));
            }
            _ => unreachable!(),
        }
    }

    nodes
}
