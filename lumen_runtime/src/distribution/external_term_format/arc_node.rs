use std::sync::Arc;

use liblumen_alloc::erts::exception::Exception;
use liblumen_alloc::Node;

use crate::distribution::nodes::try_atom_to_arc_node;

use super::atom;

pub fn decode(safe: bool, bytes: &[u8]) -> Result<(Arc<Node>, &[u8]), Exception> {
    let (atom, after_atom_bytes) = atom::decode_tagged(safe, bytes)?;
    let arc_node = try_atom_to_arc_node(&atom)?;

    Ok((arc_node, after_atom_bytes))
}
