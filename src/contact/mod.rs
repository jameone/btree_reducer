use crate::arrangement::Arrangement;
use crate::contact::api::{Gate, Id, Wiring};
use crate::xor::XOR;

pub mod api;

#[derive(PartialEq, PartialOrd, Ord, Eq, Clone)]
pub struct Contact {
    id: usize,
    gate: XOR,
    wiring: Arrangement,
}

impl Id for Contact {
    fn id(&self) -> usize {
        self.id
    }
}

impl Gate for Contact {
    fn gate(&self) -> XOR {
        self.gate.clone()
    }
}

impl Wiring for Contact {
    fn wiring(&self) -> Arrangement {
        self.wiring.clone()
    }
}
