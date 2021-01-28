/// `Input`
pub trait Input<T> {
    fn input(&self) -> T;
}

/// `VerifyInput`
pub trait VerifyInput<T> {
    fn verify_input(&self) -> T;
}

/// `Configuration`
pub trait Configuration<T> {
    fn configuration(&self) -> T;
}

/// `VerifyConfiguration`
pub trait VerifyConfiguration<T> {
    fn verify_configuration(&self) -> T;
}

/// `Program`
pub trait Program<T> {
    fn program(&self) -> T;
}

/// `VerifyProgram`
pub trait VerifyProgram<T> {
    fn verify_program(&self) -> T;
}

/// `Reinput`
pub trait Reinput<T> {
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
    fn output(&mut self) -> Result<T, Self::Error>;
}
