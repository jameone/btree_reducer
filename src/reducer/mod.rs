use crate::reducer::api::{
    Configuration, Dimension, Input, Output, Program, Reconfigure, Reinput, Reprogram, Transition,
};
use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec::Vec;
use btree_dag::{AddEdge, AddVertex, BTreeDAG, Connections, RemoveEdge, RemoveVertex, Vertices};
use crate::Error;

mod api;
mod test;

#[derive(PartialEq, PartialOrd, Ord, Eq, Clone, Debug)]
pub struct Gate<T>
where
    T: Default + Ord + Clone,
{
    id: usize,
    input: T,
    configuration: T,
    program: T,
}

impl<T> Input<T> for Gate<T>
where
    T: Default + Ord + Clone,
{
    fn input(&self) -> T {
        self.input.clone()
    }
}

impl<T> Configuration<T> for Gate<T>
where
    T: Default + Ord + Clone,
{
    fn configuration(&self) -> T {
        self.configuration.clone()
    }
}

impl<T> Program<T> for Gate<T>
where
    T: Default + Ord + Clone,
{
    fn program(&self) -> T {
        self.program.clone()
    }
}

impl Output<bool> for Gate<bool> {
    type Error = Error;
    fn output(&mut self) -> bool {
        self.input != self.configuration
    }
}

impl<T> Reinput<T> for Gate<T>
where
    T: Default + Ord + Clone,
{
    type Error = Error;
    fn reinput(&mut self, i: T) -> Result<(), Self::Error> {
        self.input = i;
        Ok(())
    }
}

impl<T> Reconfigure<T> for Gate<T>
where
    T: Default + Ord + Clone,
{
    type Error = Error;
    fn reconfigure(&mut self, c: T) -> Result<(), Self::Error> {
        self.configuration = c;
        Ok(())
    }
}

impl<T> Reprogram<T> for Gate<T>
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
    dag: BTreeDAG<Gate<T>>,
}

impl<T> BTreeReducer<T>
where
    T: Default + Ord + Clone + Transition<T>,
{
    fn new() -> Self {
        let mut dag: BTreeDAG<Gate<T>> = BTreeDAG::new();
        let contact_zero: Gate<T> = Gate {
            id: usize::default(),
            input: T::default(),
            configuration: T::default(),
            program: T::default(),
        };
        dag.add_vertex(contact_zero);
        BTreeReducer { dag }
    }

    pub fn add_gate(&mut self, c: Gate<T>) -> Gate<T>
    where
        Gate<T>: Output<T>,
    {
        let vertices: Vec<&Gate<T>> = self.dag.vertices().into_iter().collect();
        let contact: Gate<T> = Gate {
            id: vertices[vertices.len() - 1].id + 1,
            input: T::default(),
            configuration: T::default(),
            program: T::default(),
        };
        self.dag.add_vertex(contact.clone());
        self.dag.add_edge(c, contact.clone()).unwrap();
        self._resolve_branch(self.root());
        contact
    }

    pub fn root(&self) -> Gate<T> {
        let vertices: Vec<Gate<T>> = self.dag.vertices().into_iter().cloned().collect();
        vertices[0].clone()
    }

    pub fn short(&mut self, x: Gate<T>, y: Gate<T>) -> Result<BTreeSet<Gate<T>>, Error> {
        self.dag.add_edge(x, y)
    }

    pub fn remove_short(
        &mut self,
        x: Gate<T>,
        y: Gate<T>,
    ) -> Result<BTreeSet<Gate<T>>, Error> {
        self.dag.remove_edge(x, y)
    }

    pub fn update(&mut self, p: Gate<T>, u: Gate<T>)
        where
            Gate<T>: Output<T>,
    {
        let previous_parents: BTreeSet<Gate<T>> = self
            .dag
            .vertices()
            .into_iter()
            .cloned()
            .map(|v| -> (Gate<T>, &BTreeSet<Gate<T>>) {
                (v.clone(), self.dag.connections(v).unwrap())
            })
            .filter(|t| -> bool { t.1.contains(&p) })
            .map(|t| -> Gate<T> { t.0 })
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
            self._resolve_branch(previous_parent);
        }
    }

    fn _get_input_contacts(&self) -> Vec<Gate<T>> {
        self.dag
            .vertices()
            .into_iter()
            .cloned()
            .filter(|c| -> bool { self.dag.connections(c.clone()).unwrap().is_empty() })
            .collect()
    }

    fn _resolve_branch(&mut self, c: Gate<T>) -> T
    where
        T: Transition<T>,
        Gate<T>: Output<T>,
    {
        let mut final_state: T = c.clone().output();
        if let Some(contacts) = self.dag.connections(c.clone()) {
            if !contacts.is_empty() {
                let mut program: T = c.program();
                let mut state_set: bool = false;
                for contact in contacts.clone() {
                    if self._resolve_branch(contact) != c.program() && !state_set {
                        program = program.transition();
                        state_set = true;
                    }
                }
                // If the determined state is not equal to the current state,
                // update the current state with the determined state.
                if c.input() != program {
                    let mut updated_c: Gate<T> = c.clone();
                    updated_c.reinput(program).unwrap();
                    self.update(c, updated_c.clone());
                    final_state = updated_c.output();
                }
            }
        }
        // If there are no adjacent vertices, then this node is a leaf node;
        // the state is simply the output of the contact's XOR gate.
        final_state
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
        self._get_input_contacts()
            .iter()
            .cloned()
            .map(|c| -> T { c.input() })
            .collect()
    }
}

