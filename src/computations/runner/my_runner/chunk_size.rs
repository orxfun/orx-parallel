use crate::{computations::computation_kind::ComputationKind, parameters::ChunkSize};

const MAX_CHUNK_SIZE: usize = 1 << 20;
const DESIRED_MIN_CHUNK_SIZE: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResolvedChunkSize {
    Min(usize),
    Exact(usize),
}
