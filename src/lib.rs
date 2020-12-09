use pest::Parser;
use pest_derive::Parser;
use std::collections::HashMap;

pub mod parser;

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
}

#[derive(Debug, Clone, PartialEq)]
pub enum Comparison {
    Equals,
    NotEquals,
    MoreThan,
    LessThan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Condition {
    first: String,
    compare: Comparison,
    second: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SceneNodeControl {
    If {
        cond: Condition,
        else_ifs: Vec<(Condition, Vec<SceneNode>)>,
        else_content: Option<Vec<SceneNode>>,
        content: Vec<SceneNode>,
    },
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

#[derive(Debug, Clone, Default)]
pub struct Novel {
    scenes: HashMap<String, Vec<SceneNode>>,
}

impl Novel {
    pub fn new() -> Self {
        Novel::default()
    }

    pub fn add_scene(&mut self, name: String, data: &str) -> Result<(), parser::ParseErrColl> {
        self.scenes.insert(name, parse(data));
        Ok(())
    }
}

/*#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
struct Scope {
    /// This is the index of the node that was next'd. it's None when nothing has been loaded.
    index: Option<usize>,
    choice: i32,
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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
        data: &str,
    ) -> Result<(), parser::ParseErrColl> {
        self.scenes
            .insert(name, parse(data));
        Ok(())
    }

    pub fn new_state(&self, starting_scene: &str) -> NovelState {
        NovelState {
            scene: starting_scene.to_owned(),
            variables: HashMap::new(),
            scopes: vec![Scope::new()],
        }
    }

    /// This gets current node and then increments the counter.
    /// Use `.dec()` to decrement the counter.
    pub fn next<'a>(&'a self, state: &mut NovelState) -> Option<&'a SceneNodeUser> {
        state.scopes.last_mut()?.inc();

        let active_scope = state.scopes.last()?;
        let mut active_node = self.scenes[&state.scene].get(state.scopes[0].index.unwrap());
        for scope in &state.scopes[1..] {
            if let Some(SceneNode::Control(SceneNodeControl::If(_, content))) =
                active_node.map(|n| n)
            {
                active_node = content.get(scope.index.unwrap())
            }
        }
        let node = match active_node {
            Some(node) => match node {
                SceneNode::User(node) => Some(node),
                SceneNode::Control(node) => match node {
                    SceneNodeControl::If(cond, _) => {
                        // Hacky fix for scoped choices
                        state.variables.insert("choice".into(), active_scope.choice);
                        if cond.check(&state.variables) {
                            state.scopes.push(Scope::new());
                        }
                        return self.next(state);
                    }
                },
            },
            None => {
                if state.scopes.len() > 0 {
                    state.scopes.remove(state.scopes.len() - 1);
                    state.scopes.last_mut().map(|n| n.inc());
                    return self.next(state);
                } else {
                    None
                }
            }
        };
        node
    }
}

impl Scope {
    pub fn new() -> Self {
        Scope {
            index: None,
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
}*/

#[derive(Parser)]
#[grammar = "novelscript.pest"]
struct NovelscriptParser;

fn parse_if(mut pair_it: pest::iterators::Pairs<Rule>) -> (Condition, Vec<SceneNode>) {
    let condition = {
        let mut cond_it = pair_it.next().unwrap().into_inner();
        let first = cond_it.next().unwrap().as_str().to_owned();
        let compare = match cond_it.next().unwrap().as_str() {
            "=" => Comparison::Equals,
            "!=" => Comparison::NotEquals,
            ">" => Comparison::MoreThan,
            "<" => Comparison::LessThan,
            c => panic!("{}", c),
        };
        let second = cond_it.next().unwrap().as_str().to_owned();
        Condition {
            first,
            compare,
            second,
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
                .map(|choice| choice.as_str().to_owned())
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
        Rule::dialogue => {
            let mut diag_it = pair.into_inner();
            let (speaker, content) = match (diag_it.next(), diag_it.next()) {
                (Some(name), Some(text)) => {
                    (Some(name.as_str().to_owned()), text.as_str().to_owned())
                }
                (Some(text), None) => (None, text.as_str().to_owned()),
                _ => unreachable!(),
            };
            SceneNode::User(SceneNodeUser::Data(SceneNodeData::Text {
                speaker,
                content,
            }))
        }
        _ => unreachable!(),
    }
}

fn parse(data: &str) -> Vec<SceneNode> {
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
