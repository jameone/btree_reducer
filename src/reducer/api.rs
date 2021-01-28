pub trait Input<T> {
    fn input(&self) -> T;
}

pub trait Output<T> {
    fn output(&mut self) -> T;
}

pub trait State<T> {
    fn state(&self) -> T;
}

pub trait AddContact {
    fn add_contact(&self);
}

pub trait Reprogram<T> {
    type Error;
    fn reprogram(&mut self, p: T) -> Result<(), Self::Error>;
}

pub trait TransitionState<T> {
    type Error;
    fn transition_state(&mut self, s: T) -> Result<(), Self::Error>;
}

pub trait TransitionInput<T> {
    type Error;
    fn transition_input(&mut self, i: T) -> Result<(), Self::Error>;
}
