/// Every type of block in the game has its own name
///
/// Allows ease of storing and accessing block dictionary from
/// Specifically stored as a u8 to maximize memory savings
#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Block {
    Air, // air must remain at the start
    Grass,
    Dirt,
    Stone,
    Bedrock,
    Missing, // missing must remain at the end
    // cannot exceed 256 entrees
}

impl Block {
    /// Takes input string and returns its corresponding name.
    /// Used to take config inputs and convert into keys for rendering images.
    ///
    /// # Examples
    ///
    /// ```
    /// use floralcraft::terrain::Block;
    ///
    /// assert_eq!(Block::from_string("air"), Block::Air);
    /// assert_eq!(Block::from_string("bedrock"), Block::Bedrock);
    /// assert_eq!(Block::from_string("air?!"), Block::Missing);
    /// ```
    pub fn from_string(s: &str) -> Self {
        match s {
            "air" => Self::Air,
            "grass" => Self::Grass,
            "dirt" => Self::Dirt,
            "stone" => Self::Stone,
            "bedrock" => Self::Bedrock,
            _ => Self::Missing,
        }
    }

    // uses transmute to convert a u32 to a block name
    // requires missing to be the last item in block name enum
    const fn from_u32(value: u32) -> Self {
        let value_u8 = value as u8;
        if value_u8 < (Self::Missing as u8) {
            unsafe { std::mem::transmute(value_u8) }
        } else {
            Self::Missing
        }
    }

    /// Returns the block definition associated with the block name instance.
    ///
    /// Allows access to default properties such as if the air block is collidable.
    ///
    /// # Examples
    ///
    /// ```
    /// use floralcraft::terrain::{ BlockDefinition, Block };
    ///
    /// let air_block: BlockDefinition = Block::Air.definition();
    /// let missing_block: BlockDefinition = Block::Missing.definition();
    ///
    /// assert_eq!(air_block.is_collidable(), false);
    /// assert!(!air_block.is_visible());
    /// assert_eq!(air_block, missing_block);
    /// ```
    pub const fn definition(&self) -> BlockDefinition {
        match self {
            Self::Air => BlockDefinition::AIR,
            Self::Grass => BlockDefinition::GRASS,
            Self::Dirt => BlockDefinition::DIRT,
            Self::Stone => BlockDefinition::STONE,
            Self::Bedrock => BlockDefinition::BEDROCK,
            Self::Missing => BlockDefinition::AIR,
        }
    }
}

/// Struct that stores generic block info.
/// Intended to be used for dictionaries, not individual blocks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockDefinition {
    data: u32,
}

impl BlockDefinition {
    // block type (bits 0-7)
    const NAME_MASK: u32 = 0xff;
    const NAME_SHIFT: u32 = 0;

    // is hoverable (bit 8)
    const HOVERABLE_MASK: u32 = 1 << 8;

    // is visible (bit 9)
    const VISIBLE_MASK: u32 = 1 << 9;

    // is breakable (bit 10)
    const BREAKABLE_MASK: u32 = 1 << 10;

    // is collidable (bit 11)
    const COLLIDABLE_MASK: u32 = 1 << 11;

    // is replacable (bit 12)
    const REPLACEABLE_MASK: u32 = 1 << 12;

    // creates a new block given all characteristics of it
    const fn new(
        name: Block,
        is_hoverable: bool,
        is_visible: bool,
        is_breakable: bool,
        is_collidable: bool,
        is_replaceable: bool
    ) -> Self {
        let mut data: u32 = 0;

        data |= ((name as u32) & Self::NAME_MASK) << Self::NAME_SHIFT;

        if is_hoverable {
            data |= Self::HOVERABLE_MASK;
        }
        if is_visible {
            data |= Self::VISIBLE_MASK;
        }
        if is_breakable {
            data |= Self::BREAKABLE_MASK;
        }
        if is_collidable {
            data |= Self::COLLIDABLE_MASK;
        }
        if is_replaceable {
            data |= Self::REPLACEABLE_MASK;
        }

        Self { data }
    }

    /// The predefined air block.
    ///
    /// Air is an invisible, non-collidable, and replaceable block.
    ///
    /// # Examples
    ///
    /// ```
    /// use floralcraft::terrain::BlockDefinition;
    ///
    /// assert!(!BlockDefinition::AIR.is_visible());
    /// assert!(!BlockDefinition::AIR.is_collidable());
    /// assert!(BlockDefinition::AIR.is_replaceable());
    /// ```
    pub const AIR: Self = Self::new(Block::Air, false, false, false, false, true);
    pub const GRASS: Self = Self::new(Block::Grass, true, true, true, true, false);
    pub const DIRT: Self = Self::new(Block::Dirt, true, true, true, true, false);
    pub const STONE: Self = Self::new(Block::Stone, true, true, true, true, false);
    pub const BEDROCK: Self = Self::new(Block::Bedrock, true, true, false, true, false);

    /// Returns the `Block` (type) of this block.
    ///
    /// # Examples
    ///
    /// ```
    /// use floralcraft::terrain::{BlockDefinition, Block};
    ///
    /// assert_eq!(BlockDefinition::AIR.get_name(), Block::Air);
    /// assert_eq!(BlockDefinition::BEDROCK.get_name(), Block::Bedrock);
    /// ```
    pub const fn get_name(&self) -> Block {
        Block::from_u32(self.data & Self::NAME_MASK)
    }

    /// Checks if this block is hoverable.
    ///
    /// A hoverable block typically allows the player to highlight it when looking at it.
    ///
    /// # Examples
    ///
    /// ```
    /// use floralcraft::terrain::BlockDefinition;
    ///
    /// assert!(BlockDefinition::GRASS.is_hoverable());
    /// assert!(!BlockDefinition::AIR.is_hoverable());
    /// ```
    pub const fn is_hoverable(&self) -> bool {
        (self.data & Self::HOVERABLE_MASK) != 0
    }

    /// Checks if this block is visible in the game world.
    pub const fn is_visible(&self) -> bool {
        (self.data & Self::VISIBLE_MASK) != 0
    }

    /// Checks if this block can be destroyed by players.
    pub const fn is_breakable(&self) -> bool {
        (self.data & Self::BREAKABLE_MASK) != 0
    }

    /// Checks if this block can prevent entities from passing through it.
    pub const fn is_collidable(&self) -> bool {
        (self.data & Self::COLLIDABLE_MASK) != 0
    }

    /// Checks if this block can be replaced by placing another block over it.
    pub const fn is_replaceable(&self) -> bool {
        (self.data & Self::REPLACEABLE_MASK) != 0
    }
}
