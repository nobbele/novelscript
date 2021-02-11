use pest::Parser;
use pest_derive::Parser;
//use petgraph::{graph::NodeIndex, Graph};
use std::{
    collections::HashMap,
    ops::Range,
};
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
        expression: Option<String>,
        placement: Option<String>,
    },
    Background {
        name: String,
    },
    PlaySound {
        name: String,
        channel: String,
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
    pub fn new_reverse(mut self) -> Self {
        self.compare = match self.compare {
            Comparison::Equals => Comparison::NotEquals,
            Comparison::NotEquals => Comparison::Equals,
            // I know these are incorrect but can't fix it at the moment
            Comparison::MoreThan => Comparison::LessThan,
            Comparison::LessThan => Comparison::MoreThan,
        };
        self
    }

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
        content: Range<usize>,
        else_ifs: Vec<(Condition, Range<usize>)>,
        else_content: Option<Range<usize>>,
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

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
enum Branch {
    First,
    Middle(usize),
    Last,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
struct Scope {
    /// This is the index of the node that was next'd. it's None when nothing has been loaded.
    //index: Option<usize>,
    choice: i32,
    branch: Option<Branch>,
}

/*impl Scope {
    fn inc(&mut self) {
        if let Some(idx) = &mut self.index {
            *idx += 1;
        } else {
            self.index = Some(0);
        }
    }
}*/

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NovelState {
    scene: String,
    variables: HashMap<String, i32>,
    scopes: Vec1<Scope>,
    offset: usize,
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

#[derive(Debug)]
pub enum GraphNode<'a> {
    Root,
    Node { node: &'a SceneNodeUser },
    Branch(Condition),
}

/*pub trait RevNeighbors {
    fn rev_neighbors(
        &self,
        a: NodeIndex<u32>,
    ) -> std::iter::Rev<<Vec<NodeIndex> as IntoIterator>::IntoIter>;
}

impl<'a> RevNeighbors for Graph<GraphNode<'a>, ()> {
    fn rev_neighbors(
        &self,
        a: NodeIndex<u32>,
    ) -> std::iter::Rev<<Vec<NodeIndex> as IntoIterator>::IntoIter> {
        self.neighbors(a).collect::<Vec<_>>().into_iter().rev()
    }
}*/

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
            offset: 0,
        }
    }

    pub fn add_scene(&mut self, name: String, data: &str) {
        self.add_nodes(name, parse(data));
    }

    pub fn add_nodes(&mut self, name: String, data: Vec<SceneNode>) {
        self.scenes.insert(name, data);
    }

    /*fn parse_into_graph<'a>(
        &'a self,
        graph: &mut Graph<GraphNode<'a>, ()>,
        parent: NodeIndex,
        content: &'a [SceneNode],
    ) {
        for node in content {
            match node {
                SceneNode::User(node) => {
                    let graph_node = graph.add_node(GraphNode::Node { node });
                    graph.update_edge(parent, graph_node, ());
                }
                SceneNode::Control(node) => match node {
                    SceneNodeControl::If {
                        cond,
                        else_ifs,
                        else_content,
                        content,
                    } => {
                        {
                            let graph_node = graph.add_node(GraphNode::Branch(cond.clone()));
                            graph.update_edge(parent, graph_node, ());
                            self.parse_into_graph(graph, graph_node, content);
                        }
                        for (cond, content) in else_ifs {
                            let graph_node = graph.add_node(GraphNode::Branch(cond.clone()));
                            graph.update_edge(parent, graph_node, ());
                            self.parse_into_graph(graph, graph_node, content);
                        }
                        if let Some(content) = else_content {
                            let graph_node =
                                graph.add_node(GraphNode::Branch(cond.clone().new_reverse()));
                            graph.update_edge(parent, graph_node, ());
                            self.parse_into_graph(graph, graph_node, content);
                        }
                    }
                    SceneNodeControl::Jump(target) => {
                        let scene = self
                            .scenes
                            .get(target)
                            .unwrap_or_else(|| panic!("Couldn't find scene '{}'", target));
                        self.parse_into_graph(graph, parent, scene);
                    }
                },
            }
        }
    }

    pub fn extract_graph<'a>(
        &'a self,
        starting_scene: &str,
    ) -> (Graph<GraphNode<'a>, ()>, NodeIndex) {
        let mut graph = Graph::<GraphNode, ()>::new();
        let root = graph.add_node(GraphNode::Root);
        let scene = self
            .scenes
            .get(starting_scene)
            .unwrap_or_else(|| panic!("Couldn't find scene '{}'", starting_scene));
        self.parse_into_graph(&mut graph, root, scene);
        (graph, root)
    }*/

    fn get_item(&mut self, state: &mut NovelState) -> Option<SceneNode> {
        let active_scene = self
            .scenes
            .get(&state.scene)
            .unwrap_or_else(|| panic!("Couldn't find scene '{}'", state.scene));

        let mut prev_scope = &state.scopes[0];
        let mut active_node = active_scene.front_mut();
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
                        Branch::First => active_scene
                            [content.start - state.offset..content.end - state.offset]
                            .front_mut(),
                        Branch::Middle(n) => else_ifs
                            .get(n)
                            .map(|o| {
                                active_scene[o.1.start - state.offset..o.1.end - state.offset]
                                    .front_mut()
                            })
                            .flatten(),
                        Branch::Last => else_content
                            .as_ref()
                            .map(|c| {
                                active_scene[c.start - state.offset..c.end - state.offset]
                                    .front_mut()
                            })
                            .flatten(),
                    };

                    if let Some(SceneNode::Control(SceneNodeControl::If { .. })) = active_node {
                    } else {
                        let node = match branch {
                            Branch::First => active_scene.remove(0),
                            Branch::Middle(n) => else_ifs
                                .get(n)
                                .map(|o| active_scene.remove(o.1.start))
                                .flatten(),
                            Branch::Last => else_content
                                .as_ref()
                                .map(|c| active_scene.remove(c.start))
                                .flatten(),
                        };
                        state.offset += 1;
                    };
                }
            }
            prev_scope = scope;
        }
        return None;
    }

    pub fn next(&self, state: &mut NovelState) -> Option<SceneNodeUser> {
        let active_node = self.get_item(state);
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
                        } else if else_content.is_some() {
                            state.scopes.last_mut().branch = Some(Branch::Last);
                            state.scopes.push(Scope::default())
                        }

                        return self.next(state);
                    }
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

