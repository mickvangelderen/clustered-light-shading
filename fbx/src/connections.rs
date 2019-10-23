use fbx::*;

#[derive(Debug)]
pub struct Connections {
    pub oo: Vec<(i64, i64)>,
    pub op: Vec<(i64, i64, String)>,
}

impl Connections {
    pub fn from_fbx(node: &Node, stack: &mut Vec<String>) -> Self {
        stack.push(node.name.clone());

        let mut oo = Vec::new();
        let mut op = Vec::new();

        for node in node.children.iter() {
            assert_eq!("C", node.name.as_str());

            match node.properties[0].as_str() {
                "OO" => {
                    oo.push((node.properties[1].to_i64_exact(), node.properties[2].to_i64_exact()));
                }
                "OP" => {
                    op.push((
                        node.properties[1].to_i64_exact(),
                        node.properties[2].to_i64_exact(),
                        node.properties[3].as_str().to_string(),
                    ));
                }
                unknown => {
                    panic!("Unknown connections property kind {:?}", unknown);
                }
            }
        }

        stack.pop();

        Self { oo, op }
    }
}
