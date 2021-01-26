pub mod api;
use api::{Configuration, Input, Output, Reconfigure, Toggle};

#[derive(PartialEq, PartialOrd, Ord, Eq, Clone, Debug)]
pub struct XOR(bool, bool);

impl XOR {
    pub fn new() -> Self {
        XOR(bool::default(), bool::default())
    }
}

impl Default for XOR {
    fn default() -> Self {
        Self::new()
    }
}

impl Input for XOR {
    fn input(&self) -> bool {
        self.0
    }
}

impl Output for XOR {
    fn output(&self) -> bool {
        self.0 != self.1
    }
}

impl Configuration for XOR {
    fn configuration(&self) -> bool {
        self.1
    }
}

impl Toggle for XOR {
    fn toggle(&mut self) {
        self.0 = !self.0;
    }
}

impl Reconfigure for XOR {
    fn reconfigure(&mut self) {
        self.1 = !self.1;
    }
}

#[cfg(test)]
mod unit_tests {
    use crate::xor::api::{Configuration, Input, Output, Reconfigure, Toggle};
    use crate::xor::XOR;

    #[test]
    fn new() {
        let xor: XOR = XOR::new();
        assert_eq!(xor, XOR(false, false))
    }

    #[test]
    fn default() {
        let xor: XOR = XOR::default();
        assert_eq!(xor, XOR::new())
    }

    #[test]
    fn input() {
        let xor: XOR = XOR::new();
        assert!(!xor.input())
    }

    #[test]
    fn configuration() {
        let xor: XOR = XOR::new();
        assert!(!xor.configuration())
    }

    #[test]
    fn output() {
        let mut xor: XOR = XOR::new();
        assert!(!xor.output());

        xor.0 = true;
        assert!(xor.output())
    }

    #[test]
    fn toggle() {
        let mut xor: XOR = XOR::new();
        assert!(!xor.0);
        xor.toggle();
        assert!(xor.0)
    }

    #[test]
    fn reconfigure() {
        let mut xor: XOR = XOR::new();
        assert!(!xor.1);
        xor.reconfigure();
        assert!(xor.1)
    }
}