pub fn try_str_to_bool(s: String) -> Result<Vec<bool>, Error> {
    let mut pv_vec: Vec<bool> = Vec::new();
    for char in s.chars() {
        if char == '0' {
            pv_vec.push(false);
        } else if char == '1' {
            pv_vec.push(true);
        } else {
            return Err(Error::ImproperDimension);
        }
    }
    Ok(pv_vec)
}

pub fn bool_to_str(v: Vec<bool>) -> String {
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

impl Input<String> for BTreeReducer<bool>
where
    Self: Input<Vec<bool>>,
{
    fn input(&self) -> String {
        bool_to_str(self.input())
    }
}

impl<T> Output<T> for BTreeReducer<T>
where
    T: Clone + Ord + Default + Transition<T>,
    Gate<T>: Output<T>,
{
    type Error = Error;
    fn output(&mut self) -> T {
        self._resolve_branch(self.root())
    }
}

impl Output<String> for BTreeReducer<bool> {
    type Error = Error;
    fn output(&mut self) -> String {
        if self._resolve_branch(self.root()) {
            String::from("1")
        } else {
            String::from("0")
        }
    }
}

impl<T> Reinput<Vec<T>> for BTreeReducer<T>
where
    T: Clone + Ord + Default + Transition<T>,
    Gate<T>: Output<T>,
{
    type Error = Error;
    fn reinput(&mut self, iv: Vec<T>) -> Result<(), Self::Error> {
        let input: Vec<T> = self.input();
        if input.dimension() != iv.dimension() {
            return Err(Error::ImproperDimension);
        }
        for (vertex, state) in self._get_input_contacts().iter().cloned().zip(iv) {
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
        let sv: Vec<bool> = try_str_to_bool(ss)?;
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
        bool_to_str(self.configuration())
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
        bool_to_str(self.program())
    }
}

impl<T> Reconfigure<Vec<T>> for BTreeReducer<T>
where
    T: Clone + Ord + Default + Transition<T>,
    Gate<T>: Output<T>,
{
    type Error = Error;
    fn reconfigure(&mut self, cv: Vec<T>) -> Result<(), Self::Error> {
        let configuration: Vec<T> = self.configuration();
        if configuration.dimension() != cv.dimension() {
            return Err(Error::ImproperDimension);
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
        let sv: Vec<bool> = try_str_to_bool(ss)?;
        self.reconfigure(sv)
    }
}

impl<T> Reprogram<Vec<T>> for BTreeReducer<T>
where
    T: Clone + Ord + Default + Transition<T>,
    Gate<T>: Output<T>,
{
    type Error = Error;
    fn reprogram(&mut self, pv: Vec<T>) -> Result<(), Self::Error> {
        let program: Vec<T> = self.program();
        if program.dimension() != pv.dimension() {
            return Err(Error::ImproperDimension);
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
        let pv_vec: Vec<bool> = try_str_to_bool(ps)?;
        self.reprogram(pv_vec)
    }
}
