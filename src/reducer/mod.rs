use crate::reducer::api::{
    Configuration, Dimension, Input, Output, Program, Reconfigure, Reinput, Reprogram, Transition,
};
use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec::Vec;
use btree_dag::error::Error;
use btree_dag::{AddEdge, AddVertex, BTreeDAG, Connections, RemoveEdge, RemoveVertex, Vertices};

mod api;

#[derive(PartialEq, PartialOrd, Ord, Eq, Clone, Debug)]
pub struct Contact<T>
where
    T: Default + Ord + Clone,
{
    id: usize,
    input: T,
    configuration: T,
    program: T,
}

impl<T> Input<T> for Contact<T>
where
    T: Default + Ord + Clone,
{
    fn input(&self) -> T {
        self.input.clone()
    }
}

impl<T> Configuration<T> for Contact<T>
where
    T: Default + Ord + Clone,
{
    fn configuration(&self) -> T {
        self.configuration.clone()
    }
}

impl<T> Program<T> for Contact<T>
where
    T: Default + Ord + Clone,
{
    fn program(&self) -> T {
        self.program.clone()
    }
}

impl Output<bool> for Contact<bool> {
    type Error = Error;
    fn output(&mut self) -> Result<bool, Self::Error> {
        Ok(self.input != self.configuration)
    }
}

impl<T> Reinput<T> for Contact<T>
where
    T: Default + Ord + Clone,
{
    type Error = Error;
    fn reinput(&mut self, i: T) -> Result<(), Self::Error> {
        self.input = i;
        Ok(())
    }
}

impl<T> Reconfigure<T> for Contact<T>
where
    T: Default + Ord + Clone,
{
    type Error = Error;
    fn reconfigure(&mut self, c: T) -> Result<(), Self::Error> {
        self.configuration = c;
        Ok(())
    }
}

