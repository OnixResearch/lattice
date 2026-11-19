//! Opt-in pure interaction-net worker profile on the existing process route.
//!
//! The pinned Noble executable evaluates one immutable `ngraph-net-i64/3`
//! program over two wrapping signed 64-bit inputs. This module admits the
//! producer's exact text forms (`text`), normalizes its report (`report`),
//! decides semantic-count, recomputation and semantic-limit outcomes (`count`,
//! `lineage`), declares the closed operation contract, its arguments and the
//! exact `bounded_process` pack (`route`), and adapts the existing operation
//! process runtime (`worker`). Canonical integers, host bindings and the
//! normalized outcome vocabulary stay in [`crate::pure_inet_profile`].

pub mod count;
pub mod lineage;
pub mod report;
pub mod route;
pub mod text;
pub mod worker;

/// One node of a closed pure arithmetic graph. Operands name strictly
/// earlier nodes, so every admitted program is finite and acyclic; a node
/// used more than once is shared by the producer's `dup` agents.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Node {
    /// One of the two admitted input slots.
    Input(u8),
    /// A signed 64-bit constant.
    Const(i64),
    /// Wrapping two's-complement addition of two earlier nodes.
    Add(u32, u32),
    /// Wrapping two's-complement subtraction, first operand minus second.
    Sub(u32, u32),
    /// Wrapping two's-complement multiplication of two earlier nodes.
    Mul(u32, u32),
}

/// A finite acyclic graph over exactly two inputs with one output node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    nodes: Vec<Node>,
    output: u32,
}

impl Program {
    /// Admit a graph only when it is bounded, defines each input slot exactly
    /// once, names only earlier operands and has one existing output node.
    pub fn new(nodes: Vec<Node>, output: u32) -> Result<Self, Error> {
        if nodes.is_empty() {
            return Err(crate::pure_inet_profile::PureProfileError::EmptyProgram.into());
        }
        if nodes.len() > crate::pure_inet_profile::MAX_PROGRAM_OPERATIONS {
            return Err(crate::pure_inet_profile::PureProfileError::ProgramTooLarge.into());
        }
        for (index, node) in (0_u32..).zip(&nodes) {
            match *node {
                Node::Input(slot) if usize::from(slot) >= crate::pure_inet_profile::INPUT_COUNT => {
                    return Err(crate::pure_inet_profile::PureProfileError::InvalidInputIndex.into());
                }
                Node::Add(left, right) | Node::Sub(left, right) | Node::Mul(left, right)
                    if left >= index || right >= index =>
                {
                    return Err(Error::CyclicOperand);
                }
                Node::Input(_) | Node::Const(_) | Node::Add(..) | Node::Sub(..) | Node::Mul(..) => {}
            }
        }
        if usize::try_from(output).map_or(true, |output| output >= nodes.len()) {
            return Err(Error::InvalidOutputNode);
        }
        let is_each_slot_defined_once = (0..crate::pure_inet_profile::INPUT_COUNT).all(|slot| {
            nodes
                .iter()
                .filter(|node| matches!(node, Node::Input(index) if usize::from(*index) == slot))
                .count()
                == 1
        });
        if !is_each_slot_defined_once {
            return Err(Error::WrongInputArity);
        }
        debug_assert!(nodes.len() <= crate::pure_inet_profile::MAX_PROGRAM_OPERATIONS);
        Ok(Self { nodes, output })
    }

    /// The admitted nodes in canonical order.
    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }

    /// The single output node.
    pub fn output(&self) -> u32 {
        self.output
    }

    /// The independent host comparator: Rust `wrapping_*` arithmetic over
    /// the same graph, never the producer's net reduction.
    pub fn evaluate(&self, inputs: crate::pure_inet_profile::PureInputs) -> Result<i64, Error> {
        let mut values: Vec<i64> = Vec::with_capacity(self.nodes.len());
        for node in &self.nodes {
            let operand = |index: u32| -> Result<i64, Error> {
                usize::try_from(index).ok().and_then(|index| values.get(index).copied()).ok_or(Error::CyclicOperand)
            };
            let value = match *node {
                Node::Input(0) => inputs.first,
                Node::Input(_) => inputs.second,
                Node::Const(value) => value,
                Node::Add(left, right) => operand(left)?.wrapping_add(operand(right)?),
                Node::Sub(left, right) => operand(left)?.wrapping_sub(operand(right)?),
                Node::Mul(left, right) => operand(left)?.wrapping_mul(operand(right)?),
            };
            values.push(value);
        }
        debug_assert_eq!(values.len(), self.nodes.len());
        usize::try_from(self.output)
            .ok()
            .and_then(|output| values.get(output).copied())
            .ok_or(Error::InvalidOutputNode)
    }
}

/// Refusals of the worker profile; canonical-integer, identity and binding
/// refusals keep their existing host-core meaning.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum Error {
    #[error(transparent)]
    Core(#[from] crate::pure_inet_profile::PureProfileError),
    #[error("interaction-net operand must name an earlier node")]
    CyclicOperand,
    #[error("interaction-net program must name exactly one existing output node")]
    InvalidOutputNode,
    #[error("interaction-net program must define each of its two input slots exactly once")]
    WrongInputArity,
    #[error("interaction-net program text is not the canonical closed producer form")]
    MalformedProgramText,
    #[error("interaction-net program uses an operation, slot, fact, or output outside the closed vocabulary")]
    UnsupportedProgramLine,
    #[error("interaction-net input text is not the canonical two-input vector form")]
    MalformedInputsText,
    #[error("interaction-net invocation arguments are malformed or outside the closed contract")]
    InvalidArguments,
    #[error("interaction-net artifact is missing, linked, unbounded, or unreadable")]
    ArtifactUnavailable,
    #[error("interaction-net producer ceilings exceed the profile or process bounds")]
    InvalidCeilings,
    #[error("interaction-net semantic count binding is crossed, malformed, or unqualified")]
    CrossedCountBinding,
    #[error("interaction-net recomputation is not admitted")]
    RecomputationRefused,
    #[error("interaction-net consumer binding is superseded")]
    SupersededConsumerBinding,
    #[error("interaction-net pack could not be authored")]
    PackAuthoring,
}
