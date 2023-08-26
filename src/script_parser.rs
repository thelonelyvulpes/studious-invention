use crate::stub_engine::{Node, NodeContainer, StubEngine};
use nom::bytes::complete::{tag, take_while, take_while1};
use nom::sequence::Tuple;
use nom::{IResult, Parser};
use std::path::Path;
use tokio;

pub(crate) async fn load(filename: String) -> std::io::Result<StubEngine> {
    let path = Path::new(filename.as_str());
    let text = tokio::fs::read_to_string(path).await?;
    build_engine(text.as_str())
}

fn build_engine(script: &str) -> std::io::Result<StubEngine> {
    let mut line = Box::new(1);
    let mut nodes = Vec::with_capacity(128);
    let (script, (major, minor)) = bolt_version(script).unwrap();
    nodes.push(NodeContainer {
        node: Node::Bolt { major, minor },
        script_line: 1,
    });

    Ok(StubEngine { nodes })
}

fn bolt_version(script: &str) -> IResult<&str, (u8, u8)> {
    let (script, _) = tag("!: BOLT ")(script)?;
    let (script, (major, _, minor)) = (
        |x| take_while(|y: char| y.is_digit(10))(x),
        |x| take_while1(|y| y == '.').parse(x),
        |x| take_while(|y: char| y.is_digit(10))(x),
    )
        .parse(script)?;

    Ok((script, (major.parse().unwrap(), minor.parse().unwrap())))
}

#[cfg(test)]
mod tests {
    use crate::script_parser::bolt_version;
    #[test]
    fn it_works() {
        let (script, (major, minor)) = bolt_version("!: BOLT 42.10").unwrap();
        assert_eq!(major, 42);
        assert_eq!(minor, 10);
    }
}
