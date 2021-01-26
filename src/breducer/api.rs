use crate::arrangement::Arrangement;
use alloc::vec::Vec;
use btree_dag::error::Error;

pub trait Input {
    fn input(&self) -> Vec<bool>;
}

pub trait Output {
    fn output(&self) -> bool;
}

pub trait State {
    fn state(&self) -> Vec<bool>;
}

pub trait AddContact {
    fn add_contact(&self);
}

pub trait AddWiring {
    fn add_wiring(&self, a: Arrangement);
}

pub trait TransitionState {
    fn transition_state(&mut self, sv: Vec<bool>) -> Result<Vec<bool>, Error>;
}

pub trait TransitionInput {
    fn transition_input(&mut self, sv: Vec<bool>) -> Result<Vec<bool>, Error>;
}
