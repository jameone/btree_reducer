use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec::Vec;

/// `Transition`
pub trait Transition<T> {
    fn transition(&self) -> T;
}

/// `Dimension`
pub trait Dimension {
    fn dimension(&self) -> usize;
}

impl Dimension for bool {
    fn dimension(&self) -> usize {
        1
    }
}

impl<T> Dimension for Vec<T> {
    fn dimension(&self) -> usize {
        self.len()
    }
}

impl Dimension for String {
    fn dimension(&self) -> usize {
        self.len()
    }
}

impl Dimension for &bool {
    fn dimension(&self) -> usize {
        1
    }
}

impl<T> Dimension for &Vec<T> {
    fn dimension(&self) -> usize {
        self.len()
    }
}

impl Dimension for &String {
    fn dimension(&self) -> usize {
        self.len()
    }
}

/// `Input`
pub trait Input<T> {
    fn input(&self) -> T;
}

/// `Configuration`
pub trait Configuration<T> {
    fn configuration(&self) -> T;
}

/// `Program`
pub trait Program<T> {
    fn program(&self) -> T;
}

/// `Reinput`
pub trait Reinput<T>
where
    Self: Input<T>,
{
    type Error;
    fn reinput(&mut self, i: T) -> Result<(), Self::Error>;
}

/// `Reconfigure`
pub trait Reconfigure<T> {
    type Error;
    fn reconfigure(&mut self, c: T) -> Result<(), Self::Error>;
}

/// `Reprogram`
pub trait Reprogram<T> {
    type Error;
    fn reprogram(&mut self, p: T) -> Result<(), Self::Error>;
}

/// `Output`
pub trait Output<T> {
    type Error;
    fn output(&mut self) -> T;
}

/// `Short`
pub trait Short<T> {
    type Error;
    fn short(&mut self, x: T, y: T) -> Result<BTreeSet<T>, Self::Error>;
}

/// `RemoveShort`
pub trait RemoveShort<T> {
    type Error;
    fn remove_short(&mut self, x: T, y: T) -> Result<BTreeSet<T>, Self::Error>;
}

/// `AddGate`
pub trait AddGate<T> {
    fn add_gate(&mut self, g: T) -> T;
}
