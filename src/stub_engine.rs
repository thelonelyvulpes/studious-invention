use std::collections::HashMap;
use std::time::Duration;

pub(crate) struct StubEngine {
    pub(crate) nodes: Vec<NodeContainer>,
}

pub(crate) struct NodeContainer {
    pub(crate) node: Node,
    pub(crate) script_line: i32,
}

pub(crate) trait ValueType {
    fn jolt(self) -> Vec<u8>;
}

impl ValueType for i64 {
    fn jolt(self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }
}
impl ValueType for f64 {
    fn jolt(self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }
}

pub(crate) enum Node {
    Bolt {
        major: u8,
        minor: u8,
    },
    Fork {
        nodes: Vec<Node>,
    },
    Optional {
        node: Box<Node>,
    },
    Success {
        meta: HashMap<String, String>,
    },
    Failure {
        meta: HashMap<String, String>,
    },
    Ignored,
    Record {
        values: Vec<Box<dyn ValueType>>,
    },
    Hello {
        auth: Option<HashMap<String, String>>,
        meta: HashMap<String, String>,
    },
    Begin {
        meta: HashMap<String, String>,
    },
    Run {
        query: String,
        params: HashMap<String, String>,
        meta: HashMap<String, String>,
    },
    AssertOrder {
        time: Duration,
    },
    Commit(),
    Rollback(),
}

fn run(node: Node) -> bool {
    match node {
        Node::Fork { nodes } => {
            for node in nodes {
                if run(node) {
                    return true;
                }
            }
            false
        }
        Node::Run {
            query,
            params,
            meta,
        } => {
            return true;
        }
        Node::Hello { auth, meta } => {
            return true;
        }
        _ => true,
    }
}

pub(crate) fn create_engine() -> StubEngine {
    let mut engine = StubEngine { nodes: Vec::new() };
    engine.nodes.push(NodeContainer {
        node: Node::Ignored,
        script_line: 0,
    });

    engine
}
