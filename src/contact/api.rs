use crate::arrangement::Arrangement;
use crate::xor::XOR;

pub trait Id {
    fn id(&self) -> usize;
}

pub trait Gate {
    fn gate(&self) -> XOR;
}

pub trait Wiring {
    fn wiring(&self) -> Arrangement;
}
