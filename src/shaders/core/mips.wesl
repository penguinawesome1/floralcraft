import constants::{MIP_CAPACITY_L1, MIP_CAPACITY_L2, MIP_CAPACITY_L3, MIP_CAPACITY_L4, MIP_CAPACITY_L5};

struct SkipMips {
    l1: array<atomic<u32>, MIP_CAPACITY_L1>,
    l2: array<atomic<u32>, MIP_CAPACITY_L2>,
    l3: array<atomic<u32>, MIP_CAPACITY_L3>,
    l4: array<atomic<u32>, MIP_CAPACITY_L4>,
    l5: array<atomic<u32>, MIP_CAPACITY_L5>,
}

struct SkipMipsView {
    l1: array<u32, MIP_CAPACITY_L1>,
    l2: array<u32, MIP_CAPACITY_L2>,
    l3: array<u32, MIP_CAPACITY_L3>,
    l4: array<u32, MIP_CAPACITY_L4>,
    l5: array<u32, MIP_CAPACITY_L5>,
}