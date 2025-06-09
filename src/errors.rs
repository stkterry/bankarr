use std::{alloc::{Layout, LayoutError}, fmt};


#[derive(Debug, Clone)]
pub struct BankFullError {}

// Theres no reason to provide `test` coverage for these implementations.

#[cfg(not(tarpaulin_include))]
impl fmt::Display for BankFullError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "bank is full")
    }
}

#[derive(Debug, Clone)]
pub enum AllocErr {
    Overflow,
    Layout,
    Alloc { layout: Layout }
}

#[cfg(not(tarpaulin_include))]
impl AllocErr {
    #[inline]
    pub(super) const fn layout(_err: LayoutError) -> Self { Self::Layout }

    #[inline]
    pub(super) const fn alloc(layout: Layout) -> Self { Self::Alloc { layout } }
}

#[cfg(not(tarpaulin_include))]
impl fmt::Display for AllocErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Allocation error: {:?}", self)
    }
}

