#[derive(PartialEq, PartialOrd, Ord, Eq, Clone, Debug)]
pub enum Arrangement {
    Series,
    Parallel,
}

impl From<Arrangement> for bool {
    fn from(a: Arrangement) -> Self {
        match a {
            Arrangement::Series => true,
            Arrangement::Parallel => false,
        }
    }
}