impl<T> Reprogram<T> for Contact<T>
where
    T: Default + Ord + Clone,
{
    type Error = Error;
    fn reprogram(&mut self, p: T) -> Result<(), Self::Error> {
        self.program = p;
        Ok(())
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct BTreeReducer<T>
where
    T: Default + Ord + Clone,
{
    dag: BTreeDAG<Contact<T>>,
}

impl<T> BTreeReducer<T>
where
    T: Default + Ord + Clone + Transition<T>,
{
    fn new() -> Self {
        let mut dag: BTreeDAG<Contact<T>> = BTreeDAG::new();
        let contact_zero: Contact<T> = Contact {
            id: 0,
            input: T::default(),
            configuration: T::default(),
            program: T::default(),
        };
        dag.add_vertex(contact_zero);
        BTreeReducer { dag }
    }

    fn add_contact(&mut self, c: Contact<T>) -> Contact<T>
    where
        Contact<T>: Output<T>,
    {
        let vertices: Vec<&Contact<T>> = self.dag.vertices().into_iter().collect();
        let contact: Contact<T> = Contact {
            id: vertices[vertices.len() - 1].id + 1,
            input: T::default(),
            configuration: T::default(),
            program: T::default(),
        };
        self.dag.add_vertex(contact.clone());
        self.dag.add_edge(c, contact.clone()).unwrap();
        self._resolve_branch(self.root()).unwrap();
        contact
    }

    pub fn root(&self) -> Contact<T> {
        let vertices: Vec<Contact<T>> = self.dag.vertices().into_iter().cloned().collect();
        vertices[0].clone()
    }

    fn update(&mut self, p: Contact<T>, u: Contact<T>)
    where
        Contact<T>: Output<T>,
    {
        let previous_parents: BTreeSet<Contact<T>> = self
            .dag
            .vertices()
            .into_iter()
            .cloned()
            .map(|v| -> (Contact<T>, &BTreeSet<Contact<T>>) {
                (v.clone(), self.dag.connections(v).unwrap())
            })
            .filter(|t| -> bool { t.1.contains(&p) })
            .map(|t| -> Contact<T> { t.0 })
            .collect();

        // Get all the edges from the previous vertex;
        // let result = self.dag.remove_vertex(c);
        let removal = self.dag.remove_vertex(p);
        self.dag.add_vertex(u.clone());
        // Add children back.
        if let Ok(previous_children) = removal {
            for previous_child in previous_children {
                self.dag.add_edge(u.clone(), previous_child).unwrap();
            }
        }
        // Add parents back.
        for previous_parent in previous_parents {
            self.dag
                .add_edge(previous_parent.clone(), u.clone())
                .unwrap();
            self._resolve_branch(previous_parent).unwrap();
        }
    }

    fn get_input_contacts(&self) -> Vec<Contact<T>> {
        self.dag
            .vertices()
            .into_iter()
            .cloned()
            .filter(|c| -> bool { self.dag.connections(c.clone()).unwrap().is_empty() })
            .collect()
    }

    pub fn short(&mut self, x: Contact<T>, y: Contact<T>) -> Result<BTreeSet<Contact<T>>, Error> {
        self.dag.add_edge(x, y)
    }

    pub fn remove_short(
        &mut self,
        x: Contact<T>,
        y: Contact<T>,
    ) -> Result<BTreeSet<Contact<T>>, Error> {
        self.dag.remove_edge(x, y)
    }

    fn _resolve_branch(&mut self, c: Contact<T>) -> Result<T, Error>
    where
        T: Transition<T>,
        Contact<T>: Output<T>,
    {
        let mut final_state: T = c.clone().output().unwrap_or_default();
        if let Some(contacts) = self.dag.connections(c.clone()) {
            if !contacts.is_empty() {
                let state: T = c.input();
                let mut assumed_state: T = c.program();
                let mut state_set: bool = false;
                for contact in contacts.clone() {
                    if self._resolve_branch(contact).unwrap() != assumed_state && !state_set {
                        assumed_state = assumed_state.transition();
                        state_set = true;
                    }
                }
                // If the determined state is not equal to the current state,
                // update the current state with the determined state.
                if state != assumed_state {
                    let mut updated_c: Contact<T> = c.clone();
                    updated_c.reinput(assumed_state).unwrap();
                    self.update(c, updated_c.clone());
                    final_state = updated_c.output().unwrap_or_default();
                }
            }
        }
        // If there are no adjacent vertices, then this node is a leaf node;
        // the state is simply the output of the contact's XOR gate.
        Ok(final_state)
    }
    fn try_str_to_bool(s: String) -> Result<Vec<bool>, Error> {
        let mut pv_vec: Vec<bool> = Vec::new();
        for char in s.chars() {
            if char == '0' {
                pv_vec.push(false);
            } else if char == '1' {
                pv_vec.push(true);
            } else {
                return Err(Error::EdgeExistsError);
            }
        }
        Ok(pv_vec)
    }

    fn bool_to_str(v: Vec<bool>) -> String {
        let mut s: String = String::new();
        for bit in v {
            if !bit {
                s.push('0');
            } else {
                s.push('1');
            }
        }
        s
    }
}

impl<T> Default for BTreeReducer<T>
where
    T: Clone + Ord + Default + Transition<T>,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Input<Vec<T>> for BTreeReducer<T>
where
    T: Clone + Ord + Default + Transition<T>,
{
    fn input(&self) -> Vec<T> {
        self.get_input_contacts()
            .iter()
            .cloned()
            .map(|c| -> T { c.input() })
            .collect()
    }
}

impl Input<String> for BTreeReducer<bool>
where
    Self: Input<Vec<bool>>,
{
    fn input(&self) -> String {
        BTreeReducer::<bool>::bool_to_str(self.input())
    }
}

impl<T> Output<T> for BTreeReducer<T>
where
    T: Clone + Ord + Default + Transition<T>,
    Contact<T>: Output<T>,
{
    type Error = Error;
    fn output(&mut self) -> Result<T, Self::Error> {
        self._resolve_branch(self.root())
    }
}

impl Output<String> for BTreeReducer<bool> {
    type Error = Error;
    fn output(&mut self) -> Result<String, Self::Error> {
        if self._resolve_branch(self.root())? {
            Ok(String::from("1"))
        } else {
            Ok(String::from("0"))
        }
    }
}

impl<T> Reinput<Vec<T>> for BTreeReducer<T>
where
    T: Clone + Ord + Default + Transition<T>,
    Contact<T>: Output<T>,
{
    type Error = Error;
    fn reinput(&mut self, iv: Vec<T>) -> Result<(), Self::Error> {
        let input: Vec<T> = self.input();
        if input.dimension() != iv.dimension() {
            return Err(Error::EdgeExistsError);
        }
        for (vertex, state) in self.get_input_contacts().iter().cloned().zip(iv) {
            if vertex.input() != state {
                let mut updated_vertex = vertex.clone();
                updated_vertex.reinput(state)?;
                self.update(vertex.clone(), updated_vertex);
            }
        }
        Ok(())
    }
}

impl Reinput<String> for BTreeReducer<bool>
    where
        Self: Reinput<Vec<bool>>,
{
    type Error = Error;
    fn reinput(&mut self, ss: String) -> Result<(), Self::Error> {
        let sv: Vec<bool> = BTreeReducer::<bool>::try_str_to_bool(ss)?;
        self.reinput(sv)
    }
}

impl<T> Configuration<Vec<T>> for BTreeReducer<T>
where
    T: Clone + Ord + Default,
{
    fn configuration(&self) -> Vec<T> {
        self.dag
            .vertices()
            .into_iter()
            .map(|c| -> T { c.configuration() })
            .collect()
    }
}

impl Configuration<String> for BTreeReducer<bool> {
    fn configuration(&self) -> String {
        BTreeReducer::<bool>::bool_to_str(self.configuration())
    }
}

impl<T> Program<Vec<T>> for BTreeReducer<T>
where
    T: Clone + Ord + Default,
{
    fn program(&self) -> Vec<T> {
        self.dag
            .vertices()
            .into_iter()
            .map(|c| -> T { c.program() })
            .collect()
    }
}

impl Program<String> for BTreeReducer<bool> {
    fn program(&self) -> String {
        BTreeReducer::<bool>::bool_to_str(self.program())
    }
}

impl<T> Reconfigure<Vec<T>> for BTreeReducer<T>
where
    T: Clone + Ord + Default + Transition<T>,
    Contact<T>: Output<T>,
{
    type Error = Error;
    fn reconfigure(&mut self, cv: Vec<T>) -> Result<(), Self::Error> {
        let configuration: Vec<T> = self.configuration();
        if configuration.dimension() != cv.dimension() {
            return Err(Error::EdgeExistsError);
        }
        for (vertex, state) in self.dag.clone().vertices().into_iter().cloned().zip(cv) {
            if vertex.configuration() != state {
                let mut updated_vertex = vertex.clone();
                updated_vertex.reconfigure(state)?;
                self.update(vertex.clone(), updated_vertex);
            }
        }
        Ok(())
    }
}

impl Reconfigure<String> for BTreeReducer<bool>
    where
        Self: Reconfigure<Vec<bool>>,
{
    type Error = Error;
    fn reconfigure(&mut self, ss: String) -> Result<(), Self::Error> {
        let sv: Vec<bool> = BTreeReducer::<bool>::try_str_to_bool(ss)?;
        self.reconfigure(sv)
    }
}

impl<T> Reprogram<Vec<T>> for BTreeReducer<T>
where
    T: Clone + Ord + Default + Transition<T>,
    Contact<T>: Output<T>,
{
    type Error = Error;
    fn reprogram(&mut self, pv: Vec<T>) -> Result<(), Self::Error> {
        let program: Vec<T> = self.program();
        if program.dimension() != pv.dimension() {
            return Err(Error::EdgeExistsError);
        }
        for (vertex, state) in self.dag.clone().vertices().into_iter().cloned().zip(pv) {
            if vertex.program != state {
                let mut updated_vertex = vertex.clone();
                updated_vertex.reprogram(state)?;
                self.update(vertex.clone(), updated_vertex);
            }
        }
        Ok(())
    }
}

impl Reprogram<String> for BTreeReducer<bool>
where
    Self: Reprogram<Vec<bool>>,
{
    type Error = Error;
    fn reprogram(&mut self, ps: String) -> Result<(), Error> {
        let pv_vec: Vec<bool> = BTreeReducer::<bool>::try_str_to_bool(ps)?;
        self.reprogram(pv_vec)
    }
}

#[cfg(test)]
mod unit_tests {
    use crate::reducer::api::{Configuration, Input, Output, Reconfigure, Reinput, Reprogram, Transition};
    use crate::reducer::{BTreeReducer, Contact};
    use alloc::string::String;
    use alloc::vec::Vec;
    use btree_dag::error::Error;
    use alloc::collections::BTreeSet;

    #[test]
    fn new() {
        let reducer: BTreeReducer<bool> = BTreeReducer::new();
        assert_eq!(reducer, BTreeReducer::default())
    }

    #[test]
    fn input() {
        let reducer: BTreeReducer<bool> = BTreeReducer::new();
        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 1);
        assert!(!input[0])
    }

    #[test]
    fn configuration() {
        let reducer: BTreeReducer<bool> = BTreeReducer::new();
        let configuration: Vec<bool> = reducer.configuration();
        assert_eq!(configuration.len(), 1);
        assert!(!configuration[0])
    }

    #[test]
    fn output() -> Result<(), Error> {
        let mut reducer: BTreeReducer<bool> = BTreeReducer::new();
        let output: bool = reducer.output()?;
        assert!(!output);
        Ok(())
    }

    #[test]
    fn root() {
        let reducer: BTreeReducer<bool> = BTreeReducer::new();
        assert_eq!(
            reducer.root(),
            Contact {
                id: 0,
                input: bool::default(),
                configuration: bool::default(),
                program: bool::default(),
            }
        );
    }

    #[test]
    fn update() -> Result<(), Error> {
        let mut reducer: BTreeReducer<bool> = BTreeReducer::new();
        let mut root = reducer.root();
        assert!(!root.input());
        assert!(!root.configuration());
        assert!(!root.output()?);

        let mut newroot = reducer.root();
        newroot.reinput(true)?;
        reducer.update(reducer.root(), newroot);

        assert!(reducer.root().input());
        assert!(!reducer.root().configuration());
        assert!(reducer.root().output()?);

        let mut newroot = reducer.root();
        newroot.reinput(false)?;
        reducer.update(reducer.root(), newroot);

        let mut newroot = reducer.root();
        newroot.reconfigure(true)?;
        reducer.update(reducer.root(), newroot);

        assert!(!reducer.root().input());
        assert!(reducer.root().configuration());
        assert!(reducer.root().output()?);

        let mut newroot = reducer.root();
        newroot.reconfigure(false)?;
        reducer.update(reducer.root(), newroot);

        assert!(!reducer.root().input());
        assert!(!reducer.root().configuration());
        assert!(!reducer.root().output()?);
        Ok(())
    }

    #[test]
    fn add_contact() -> Result<(), Error> {
        let mut reducer: BTreeReducer<bool> = BTreeReducer::new();
        reducer.add_contact(reducer.root());

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 1);
        assert!(!input[0]);

        let configuration: Vec<bool> = reducer.configuration();
        assert_eq!(configuration.len(), 2);
        assert!(!configuration[0]);
        assert!(!configuration[1]);

        let output: bool = reducer.output()?;
        assert!(!output);

        let series = reducer.add_contact(reducer.root());

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(!input[0]);
        assert!(!input[1]);

        let configuration: Vec<bool> = reducer.configuration();
        assert_eq!(configuration.len(), 3);
        assert!(!configuration[0]);
        assert!(!configuration[1]);
        assert!(!configuration[2]);

        let output: bool = reducer.output()?;
        assert!(!output);

        reducer.add_contact(series);

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(!input[0]);
        assert!(!input[1]);

        let configuration: Vec<bool> = reducer.configuration();
        assert_eq!(configuration.len(), 4);
        assert!(!configuration[0]);
        assert!(!configuration[1]);
        assert!(!configuration[2]);
        assert!(!configuration[3]);

        let output: bool = reducer.output()?;
        assert!(!output);
        Ok(())
    }

    #[test]
    fn reinput() -> Result<(), Error> {
        let mut reducer: BTreeReducer<bool> = BTreeReducer::new();
        reducer.add_contact(reducer.root());

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 1);
        assert!(!input[0]);

        let mut iv: Vec<bool> = Vec::new();
        iv.push(true);
        iv.push(true);
        assert!(reducer.reinput(iv).is_err());

        let mut iv: Vec<bool> = Vec::new();
        iv.push(true);
        reducer.reinput(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 1);
        assert!(input[0]);

        let mut iv: Vec<bool> = Vec::new();
        iv.push(true);
        reducer.reinput(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 1);
        assert!(input[0]);

        let mut iv: Vec<bool> = Vec::new();
        iv.push(false);
        reducer.reinput(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 1);
        assert!(!input[0]);

        let mut reducer: BTreeReducer<bool> = BTreeReducer::new();
        reducer.add_contact(reducer.root());
        reducer.add_contact(reducer.root());

        let mut iv: Vec<bool> = Vec::new();
        iv.push(true);
        assert!(reducer.reinput(iv).is_err());

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(!input[0]);
        assert!(!input[1]);

        let mut iv: Vec<bool> = Vec::new();
        iv.push(true);
        iv.push(true);
        reducer.reinput(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(input[0]);
        assert!(input[1]);

        let mut iv: Vec<bool> = Vec::new();
        iv.push(true);
        iv.push(true);
        reducer.reinput(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(input[0]);
        assert!(input[1]);

        let mut iv: Vec<bool> = Vec::new();
        iv.push(false);
        iv.push(true);
        reducer.reinput(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(!input[0]);
        assert!(input[1]);

        let mut iv: Vec<bool> = Vec::new();
        iv.push(true);
        iv.push(false);
        reducer.reinput(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(input[0]);
        assert!(!input[1]);

        let mut iv: Vec<bool> = Vec::new();
        iv.push(false);
        iv.push(false);
        reducer.reinput(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(!input[0]);
        assert!(!input[1]);
        Ok(())
    }

    #[test]
    fn reconfigure() -> Result<(), Error> {
        let mut reducer: BTreeReducer<bool> = BTreeReducer::new();
        reducer.add_contact(reducer.root());

        let configuration: Vec<bool> = reducer.configuration();
        assert_eq!(configuration.len(), 2);
        assert!(!configuration[0]);
        assert!(!configuration[1]);

        let mut sv: Vec<bool> = Vec::new();
        sv.push(true);
        sv.push(true);
        sv.push(true);
        assert!(reducer.reconfigure(sv).is_err());

        let mut sv: Vec<bool> = Vec::new();
        sv.push(true);
        assert!(reducer.reconfigure(sv).is_err());

        let mut sv: Vec<bool> = Vec::new();
        sv.push(true);
        sv.push(true);
        reducer.reconfigure(sv)?;

        let configuration: Vec<bool> = reducer.configuration();
        assert_eq!(configuration.len(), 2);
        assert!(configuration[0]);
        assert!(configuration[1]);

        let mut sv: Vec<bool> = Vec::new();
        sv.push(false);
        sv.push(true);
        reducer.reconfigure(sv)?;

        let configuration: Vec<bool> = reducer.configuration();
        assert_eq!(configuration.len(), 2);
        assert!(!configuration[0]);
        assert!(configuration[1]);

        let mut sv: Vec<bool> = Vec::new();
        sv.push(false);
        sv.push(false);
        reducer.reconfigure(sv)?;

        let configuration: Vec<bool> = reducer.configuration();
        assert_eq!(configuration.len(), 2);
        assert!(!configuration[0]);
        assert!(!configuration[1]);

        let mut sv: Vec<bool> = Vec::new();
        sv.push(false);
        sv.push(false);
        reducer.reconfigure(sv)?;

        let configuration: Vec<bool> = reducer.configuration();
        assert_eq!(configuration.len(), 2);
        assert!(!configuration[0]);
        assert!(!configuration[1]);

        let mut sv: Vec<bool> = Vec::new();
        sv.push(true);
        sv.push(false);
        reducer.reconfigure(sv)?;

        let configuration: Vec<bool> = reducer.configuration();
        assert_eq!(configuration.len(), 2);
        assert!(configuration[0]);
        assert!(!configuration[1]);

        Ok(())
    }

    #[test]
    fn and_truth_table() -> Result<(), Error> {
        let mut reducer: BTreeReducer<bool> = BTreeReducer::new();
        let series = reducer.add_contact(reducer.root());
        reducer.add_contact(series.clone());
        reducer.add_contact(series);

        let mut pv: Vec<bool> = Vec::new();
        pv.push(false);
        pv.push(true);
        pv.push(false);
        pv.push(false);
        reducer.reprogram(pv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(!input[0]);
        assert!(!input[1]);

        let configuration: Vec<bool> = reducer.configuration();
        assert_eq!(configuration.len(), 4);
        assert!(!configuration[0]);
        assert!(!configuration[1]);
        assert!(!configuration[2]);
        assert!(!configuration[3]);

        let output: bool = reducer.output()?;
        assert!(!output);

        let mut iv: Vec<bool> = Vec::new();
        iv.push(true);
        iv.push(false);
        reducer.reinput(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(input[0]);
        assert!(!input[1]);

        let configuration: Vec<bool> = reducer.configuration();
        assert_eq!(configuration.len(), 4);
        assert!(!configuration[0]);
        assert!(!configuration[1]);
        assert!(!configuration[2]);
        assert!(!configuration[3]);

        let output: bool = reducer.output()?;
        assert!(!output);

        let mut iv: Vec<bool> = Vec::new();
        iv.push(true);
        iv.push(true);
        reducer.reinput(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(input[0]);
        assert!(input[1]);

        let configuration: Vec<bool> = reducer.configuration();
        assert_eq!(configuration.len(), 4);
        assert!(!configuration[0]);
        assert!(!configuration[1]);
        assert!(!configuration[2]);
        assert!(!configuration[3]);

        let output: bool = reducer.output()?;
        assert!(output);

        let mut iv: Vec<bool> = Vec::new();
        iv.push(false);
        iv.push(true);
        reducer.reinput(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(!input[0]);
        assert!(input[1]);

        let configuration: Vec<bool> = reducer.configuration();
        assert_eq!(configuration.len(), 4);
        assert!(!configuration[0]);
        assert!(!configuration[1]);
        assert!(!configuration[2]);
        assert!(!configuration[3]);

        let output: bool = reducer.output()?;
        assert!(!output);
        Ok(())
    }

    #[test]
    fn nand_truth_table() -> Result<(), Error> {
        let mut reducer: BTreeReducer<bool> = BTreeReducer::new();
        let series = reducer.add_contact(reducer.root());
        reducer.add_contact(series.clone());
        reducer.add_contact(series);

        let mut pv: Vec<bool> = Vec::new();
        pv.push(false);
        pv.push(true);
        pv.push(false);
        pv.push(false);
        reducer.reprogram(pv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(!input[0]);
        assert!(!input[1]);

        let configuration: Vec<bool> = reducer.configuration();
        assert_eq!(configuration.len(), 4);
        assert!(!configuration[0]);
        assert!(!configuration[1]);
        assert!(!configuration[2]);
        assert!(!configuration[3]);

        let output: bool = reducer.output()?;
        assert!(!output);

        let mut sv: Vec<bool> = Vec::new();
        sv.push(true);
        sv.push(false);
        sv.push(false);
        sv.push(false);
        reducer.reconfigure(sv)?;

        let mut iv: Vec<bool> = Vec::new();
        iv.push(true);
        iv.push(false);
        reducer.reinput(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(input[0]);
        assert!(!input[1]);

        let configuration: Vec<bool> = reducer.configuration();
        assert_eq!(configuration.len(), 4);
        assert!(configuration[0]);
        assert!(!configuration[1]);
        assert!(!configuration[2]);
        assert!(!configuration[3]);

        let output: bool = reducer.output()?;
        assert!(output);

        let mut iv: Vec<bool> = Vec::new();
        iv.push(true);
        iv.push(true);
        reducer.reinput(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(input[0]);
        assert!(input[1]);

        let configuration: Vec<bool> = reducer.configuration();
        assert_eq!(configuration.len(), 4);
        assert!(configuration[0]);
        assert!(!configuration[1]);
        assert!(!configuration[2]);
        assert!(!configuration[3]);

        let output: bool = reducer.output()?;
        assert!(!output);

        let mut iv: Vec<bool> = Vec::new();
        iv.push(false);
        iv.push(true);
        reducer.reinput(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(!input[0]);
        assert!(input[1]);

        let configuration: Vec<bool> = reducer.configuration();
        assert_eq!(configuration.len(), 4);
        assert!(configuration[0]);
        assert!(!configuration[1]);
        assert!(!configuration[2]);
        assert!(!configuration[3]);

        let output: bool = reducer.output()?;
        assert!(output);
        Ok(())
    }

    #[test]
    fn or_truth_table() -> Result<(), Error> {
        let mut reducer: BTreeReducer<bool> = BTreeReducer::new();
        let parallel = reducer.add_contact(reducer.root());
        reducer.add_contact(parallel.clone());
        reducer.add_contact(parallel);

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(!input[0]);
        assert!(!input[1]);

        let configuration: Vec<bool> = reducer.configuration();
        assert_eq!(configuration.len(), 4);
        assert!(!configuration[0]);
        assert!(!configuration[1]);
        assert!(!configuration[2]);
        assert!(!configuration[3]);

        let output: bool = reducer.output()?;
        assert!(!output);

        let mut iv: Vec<bool> = Vec::new();
        iv.push(true);
        iv.push(false);
        reducer.reinput(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(input[0]);
        assert!(!input[1]);

        let configuration: Vec<bool> = reducer.configuration();
        assert_eq!(configuration.len(), 4);
        assert!(!configuration[0]);
        assert!(!configuration[1]);
        assert!(!configuration[2]);
        assert!(!configuration[3]);

        let output: bool = reducer.output()?;
        assert!(output);

        let mut iv: Vec<bool> = Vec::new();
        iv.push(true);
        iv.push(true);
        reducer.reinput(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(input[0]);
        assert!(input[1]);

        let configuration: Vec<bool> = reducer.configuration();
        assert_eq!(configuration.len(), 4);
        assert!(!configuration[0]);
        assert!(!configuration[1]);
        assert!(!configuration[2]);
        assert!(!configuration[3]);

        let output: bool = reducer.output()?;
        assert!(output);

        let mut iv: Vec<bool> = Vec::new();
        iv.push(false);
        iv.push(true);
        reducer.reinput(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(!input[0]);
        assert!(input[1]);

        let configuration: Vec<bool> = reducer.configuration();
        assert_eq!(configuration.len(), 4);
        assert!(!configuration[0]);
        assert!(!configuration[1]);
        assert!(!configuration[2]);
        assert!(!configuration[3]);

        let output: bool = reducer.output()?;
        assert!(output);
        Ok(())
    }

    #[test]
    fn nor_truth_table() -> Result<(), Error> {
        let mut reducer: BTreeReducer<bool> = BTreeReducer::new();
        let parallel = reducer.add_contact(reducer.root());
        reducer.add_contact(parallel.clone());
        reducer.add_contact(parallel);

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(!input[0]);
        assert!(!input[1]);

        let configuration: Vec<bool> = reducer.configuration();
        assert_eq!(configuration.len(), 4);
        assert!(!configuration[0]);
        assert!(!configuration[1]);
        assert!(!configuration[2]);
        assert!(!configuration[3]);

        let output: bool = reducer.output()?;
        assert!(!output);

        let mut sv: Vec<bool> = Vec::new();
        sv.push(true);
        sv.push(false);
        sv.push(false);
        sv.push(false);
        reducer.reconfigure(sv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(!input[0]);
        assert!(!input[1]);

        let configuration: Vec<bool> = reducer.configuration();
        assert_eq!(configuration.len(), 4);
        assert!(configuration[0]);
        assert!(!configuration[1]);
        assert!(!configuration[2]);
        assert!(!configuration[3]);

        let output: bool = reducer.output()?;
        assert!(output);

        let mut sv: Vec<bool> = Vec::new();
        sv.push(false);
        sv.push(true);
        sv.push(false);
        sv.push(false);
        reducer.reconfigure(sv)?;

        let mut iv: Vec<bool> = Vec::new();
        iv.push(true);
        iv.push(false);
        reducer.reinput(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(input[0]);
        assert!(!input[1]);

        let configuration: Vec<bool> = reducer.configuration();
        assert_eq!(configuration.len(), 4);
        assert!(!configuration[0]);
        assert!(configuration[1]);
        assert!(!configuration[2]);
        assert!(!configuration[3]);

        let output: bool = reducer.output()?;
        assert!(!output);

        let mut iv: Vec<bool> = Vec::new();
        iv.push(true);
        iv.push(true);
        reducer.reinput(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(input[0]);
        assert!(input[1]);

        let configuration: Vec<bool> = reducer.configuration();
        assert_eq!(configuration.len(), 4);
        assert!(!configuration[0]);
        assert!(configuration[1]);
        assert!(!configuration[2]);
        assert!(!configuration[3]);

        let output: bool = reducer.output()?;
        assert!(!output);

        let mut iv: Vec<bool> = Vec::new();
        iv.push(false);
        iv.push(true);
        reducer.reinput(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(!input[0]);
        assert!(input[1]);

        let configuration: Vec<bool> = reducer.configuration();
        assert_eq!(configuration.len(), 4);
        assert!(!configuration[0]);
        assert!(configuration[1]);
        assert!(!configuration[2]);
        assert!(!configuration[3]);

        let output: bool = reducer.output()?;
        assert!(!output);
        Ok(())
    }

    #[test]
    fn xor_truth_table() -> Result<(), Error> {
        let mut reducer: BTreeReducer<bool> = BTreeReducer::new();
        let series_0 = reducer.add_contact(reducer.root());
        let parallel_1 = reducer.add_contact(series_0.clone());
        let series_1 = reducer.add_contact(series_0.clone());
        let input_0 = reducer.add_contact(parallel_1.clone());
        let input_1 = reducer.add_contact(parallel_1.clone());
        reducer.short(series_1.clone(), input_0)?;
        reducer.short(series_1, input_1)?;

        let mut pv: Vec<bool> = Vec::new();
        pv.push(false);
        pv.push(true);
        pv.push(false);
        pv.push(true);
        pv.push(false);
        pv.push(false);
        reducer.reprogram(pv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(!input[0]);
        assert!(!input[1]);

        let configuration: Vec<bool> = reducer.configuration();
        assert_eq!(configuration.len(), 6);
        assert!(!configuration[0]);
        assert!(!configuration[1]);
        assert!(!configuration[2]);
        assert!(!configuration[3]);
        assert!(!configuration[4]);
        assert!(!configuration[5]);

        let output: bool = reducer.output()?;
        assert!(!output);

        let mut sv: Vec<bool> = Vec::new();
        sv.push(false);
        sv.push(false);
        sv.push(false);
        sv.push(true);
        sv.push(false);
        sv.push(false);
        reducer.reconfigure(sv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(!input[0]);
        assert!(!input[1]);

        let configuration: Vec<bool> = reducer.configuration();
        assert_eq!(configuration.len(), 6);
        assert!(!configuration[0]);
        assert!(!configuration[1]);
        assert!(!configuration[2]);
        assert!(configuration[3]);
        assert!(!configuration[4]);
        assert!(!configuration[5]);

        let output: bool = reducer.output()?;
        assert!(!output);

        let mut iv: Vec<bool> = Vec::new();
        iv.push(true);
        iv.push(false);
        reducer.reinput(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(input[0]);
        assert!(!input[1]);

        let configuration: Vec<bool> = reducer.configuration();
        assert_eq!(configuration.len(), 6);
        assert!(!configuration[0]);
        assert!(!configuration[1]);
        assert!(!configuration[2]);
        assert!(configuration[3]);
        assert!(!configuration[4]);
        assert!(!configuration[5]);

        let output: bool = reducer.output()?;
        assert!(output);

        let mut iv: Vec<bool> = Vec::new();
        iv.push(false);
        iv.push(true);
        reducer.reinput(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(!input[0]);
        assert!(input[1]);

        let configuration: Vec<bool> = reducer.configuration();
        assert_eq!(configuration.len(), 6);
        assert!(!configuration[0]);
        assert!(!configuration[1]);
        assert!(!configuration[2]);
        assert!(configuration[3]);
        assert!(!configuration[4]);
        assert!(!configuration[5]);

        let output: bool = reducer.output()?;
        assert!(output);

        let mut iv: Vec<bool> = Vec::new();
        iv.push(false);
        iv.push(false);
        reducer.reinput(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(!input[0]);
        assert!(!input[1]);

        let configuration: Vec<bool> = reducer.configuration();
        assert_eq!(configuration.len(), 6);
        assert!(!configuration[0]);
        assert!(!configuration[1]);
        assert!(!configuration[2]);
        assert!(configuration[3]);
        assert!(!configuration[4]);
        assert!(!configuration[5]);

        let output: bool = reducer.output()?;
        assert!(!output);
        Ok(())
    }

    #[test]
    fn xnor_truth_table() -> Result<(), Error> {
        let mut reducer: BTreeReducer<bool> = BTreeReducer::new();
        let series_0 = reducer.add_contact(reducer.root());
        let parallel_1 = reducer.add_contact(series_0.clone());
        let series_1 = reducer.add_contact(series_0.clone());
        let input_0 = reducer.add_contact(parallel_1.clone());
        let input_1 = reducer.add_contact(parallel_1.clone());
        reducer.short(series_1.clone(), input_0)?;
        reducer.short(series_1, input_1)?;

        let mut pv: Vec<bool> = Vec::new();
        pv.push(false);
        pv.push(true);
        pv.push(false);
        pv.push(true);
        pv.push(false);
        pv.push(false);
        reducer.reprogram(pv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(!input[0]);
        assert!(!input[1]);

        let configuration: Vec<bool> = reducer.configuration();
        assert_eq!(configuration.len(), 6);
        assert!(!configuration[0]);
        assert!(!configuration[1]);
        assert!(!configuration[2]);
        assert!(!configuration[3]);
        assert!(!configuration[4]);
        assert!(!configuration[5]);

        let output: bool = reducer.output()?;
        assert!(!output);

        let mut sv: Vec<bool> = Vec::new();
        sv.push(true);
        sv.push(false);
        sv.push(false);
        sv.push(true);
        sv.push(false);
        sv.push(false);
        reducer.reconfigure(sv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(!input[0]);
        assert!(!input[1]);

        let configuration: Vec<bool> = reducer.configuration();
        assert_eq!(configuration.len(), 6);
        assert!(configuration[0]);
        assert!(!configuration[1]);
        assert!(!configuration[2]);
        assert!(configuration[3]);
        assert!(!configuration[4]);
        assert!(!configuration[5]);

        let output: bool = reducer.output()?;
        assert!(output);

        let mut iv: Vec<bool> = Vec::new();
        iv.push(true);
        iv.push(false);
        reducer.reinput(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(input[0]);
        assert!(!input[1]);

        let configuration: Vec<bool> = reducer.configuration();
        assert_eq!(configuration.len(), 6);
        assert!(configuration[0]);
        assert!(!configuration[1]);
        assert!(!configuration[2]);
        assert!(configuration[3]);
        assert!(!configuration[4]);
        assert!(!configuration[5]);

        let output: bool = reducer.output()?;
        assert!(!output);

        let mut iv: Vec<bool> = Vec::new();
        iv.push(false);
        iv.push(true);
        reducer.reinput(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(!input[0]);
        assert!(input[1]);

        let configuration: Vec<bool> = reducer.configuration();
        assert_eq!(configuration.len(), 6);
        assert!(configuration[0]);
        assert!(!configuration[1]);
        assert!(!configuration[2]);
        assert!(configuration[3]);
        assert!(!configuration[4]);
        assert!(!configuration[5]);

        let output: bool = reducer.output()?;
        assert!(!output);

        let mut iv: Vec<bool> = Vec::new();
        iv.push(false);
        iv.push(false);
        reducer.reinput(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(!input[0]);
        assert!(!input[1]);

        let configuration: Vec<bool> = reducer.configuration();
        assert_eq!(configuration.len(), 6);
        assert!(configuration[0]);
        assert!(!configuration[1]);
        assert!(!configuration[2]);
        assert!(configuration[3]);
        assert!(!configuration[4]);
        assert!(!configuration[5]);

        let output: bool = reducer.output()?;
        assert!(output);
        Ok(())
    }

    #[test]
    fn and_truth_table_trans_nand_truth_table() -> Result<(), Error> {
        let mut reducer: BTreeReducer<bool> = BTreeReducer::new();
        let series = reducer.add_contact(reducer.root());
        reducer.add_contact(series.clone());
        reducer.add_contact(series);

        let mut pv: Vec<bool> = Vec::new();
        pv.push(false);
        pv.push(true);
        pv.push(true);
        pv.push(true);
        reducer.reprogram(pv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(!input[0]);
        assert!(!input[1]);

        let configuration: Vec<bool> = reducer.configuration();
        assert_eq!(configuration.len(), 4);
        assert!(!configuration[0]);
        assert!(!configuration[1]);
        assert!(!configuration[2]);
        assert!(!configuration[3]);

        let output: bool = reducer.output()?;
        assert!(!output);

        let mut iv: Vec<bool> = Vec::new();
        iv.push(true);
        iv.push(false);
        reducer.reinput(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(input[0]);
        assert!(!input[1]);

        let configuration: Vec<bool> = reducer.configuration();
        assert_eq!(configuration.len(), 4);
        assert!(!configuration[0]);
        assert!(!configuration[1]);
        assert!(!configuration[2]);
        assert!(!configuration[3]);

        let output: bool = reducer.output()?;
        assert!(!output);

        let mut iv: Vec<bool> = Vec::new();
        iv.push(true);
        iv.push(true);
        reducer.reinput(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(input[0]);
        assert!(input[1]);

        let configuration: Vec<bool> = reducer.configuration();
        assert_eq!(configuration.len(), 4);
        assert!(!configuration[0]);
        assert!(!configuration[1]);
        assert!(!configuration[2]);
        assert!(!configuration[3]);

        let output: bool = reducer.output()?;
        assert!(output);

        let mut iv: Vec<bool> = Vec::new();
        iv.push(false);
        iv.push(true);
        reducer.reinput(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(!input[0]);
        assert!(input[1]);

        let configuration: Vec<bool> = reducer.configuration();
        assert_eq!(configuration.len(), 4);
        assert!(!configuration[0]);
        assert!(!configuration[1]);
        assert!(!configuration[2]);
        assert!(!configuration[3]);

        let output: bool = reducer.output()?;
        assert!(!output);

        let mut iv: Vec<bool> = Vec::new();
        iv.push(false);
        iv.push(false);
        reducer.reinput(iv)?;

        let mut sv: Vec<bool> = Vec::new();
        sv.push(true);
        sv.push(false);
        sv.push(false);
        sv.push(false);
        reducer.reconfigure(sv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(!input[0]);
        assert!(!input[1]);

        let configuration: Vec<bool> = reducer.configuration();
        assert_eq!(configuration.len(), 4);
        assert!(configuration[0]);
        assert!(!configuration[1]);
        assert!(!configuration[2]);
        assert!(!configuration[3]);

        let output: bool = reducer.output()?;
        assert!(output);

        let mut iv: Vec<bool> = Vec::new();
        iv.push(true);
        iv.push(false);
        reducer.reinput(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(input[0]);
        assert!(!input[1]);

        let configuration: Vec<bool> = reducer.configuration();
        assert_eq!(configuration.len(), 4);
        assert!(configuration[0]);
        assert!(!configuration[1]);
        assert!(!configuration[2]);
        assert!(!configuration[3]);

        let output: bool = reducer.output()?;
        assert!(output);

        let mut iv: Vec<bool> = Vec::new();
        iv.push(true);
        iv.push(true);
        reducer.reinput(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(input[0]);
        assert!(input[1]);

        let configuration: Vec<bool> = reducer.configuration();
        assert_eq!(configuration.len(), 4);
        assert!(configuration[0]);
        assert!(!configuration[1]);
        assert!(!configuration[2]);
        assert!(!configuration[3]);

        let output: bool = reducer.output()?;
        assert!(!output);

        let mut iv: Vec<bool> = Vec::new();
        iv.push(false);
        iv.push(true);
        reducer.reinput(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(!input[0]);
        assert!(input[1]);

        let configuration: Vec<bool> = reducer.configuration();
        assert_eq!(configuration.len(), 4);
        assert!(configuration[0]);
        assert!(!configuration[1]);
        assert!(!configuration[2]);
        assert!(!configuration[3]);

        let output: bool = reducer.output()?;
        assert!(output);

        Ok(())
    }

    #[test]
    fn xor_truth_table_trans_xnor_truth_table() -> Result<(), Error> {
        let mut reducer: BTreeReducer<bool> = BTreeReducer::new();
        let series_0 = reducer.add_contact(reducer.root());
        let parallel_1 = reducer.add_contact(series_0.clone());
        let series_1 = reducer.add_contact(series_0.clone());
        let input_0 = reducer.add_contact(parallel_1.clone());
        let input_1 = reducer.add_contact(parallel_1.clone());
        reducer.short(series_1.clone(), input_0)?;
        reducer.short(series_1, input_1)?;

        let mut pv: Vec<bool> = Vec::new();
        pv.push(false);
        pv.push(true);
        pv.push(false);
        pv.push(true);
        pv.push(false);
        pv.push(false);
        reducer.reprogram(pv)?;

        let mut sv: Vec<bool> = Vec::new();
        sv.push(false);
        sv.push(false);
        sv.push(false);
        sv.push(true);
        sv.push(false);
        sv.push(false);
        reducer.reconfigure(sv)?;

        // 00 -> 0
        let mut iv: Vec<bool> = Vec::new();
        iv.push(false);
        iv.push(false);
        reducer.reinput(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(!input[0]);
        assert!(!input[1]);

        let configuration: Vec<bool> = reducer.configuration();
        assert_eq!(configuration.len(), 6);
        assert!(!configuration[0]);
        assert!(!configuration[1]);
        assert!(!configuration[2]);
        assert!(configuration[3]);
        assert!(!configuration[4]);
        assert!(!configuration[5]);

        let output: bool = reducer.output()?;
        assert!(!output);

        // 10 -> 1
        let mut iv: Vec<bool> = Vec::new();
        iv.push(true);
        iv.push(false);
        reducer.reinput(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(input[0]);
        assert!(!input[1]);

        let configuration: Vec<bool> = reducer.configuration();
        assert_eq!(configuration.len(), 6);
        assert!(!configuration[0]);
        assert!(!configuration[1]);
        assert!(!configuration[2]);
        assert!(configuration[3]);
        assert!(!configuration[4]);
        assert!(!configuration[5]);

        let output: bool = reducer.output()?;
        assert!(output);

        // 01 -> 1
        let mut iv: Vec<bool> = Vec::new();
        iv.push(false);
        iv.push(true);
        reducer.reinput(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(!input[0]);
        assert!(input[1]);

        let configuration: Vec<bool> = reducer.configuration();
        assert_eq!(configuration.len(), 6);
        assert!(!configuration[0]);
        assert!(!configuration[1]);
        assert!(!configuration[2]);
        assert!(configuration[3]);
        assert!(!configuration[4]);
        assert!(!configuration[5]);

        let output: bool = reducer.output()?;
        assert!(output);

        // 11 -> 0
        let mut iv: Vec<bool> = Vec::new();
        iv.push(true);
        iv.push(true);
        reducer.reinput(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(input[0]);
        assert!(input[1]);

        let configuration: Vec<bool> = reducer.configuration();
        assert_eq!(configuration.len(), 6);
        assert!(!configuration[0]);
        assert!(!configuration[1]);
        assert!(!configuration[2]);
        assert!(configuration[3]);
        assert!(!configuration[4]);
        assert!(!configuration[5]);

        let output: bool = reducer.output()?;
        assert!(!output);

        // XOR -> XNOR
        let mut sv: Vec<bool> = Vec::new();
        sv.push(true);
        sv.push(false);
        sv.push(false);
        sv.push(true);
        sv.push(false);
        sv.push(false);
        reducer.reconfigure(sv)?;

        // 00 -> 1
        let mut iv: Vec<bool> = Vec::new();
        iv.push(false);
        iv.push(false);
        reducer.reinput(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(!input[0]);
        assert!(!input[1]);

        let configuration: Vec<bool> = reducer.configuration();
        assert_eq!(configuration.len(), 6);
        assert!(configuration[0]);
        assert!(!configuration[1]);
        assert!(!configuration[2]);
        assert!(configuration[3]);
        assert!(!configuration[4]);
        assert!(!configuration[5]);

        let output: bool = reducer.output()?;
        assert!(output);

        // 10 -> 0
        let mut iv: Vec<bool> = Vec::new();
        iv.push(true);
        iv.push(false);
        reducer.reinput(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(input[0]);
        assert!(!input[1]);

        let configuration: Vec<bool> = reducer.configuration();
        assert_eq!(configuration.len(), 6);
        assert!(configuration[0]);
        assert!(!configuration[1]);
        assert!(!configuration[2]);
        assert!(configuration[3]);
        assert!(!configuration[4]);
        assert!(!configuration[5]);

        let output: bool = reducer.output()?;
        assert!(!output);

        // 01 -> 0
        let mut iv: Vec<bool> = Vec::new();
        iv.push(false);
        iv.push(true);
        reducer.reinput(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(!input[0]);
        assert!(input[1]);

        let configuration: Vec<bool> = reducer.configuration();
        assert_eq!(configuration.len(), 6);
        assert!(configuration[0]);
        assert!(!configuration[1]);
        assert!(!configuration[2]);
        assert!(configuration[3]);
        assert!(!configuration[4]);
        assert!(!configuration[5]);

        let output: bool = reducer.output()?;
        assert!(!output);

        // 11 -> 1
        let mut iv: Vec<bool> = Vec::new();
        iv.push(true);
        iv.push(true);
        reducer.reinput(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(input[0]);
        assert!(input[1]);

        let configuration: Vec<bool> = reducer.configuration();
        assert_eq!(configuration.len(), 6);
        assert!(configuration[0]);
        assert!(!configuration[1]);
        assert!(!configuration[2]);
        assert!(configuration[3]);
        assert!(!configuration[4]);
        assert!(!configuration[5]);

        let output: bool = reducer.output()?;
        assert!(output);

        Ok(())
    }

    #[test]
    fn xor_truth_table_trans_xnor_truth_table_string() -> Result<(), Error> {
        let mut reducer: BTreeReducer<bool> = BTreeReducer::new();
        let series_0 = reducer.add_contact(reducer.root());
        let parallel_1 = reducer.add_contact(series_0.clone());
        let series_1 = reducer.add_contact(series_0.clone());
        let input_0 = reducer.add_contact(parallel_1.clone());
        let input_1 = reducer.add_contact(parallel_1.clone());
        reducer.short(series_1.clone(), input_0)?;
        reducer.short(series_1, input_1)?;

        let ps: String = String::from("010100");
        reducer.reprogram(ps)?;

        let ss: String = String::from("000100");
        reducer.reconfigure(ss)?;

        // 00 -> 0
        let is: String = String::from("00");
        reducer.reinput(is)?;

        let input: String = reducer.input();
        assert_eq!(input.as_str(), "00");

        let configuration: String = reducer.configuration();
        assert_eq!(configuration.as_str(), "000100");

        let output: String = reducer.output()?;
        assert_eq!(output, "0");

        // 10 -> 1
        let is: String = String::from("10");
        reducer.reinput(is)?;

        let input: String = reducer.input();
        assert_eq!(input.as_str(), "10");

        let configuration: String = reducer.configuration();
        assert_eq!(configuration.as_str(), "000100");

        let output: String = reducer.output()?;
        assert_eq!(output, "1");

        // 01 -> 1
        let is: String = String::from("01");
        reducer.reinput(is)?;

        let input: String = reducer.input();
        assert_eq!(input.as_str(), "01");

        let configuration: String = reducer.configuration();
        assert_eq!(configuration.as_str(), "000100");

        let output: String = reducer.output()?;
        assert_eq!(output, "1");

        // 11 -> 0
        let is: String = String::from("11");
        reducer.reinput(is)?;

        let input: String = reducer.input();
        assert_eq!(input.as_str(), "11");

        let configuration: String = reducer.configuration();
        assert_eq!(configuration.as_str(), "000100");

        let output: String = reducer.output()?;
        assert_eq!(output, "0");

        // XOR -> XNOR
        let ss: String = String::from("100100");
        reducer.reconfigure(ss)?;

        // 00 -> 1
        let is: String = String::from("00");
        reducer.reinput(is)?;

        let input: String = reducer.input();
        assert_eq!(input.as_str(), "00");

        let configuration: String = reducer.configuration();
        assert_eq!(configuration.as_str(), "100100");

        let output: String = reducer.output()?;
        assert_eq!(output, "1");

        // 10 -> 0
        let is: String = String::from("10");
        reducer.reinput(is)?;

        let input: String = reducer.input();
        assert_eq!(input.as_str(), "10");

        let configuration: String = reducer.configuration();
        assert_eq!(configuration.as_str(), "100100");

        let output: String = reducer.output()?;
        assert_eq!(output, "0");

        // 01 -> 0
        let is: String = String::from("01");
        reducer.reinput(is)?;

        let input: String = reducer.input();
        assert_eq!(input.as_str(), "01");

        let configuration: String = reducer.configuration();
        assert_eq!(configuration.as_str(), "100100");

        let output: String = reducer.output()?;
        assert_eq!(output, "0");

        // 11 -> 1
        let is: String = String::from("11");
        reducer.reinput(is)?;

        let input: String = reducer.input();
        assert_eq!(input.as_str(), "11");

        let configuration: String = reducer.configuration();
        assert_eq!(configuration.as_str(), "100100");

        let output: String = reducer.output()?;
        assert_eq!(output, "1");

        Ok(())
    }

    #[test]
    fn vowels() -> Result<(), Error> {

        impl Transition<char> for char {
            fn transition(&self) -> char {
                'y'
            }
        }

        impl Output<char> for Contact<char> {
            type Error = Error;
            fn output(&mut self) -> Result<char, Self::Error> {
                let mut vowels: BTreeSet<char> = BTreeSet::new();
                vowels.insert('a');
                vowels.insert('e');
                vowels.insert('i');
                vowels.insert('o');
                vowels.insert('u');
                vowels.insert('y');
                if vowels.contains(&self.input) {
                    Ok('y')
                } else {
                    Ok('n')
                }
            }
        }

        let mut reducer: BTreeReducer<char> = BTreeReducer::new();
        reducer.add_contact(reducer.root());
        reducer.add_contact(reducer.root());
        reducer.add_contact(reducer.root());

        let mut input: Vec<char> = Vec::new();
        input.push('f');
        input.push('o');
        input.push('x');
        reducer.reinput(input.clone())?;

        let mut program: Vec<char> = Vec::new();
        program.push('n');
        program.push('\0');
        program.push('\0');
        program.push('\0');
        reducer.reprogram(program)?;

        assert_eq!(reducer.output()?, 'y');

        let mut input: Vec<char> = Vec::new();
        input.push('c');
        input.push('a');
        input.push('t');
        reducer.reinput(input.clone())?;

        assert_eq!(reducer.output()?, 'y');

        let mut input: Vec<char> = Vec::new();
        input.push('p');
        input.push('s');
        input.push('m');
        reducer.reinput(input.clone())?;

        assert_eq!(reducer.output()?, 'n');

        let mut input: Vec<char> = Vec::new();
        input.push('i');
        input.push('b');
        input.push('m');
        reducer.reinput(input.clone())?;

        assert_eq!(reducer.output()?, 'y');

        let mut input: Vec<char> = Vec::new();
        input.push('d');
        input.push('o');
        input.push('g');
        reducer.reinput(input.clone())?;

        assert_eq!(reducer.output()?, 'y');

        let mut input: Vec<char> = Vec::new();
        input.push('t');
        input.push('l');
        input.push('s');
        reducer.reinput(input.clone())?;

        assert_eq!(reducer.output()?, 'n');

        Ok(())
    }
}
