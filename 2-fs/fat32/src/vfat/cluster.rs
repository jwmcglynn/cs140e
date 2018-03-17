use vfat::*;

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Copy, Clone, Hash)]
pub struct Cluster(u32);

impl From<u32> for Cluster {
    fn from(raw_num: u32) -> Cluster {
        Cluster(raw_num & !(0xF << 28))
    }
}

impl Cluster {
    /// Get the cluster index represented by this cluster.
    pub fn index(&self) -> u32 {
        self.0
    }
}
