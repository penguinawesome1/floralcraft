pub trait BlockGenerator: Send + Sync + 'static {
    fn choose_block(&self, pos: BlockPosition, params: &WorldGeneration) -> SnugType;
}
