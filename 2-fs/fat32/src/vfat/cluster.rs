#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Copy, Clone, Hash)]
pub struct Cluster(u32);

impl From<u32> for Cluster {
    fn from(raw_num: u32) -> Cluster {
        Cluster(raw_num & !(0xF << 28))
    }
}

impl Cluster {
    /// Is this a valid cluster?
    pub fn is_valid(&self) -> bool {
        self.0 > 2
    }

    pub fn fat_index(&self) -> u32 {
        self.0
    }

    /// Get the cluster index represented by this cluster.
    pub fn data_index(&self) -> u32 {
        self.0 - 2
    }
}
