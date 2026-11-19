//! Frozen models over normalized per-mille features: a binary decision tree and a small MLP.
//!
//! Both produce one integer score per declared action in checked `i64` arithmetic. Overflow is a
//! refusal, never a wrap or a clamp.

/// Fixed-point scale of normalized features, thresholds, and weighted sums.
pub const PER_MILLE: i64 = 1_000;
/// Activation of a hidden MLP layer that zeroes negative sums.
const RELU: &str = "relu";
/// Activation that passes sums through; the output layer's only activation.
const IDENTITY: &str = "identity";

/// The model section of an artifact, tagged by policy kind; keys sort after `kind`.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub(crate) enum ModelDocument {
    /// A binary decision tree; node 0 is the root and every child follows its parent.
    #[serde(rename = "decision-tree-v1")]
    DecisionTree {
        nodes: alloc::vec::Vec<NodeDocument>,
    },
    /// A small multilayer perceptron; the last layer has one output per action.
    #[serde(rename = "mlp-per-mille-v1")]
    Mlp {
        layers: alloc::vec::Vec<LayerDocument>,
    },
}

impl ModelDocument {
    /// The declared policy kind.
    pub(crate) const fn kind(&self) -> &'static str {
        match self {
            Self::DecisionTree { .. } => crate::TREE_POLICY_KIND,
            Self::Mlp { .. } => crate::MLP_POLICY_KIND,
        }
    }
}

/// One tree node: a leaf with one score per action, or a split on one normalized feature.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub(crate) enum NodeDocument {
    /// One score per declared action.
    Leaf { scores: alloc::vec::Vec<i64> },
    /// `feature <= threshold` continues at `below`, otherwise at `above`.
    Split {
        above: usize,
        below: usize,
        feature: usize,
        threshold: i64,
    },
}

/// One dense layer: `biases[j] + sum(weights[j][i] * input[i]) / 1000`, then its activation.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct LayerDocument {
    activation: alloc::string::String,
    biases: alloc::vec::Vec<i64>,
    weights: alloc::vec::Vec<alloc::vec::Vec<i64>>,
}

/// The input and output widths a model must match.
#[derive(Clone, Copy)]
pub(crate) struct Shape {
    pub(crate) feature_count: usize,
    pub(crate) action_count: usize,
}

