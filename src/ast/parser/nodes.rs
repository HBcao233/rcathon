use cathon::Span;
use super::super::Token;
use std::collections::HashMap;

pub type NodeId = usize;
pub type Symbol = usize;

/// 字符串驻留
#[derive(Default, Debug)]
pub struct Interner {
  map: HashMap<String, Symbol>,
  vec: Vec<String>,
}

impl Interner {
  pub fn new() -> Self {
    Self {
      map: HashMap::new(),
      vec: Vec::new(),
    }
  }
  
  fn intern(&mut self, s: &str) -> Symbol {
    if let Some(&id) = self.map.get(s) { return id; }
    let id = self.vec.len();
    self.vec.push(s.to_string());
    self.map.insert(s.to_string(), id);
    id
  }
  
  fn resolve(&self, sym: Symbol) -> &str { &self.vec[sym] }
}

#[derive(Debug)]
pub enum NodeKind {
  Module { body: Vec<NodeId> },
  Expr { value: NodeId },
  Constant { value: Token },
  BinOp { left: NodeId, op: Token, right: NodeId },
  UnaryOp { op: Token, operand: NodeId },
  Call { func: NodeId, args: Vec<NodeId> },
  Assign { target: Symbol, value: NodeId },
}

#[derive(Debug)]
pub struct Node {
  kind: NodeKind,
  span: Span,
}

impl Node {
  pub fn kind(&self) -> &NodeKind {
    &self.kind
  }
  
  pub fn span(&self) -> &Span {
    &self.span
  }
}

#[derive(Debug)]
pub struct Arena {
  pub nodes: Vec<Node>,
}

impl Arena {
  pub fn new() -> Self { 
    Arena { nodes: Vec::new() } 
  }

  pub fn alloc(&mut self, kind: NodeKind, span: Span) -> NodeId {
    let id = self.nodes.len();
    self.nodes.push(Node { kind, span });
    id
  }
  
  #[allow(non_snake_case)]
  pub fn alloc_BinOp(&mut self, left: NodeId, op: Token, right: NodeId) -> NodeId {
    let left_node = self.get(left);
    let right_node = self.get(right);
    self.alloc(
      NodeKind::BinOp { left, op, right }, 
      Span::new(left_node.span().start, right_node.span().end)
    )
  }

  pub fn get(&self, id: NodeId) -> &Node { 
    &self.nodes[id]
  }
}
