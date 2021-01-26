pub trait Input {
    fn input(&self) -> bool;
}

pub trait Output {
    fn output(&self) -> bool;
}

pub trait Configuration {
    fn configuration(&self) -> bool;
}

pub trait Toggle {
    fn toggle(&mut self);
}

pub trait Reconfigure {
    fn reconfigure(&mut self);
}