/// An admitted model.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Model {
    Tree(alloc::vec::Vec<Node>),
    Mlp(alloc::vec::Vec<Layer>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Node {
    Leaf(alloc::vec::Vec<i64>),
    Split {
        feature: usize,
        threshold: i64,
        below: usize,
        above: usize,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Layer {
    weights: alloc::vec::Vec<alloc::vec::Vec<i64>>,
    biases: alloc::vec::Vec<i64>,
    is_relu: bool,
}

/// Admits `document` for `shape` and returns it with its worst-case inference operations.
// r[impl agent_harness_runtime.frozen_policy.model]
pub(crate) fn admit(
    document: &ModelDocument,
    shape: Shape,
) -> Result<(Model, u64), crate::AdmissionError> {
    match document {
        ModelDocument::DecisionTree { nodes } => admit_tree(nodes, shape),
        ModelDocument::Mlp { layers } => admit_mlp(layers, shape),
    }
}

/// A tree whose node 0 is the root, whose children follow their parent, and in which every
/// other node has exactly one parent: so it is acyclic, connected, and terminates.
fn admit_tree(nodes: &[NodeDocument], shape: Shape) -> Result<(Model, u64), crate::AdmissionError> {
    if nodes.is_empty() || nodes.len() > crate::MAX_TREE_NODES {
        return Err(crate::AdmissionError::Model);
    }
    let mut parents = alloc::vec![0_u8; nodes.len()];
    let mut depths = alloc::vec![0_u64; nodes.len()];
    let mut admitted = alloc::vec::Vec::with_capacity(nodes.len());
    for (index, node) in nodes.iter().enumerate() {
        let depth = depths.get(index).copied().unwrap_or_default();
        admitted.push(admit_node(node, index, shape)?);
        if let NodeDocument::Split { above, below, .. } = node {
            for child in [*below, *above] {
                let (Some(parent_count), Some(child_depth)) =
                    (parents.get_mut(child), depths.get_mut(child))
                else {
                    return Err(crate::AdmissionError::Model);
                };
                *parent_count = parent_count.saturating_add(1);
                *child_depth = depth.saturating_add(1);
            }
        }
    }
    let has_single_parents = parents
        .iter()
        .skip(1)
        .all(|parent_count| *parent_count == 1);
    let worst_case = depths.iter().copied().max().unwrap_or_default();
    if !has_single_parents || worst_case > crate::MAX_TREE_DEPTH {
        return Err(crate::AdmissionError::Model);
    }
    debug_assert_eq!(admitted.len(), nodes.len());
    Ok((Model::Tree(admitted), worst_case))
}

/// One node at `index`: a split names a declared feature, a per-mille threshold, and two distinct
/// later children; a leaf holds one bounded score per action.
fn admit_node(
    node: &NodeDocument,
    index: usize,
    shape: Shape,
) -> Result<Node, crate::AdmissionError> {
    match node {
        NodeDocument::Leaf { scores } => {
            if scores.len() != shape.action_count {
                return Err(crate::AdmissionError::Model);
            }
            if !scores.iter().copied().all(is_parameter) {
                return Err(crate::AdmissionError::Parameters);
            }
            Ok(Node::Leaf(scores.clone()))
        }
        NodeDocument::Split {
            above,
            below,
            feature,
            threshold,
        } => {
            let is_split = *feature < shape.feature_count
                && (0..=PER_MILLE).contains(threshold)
                && *below > index
                && *above > index
                && below != above;
            if !is_split {
                return Err(crate::AdmissionError::Model);
            }
            Ok(Node::Split {
                feature: *feature,
                threshold: *threshold,
                below: *below,
                above: *above,
            })
        }
    }
}

/// Layers whose widths chain from the features to one output per action, with an identity
/// output layer and bounded parameters.
fn admit_mlp(
    layers: &[LayerDocument],
    shape: Shape,
) -> Result<(Model, u64), crate::AdmissionError> {
    if layers.is_empty() || layers.len() > crate::MAX_LAYERS {
        return Err(crate::AdmissionError::Model);
    }
    let mut width = shape.feature_count;
    let mut operations: u64 = 0;
    let mut admitted = alloc::vec::Vec::with_capacity(layers.len());
    for (index, layer) in layers.iter().enumerate() {
        let is_output = index.saturating_add(1) == layers.len();
        let outputs = layer.weights.len();
        let is_shaped = (1..=crate::MAX_LAYER_WIDTH).contains(&outputs)
            && layer.biases.len() == outputs
            && layer.weights.iter().all(|row| row.len() == width)
            && (!is_output || outputs == shape.action_count);
        let is_relu = match layer.activation.as_str() {
            RELU if !is_output => true,
            IDENTITY => false,
            _ => return Err(crate::AdmissionError::Model),
        };
        if !is_shaped {
            return Err(crate::AdmissionError::Model);
        }
        let is_layer_bounded = layer.biases.iter().copied().all(is_parameter)
            && layer.weights.iter().flatten().copied().all(is_parameter);
        if !is_layer_bounded {
            return Err(crate::AdmissionError::Parameters);
        }
        let products = u64::try_from(outputs.saturating_mul(width))
            .map_err(|_| crate::AdmissionError::Model)?;
        operations = operations.saturating_add(products);
        admitted.push(Layer {
            weights: layer.weights.clone(),
            biases: layer.biases.clone(),
            is_relu,
        });
        width = outputs;
    }
    debug_assert_eq!(width, shape.action_count);
    Ok((Model::Mlp(admitted), operations))
}

/// The model's score per action for `normalized` features, with the inference operations taken;
/// `None` when checked arithmetic overflows.
pub(crate) fn evaluate(model: &Model, normalized: &[i64]) -> Option<(alloc::vec::Vec<i64>, u64)> {
    match model {
        Model::Tree(nodes) => evaluate_tree(nodes, normalized),
        Model::Mlp(layers) => evaluate_mlp(layers, normalized),
    }
}

fn evaluate_tree(nodes: &[Node], normalized: &[i64]) -> Option<(alloc::vec::Vec<i64>, u64)> {
    let mut index = 0;
    let mut splits: u64 = 0;
    loop {
        match nodes.get(index)? {
            Node::Leaf(scores) => return Some((scores.clone(), splits)),
            Node::Split {
                feature,
                threshold,
                below,
                above,
            } => {
                let value = *normalized.get(*feature)?;
                debug_assert!(*below > index && *above > index, "children follow parents");
                index = if value <= *threshold { *below } else { *above };
                splits = splits.saturating_add(1);
            }
        }
    }
}

fn evaluate_mlp(layers: &[Layer], normalized: &[i64]) -> Option<(alloc::vec::Vec<i64>, u64)> {
    let mut activations: alloc::vec::Vec<i64> = normalized.to_vec();
    let mut operations: u64 = 0;
    for layer in layers {
        let mut outputs = alloc::vec::Vec::with_capacity(layer.biases.len());
        for (row, bias) in layer.weights.iter().zip(&layer.biases) {
            let mut total: i64 = 0;
            for (weight, input) in row.iter().zip(&activations) {
                total = total.checked_add(weight.checked_mul(*input)?)?;
            }
            let sum = bias.checked_add(total.checked_div(PER_MILLE)?)?;
            outputs.push(if layer.is_relu { sum.max(0) } else { sum });
            operations = operations.saturating_add(u64::try_from(row.len()).ok()?);
        }
        activations = outputs;
    }
    Some((activations, operations))
}

const fn is_parameter(value: i64) -> bool {
    value >= -crate::MAX_PARAMETER_MAGNITUDE && value <= crate::MAX_PARAMETER_MAGNITUDE
}