fn parse_if(mut pair_it: pest::iterators::Pairs<Rule>) -> (Condition, Vec<(ParseRes, usize)>, usize) {
    let condition = {
        let mut cond_it = pair_it.next().unwrap().into_inner();
        let first = cond_it.next().unwrap().as_str().trim();
        let compare = match cond_it.next().unwrap().as_str() {
            "=" => Comparison::Equals,
            "!=" => Comparison::NotEquals,
            ">" => Comparison::MoreThan,
            "<" => Comparison::LessThan,
            c => panic!("{}", c),
        };
        let second = cond_it.next().unwrap().as_str().trim();

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
    let statements = pair_it
        .next()
        .unwrap()
        .into_inner()
        .map(|statement| parse_statement(statement.into_inner().next().unwrap()))
        .collect::<Vec<_>>(); // this evaluates the maps

    (
        condition,
        statements,
        statements.fold(0, |acc, s| acc + s.1),
    )
}

fn parse_non_control(pair: pest::iterators::Pair<'_, Rule>) -> SceneNodeUser {
    match pair.as_rule() {
        Rule::choice_statement => {
            let choices = pair
                .into_inner()
                .map(|choice| choice.as_str().trim().to_owned())
                .collect::<Vec<_>>();
            SceneNodeUser::Data(SceneNodeData::Choice(choices))
        }
        Rule::dialogue_statement => {
            let mut diag_it = pair.into_inner();
            let speaker = diag_it.next().unwrap().as_str().to_owned();
            let content = diag_it.next().unwrap().as_str().to_owned();
            SceneNodeUser::Data(SceneNodeData::Text {
                speaker: if speaker == "_" { None } else { Some(speaker) },
                content,
            })
        }
        Rule::scene_statement => {
            let mut scene_it = pair.into_inner();
            let name = scene_it.next().unwrap().as_str().to_owned();
            SceneNodeUser::Load(SceneNodeLoad::Background { name })
        }
        Rule::load_statement => {
            let mut load_it = pair.into_inner();
            let character = load_it.next().unwrap().as_str().to_owned();
            let mut property_list = load_it.next().unwrap().into_inner();
            let mut properties = HashMap::new();
            while let Some(property) = property_list.next() {
                let mut property = property.into_inner();
                let key = property.next().unwrap().as_str();
                let value = property.next().unwrap().as_str();
                properties.insert(key, value);
            }
            if let Some((&key, _)) = properties
                .iter()
                .find(|(&key, _)| key != "expression" && key != "placement")
            {
                panic!("Unknown load property key {}", key);
            }
            SceneNodeUser::Load(SceneNodeLoad::Character {
                character,
                expression: properties.get("expression").copied().map(String::from),
                placement: properties.get("placement").copied().map(String::from),
            })
        }
        Rule::sound_statement => {
            let mut sound_it = pair.into_inner();
            let name = sound_it.next().unwrap().as_str().to_owned();
            let channel = sound_it.next().unwrap().as_str().to_owned();
            SceneNodeUser::Load(SceneNodeLoad::PlaySound { name, channel })
        }
        Rule::remove_statement => {
            let mut remove_it = pair.into_inner();
            let name = remove_it.next().unwrap().as_str().to_owned();
            SceneNodeUser::Load(SceneNodeLoad::RemoveCharacter { name })
        }
        Rule::set_statement => {
            let mut set_it = pair.into_inner();
            let character = set_it.next().unwrap().as_str().to_owned();
            let key = set_it.next().unwrap().as_str();
            let value = set_it.next().unwrap().as_str();
            /* really ugly really bad but it's the easiest way of writing that I could think if */
            let mut properties = HashMap::new();
            properties.insert(key, value);
            SceneNodeUser::Load(SceneNodeLoad::Character {
                character,
                expression: properties.get("expression").copied().map(String::from),
                placement: properties.get("placement").copied().map(String::from),
            })
        }
        _ => unreachable!(),
    }
}

enum ParseRes {
    Multi(Vec<SceneNode>),
    Single(SceneNode),
}

fn parse_statement(pair: pest::iterators::Pair<'_, Rule>) -> ParseRes {
    let rule = pair.as_rule();
    if rule == Rule::if_statement {
        let mut pairs_it = pair.into_inner();
        let (if_cond, if_content) = parse_if(pairs_it.next().unwrap().into_inner());
        let mut else_content = None;
        let mut else_ifs: Vec<(Condition, Vec<ParseRes, usize>, usize)> = Vec::new();
        for case in pairs_it {
            match case.as_rule() {
                Rule::else_if_case => {
                    let pair_it = case.into_inner().next().unwrap().into_inner();
                    else_ifs.push_back(parse_if(pair_it));
                }
                Rule::else_case => {
                    let statement_it = case.into_inner().next().unwrap().into_inner();
                    else_content = Some(
                        statement_it
                            .map(|statement| {
                                parse_statement(statement.into_inner().next().unwrap())
                            })
                            .map(|o| (o,))
                            .collect::<Vec<_>>(),
                    );
                }
                _ => unreachable!(),
            }
        }

        let nodes = Vec::new();

        let content_range = 0..if_content.len();
        let else_if_ranges = else_ifs
            .iter()
            .map(|c| c.2)
            .scan(content_range.end, |state, new| {
                let r = state..(state + new);
                state += new;
                r
            })
            .collect::<Vec<_>>();
        let pre_else_range = else_if_ranges.last().unwrap_or(content_range).end;
        let else_range = else_content.map(|else_content| {
            pre_else_range..pre_else_range + else_content.iter().fold(0, |ic| ic.1)
        });
        nodes.push(SceneNode::Control(SceneNodeControl::If {
            cond: if_cond,
            content: content_range,
            else_ifs: else_if_ranges,
            else_content: else_range,
        }));

        ParseRes::Multi(
            std::iter::once(if_content)
                .chain(else_ifs.iter().map(|c| c.1.iter().map(|ic| ic.0)))
                .chain(else_content.iter())
                .collect(),
        )
    } else if rule == Rule::jump_statement {
        let mut jump_it = pair.into_inner();
        let target = jump_it.next().unwrap().as_str().to_owned();
        ParseRes::Single(SceneNode::Control(SceneNodeControl::Jump(target)))
    } else {
        ParseRes::Single(SceneNode::User(parse_non_control(pair)))
    }
}

fn parse(data: &str) -> VecDeque<SceneNode> {
    let mut nodes = VecDeque::new();

    let parse = NovelscriptParser::parse(Rule::file, &data)
        .unwrap()
        .next()
        .unwrap();

    for line in parse.into_inner() {
        match line.as_rule() {
            Rule::statement => match parse_statement(line.into_inner().next().unwrap()) {
                ParseRes::Single(node) => nodes.push_back(node),
                ParseRes::Multi(nodes) => nodes.extend(nodes.iter()),
            },
            _ => unreachable!(),
        }
    }

    nodes
}
