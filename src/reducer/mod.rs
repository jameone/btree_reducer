use crate::reducer::api::{Input, Reprogram, State, TransitionInput, Output, TransitionState};
use crate::xor::api::{Configuration, Input as XorInput, Output as XorOutput, Reconfigure, Toggle};
use crate::xor::XOR;
use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec::Vec;
use btree_dag::error::Error;
use btree_dag::{AddEdge, AddVertex, BTreeDAG, Connections, RemoveVertex, Vertices};

mod api;

#[derive(PartialEq, PartialOrd, Ord, Eq, Clone, Debug)]
pub struct Contact {
    id: usize,
    gate: XOR,
    program: bool,
}

impl Reprogram<bool> for Contact {
    type Error = Error;
    fn reprogram(&mut self, p: bool) -> Result<(), Self::Error> {
        self.program = p;
        Ok(())
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct BTreeReducer {
    dag: BTreeDAG<Contact>,
}

impl BTreeReducer {
    fn new() -> Self {
        let mut dag: BTreeDAG<Contact> = BTreeDAG::new();
        let contact_zero: Contact = Contact {
            id: 0,
            gate: XOR::new(),
            program: bool::default(),
        };
        dag.add_vertex(contact_zero);
        BTreeReducer { dag }
    }

    fn add_gate(&mut self, c: Contact) -> Contact {
        let vertices: Vec<&Contact> = self.dag.vertices().into_iter().collect();
        let contact: Contact = Contact {
            id: vertices[vertices.len() - 1].id + 1,
            gate: XOR::new(),
            program: bool::default(),
        };
        self.dag.add_vertex(contact.clone());
        self.dag.add_edge(c, contact.clone()).unwrap();
        self._resolve_state(self.root());
        contact
    }

    pub fn root(&self) -> Contact {
        let vertices: Vec<Contact> = self.dag.vertices().into_iter().cloned().collect();
        vertices[0].clone()
    }

    fn update(&mut self, p: Contact, u: Contact) {
        let previous_parents: BTreeSet<Contact> = self
            .dag
            .vertices()
            .into_iter()
            .cloned()
            .map(|v| -> (Contact, &BTreeSet<Contact>) {
                (v.clone(), self.dag.connections(v).unwrap())
            })
            .filter(|t| -> bool { t.1.contains(&p) })
            .map(|t| -> Contact { t.0 })
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
            self._resolve_state(previous_parent);
        }
    }

    fn get_input_contacts(&self) -> Vec<Contact> {
        self.dag
            .vertices()
            .into_iter()
            .cloned()
            .filter(|c| -> bool { self.dag.connections(c.clone()).unwrap().is_empty() })
            .collect()
    }

    pub fn short(&mut self, x: Contact, y: Contact) -> Result<BTreeSet<Contact>, Error> {
        self.dag.add_edge(x, y)
    }

    fn _resolve_state(&mut self, c: Contact) -> bool {
        let mut final_state: bool = c.gate.output();
        if let Some(contacts) = self.dag.connections(c.clone()) {
            if !contacts.is_empty() {
                let state: bool = c.gate.input();
                let mut assumed_state: bool = c.program.clone().into();
                let mut state_set: bool = false;
                for contact in contacts.clone() {
                    if self._resolve_state(contact) != assumed_state {
                        if !state_set {
                            assumed_state = !assumed_state;
                            state_set = true;
                        }
                    }
                }
                // If the determined state is not equal to the current state,
                // update the current state with the determined state.
                if state != assumed_state {
                    let mut updated_c: Contact = c.clone();
                    updated_c.gate.toggle();
                    self.update(c, updated_c.clone());
                    final_state = updated_c.gate.output();
                }
            }
        }
        // If there are no adjacent vertices, then this node is a leaf node;
        // the state is simply the output of the contact's XOR gate.
        final_state
    }
}

fn try_str_to_bool(s: String) -> Result<Vec<bool>, Error> {
    let mut pv_vec: Vec<bool> = Vec::new();
    for char in s.chars() {
        if char == '0'.into() {
            pv_vec.push(false);
        } else if char == '1'.into() {
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
            s.push_str("0");
        } else {
            s.push_str("1");
        }
    }
    s
}

impl Default for BTreeReducer {
    fn default() -> Self {
        Self::new()
    }
}

impl Input<Vec<bool>> for BTreeReducer {
    fn input(&self) -> Vec<bool> {
        self.get_input_contacts()
            .iter()
            .map(|c| -> bool { c.gate.input() })
            .collect()
    }
}

impl Output<bool> for BTreeReducer {
    fn output(&mut self) -> bool {
        self._resolve_state(self.root())
    }
}

impl Output<String> for BTreeReducer {
    fn output(&mut self) -> String {
        if self._resolve_state(self.root()) {
            String::from("1")
        } else {
            String::from("0")
        }
    }
}

impl Input<String> for BTreeReducer
where
    Self: Input<Vec<bool>>,
{
    fn input(&self) -> String {
        bool_to_str(self.input())
    }
}

impl TransitionInput<Vec<bool>> for BTreeReducer {
    type Error = Error;
    fn transition_input(&mut self, iv: Vec<bool>) -> Result<(), Self::Error> {
        let current_iv: Vec<bool> = self.input();
        if iv.len() != current_iv.len() {
            return Err(Error::EdgeExistsError);
        }
        for (vertex, state) in self.get_input_contacts().iter().zip(iv.clone()) {
            if vertex.gate.input() != state {
                let mut updated_vertex = vertex.clone();
                updated_vertex.gate.toggle();
                self.update(vertex.clone(), updated_vertex);
            }
        }
        Ok(())
    }
}

impl TransitionInput<String> for BTreeReducer
where
    Self: TransitionInput<Vec<bool>>,
{
    type Error = Error;
    fn transition_input(&mut self, is: String) -> Result<(), Self::Error> {
        let iv: Vec<bool> = try_str_to_bool(is)?;
        self.transition_input(iv)
    }
}

impl State<Vec<bool>> for BTreeReducer {
    fn state(&self) -> Vec<bool> {
        self.dag
            .vertices()
            .into_iter()
            .map(|c| -> bool { c.gate.configuration() })
            .collect()
    }
}

impl State<String> for BTreeReducer
where
    Self: State<Vec<bool>>,
{
    fn state(&self) -> String {
        bool_to_str(self.state())
    }
}

impl TransitionState<Vec<bool>> for BTreeReducer {
    type Error = Error;
    fn transition_state(&mut self, sv: Vec<bool>) -> Result<(), Self::Error> {
        let current_sv: Vec<bool> = self.state();
        if sv.len() != current_sv.len() {
            return Err(Error::EdgeExistsError);
        }
        for (vertex, state) in self.dag.clone().vertices().into_iter().zip(sv.clone()) {
            if vertex.gate.configuration() != state {
                let mut updated_vertex = vertex.clone();
                updated_vertex.gate.reconfigure();
                self.update(vertex.clone(), updated_vertex);
            }
        }
        Ok(())
    }
}

impl TransitionState<String> for BTreeReducer
where
    Self: TransitionState<Vec<bool>>,
{
    type Error = Error;
    fn transition_state(&mut self, ss: String) -> Result<(), Self::Error> {
        let sv: Vec<bool> = try_str_to_bool(ss)?;
        self.transition_state(sv)
    }
}

impl Reprogram<Vec<bool>> for BTreeReducer {
    type Error = Error;
    fn reprogram(&mut self, pv: Vec<bool>) -> Result<(), Self::Error> {
        let current_pv: Vec<bool> = self.state();
        if pv.len() != current_pv.len() {
            return Err(Error::EdgeExistsError);
        }
        for (vertex, state) in self.dag.clone().vertices().into_iter().zip(pv.clone()) {
            if vertex.program != state {
                let mut updated_vertex = vertex.clone();
                updated_vertex.reprogram(state)?;
                self.update(vertex.clone(), updated_vertex);
            }
        }
        Ok(())
    }
}

impl Reprogram<String> for BTreeReducer
where
    Self: Reprogram<Vec<bool>>,
{
    type Error = Error;
    fn reprogram(&mut self, ps: String) -> Result<(), Error> {
        let pv_vec: Vec<bool> = try_str_to_bool(ps)?;
        self.reprogram(pv_vec)
    }
}

#[cfg(test)]
mod unit_tests {
    use crate::reducer::api::{Input, Reprogram, State, TransitionInput, Output, TransitionState};
    use crate::reducer::{BTreeReducer, Contact};
    use crate::xor::api::{
        Configuration, Input as XorInput, Output as XorOutput, Reconfigure, Toggle,
    };
    use crate::xor::XOR;
    use alloc::string::String;
    use alloc::vec::Vec;
    use btree_dag::error::Error;

    #[test]
    fn new() {
        let reducer: BTreeReducer = BTreeReducer::new();
        assert_eq!(reducer, BTreeReducer::default())
    }

    #[test]
    fn input() {
        let reducer: BTreeReducer = BTreeReducer::new();
        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 1);
        assert!(!input[0])
    }

    #[test]
    fn state() {
        let reducer: BTreeReducer = BTreeReducer::new();
        let state: Vec<bool> = reducer.state();
        assert_eq!(state.len(), 1);
        assert!(!state[0])
    }

    #[test]
    fn output() {
        let mut reducer: BTreeReducer = BTreeReducer::new();
        let output: bool = reducer.output();
        assert!(!output)
    }

    #[test]
    fn root() {
        let reducer: BTreeReducer = BTreeReducer::new();
        assert_eq!(
            reducer.root(),
            Contact {
                id: 0,
                gate: XOR::new(),
                program: bool::default(),
            }
        );
    }

    #[test]
    fn update() {
        let mut reducer: BTreeReducer = BTreeReducer::new();
        let root = reducer.root();
        assert!(!root.gate.input());
        assert!(!root.gate.configuration());
        assert!(!root.gate.output());

        let mut new_root = reducer.root();
        new_root.gate.toggle();
        reducer.update(reducer.root(), new_root);

        assert!(reducer.root().gate.input());
        assert!(!reducer.root().gate.configuration());
        assert!(reducer.root().gate.output());

        let mut new_root = reducer.root();
        new_root.gate.toggle();
        reducer.update(reducer.root(), new_root);

        let mut new_root = reducer.root();
        new_root.gate.reconfigure();
        reducer.update(reducer.root(), new_root);

        assert!(!reducer.root().gate.input());
        assert!(reducer.root().gate.configuration());
        assert!(reducer.root().gate.output());

        let mut new_root = reducer.root();
        new_root.gate.reconfigure();
        reducer.update(reducer.root(), new_root);

        assert!(!reducer.root().gate.input());
        assert!(!reducer.root().gate.configuration());
        assert!(!reducer.root().gate.output());
    }

    #[test]
    fn add_gate() {
        let mut reducer: BTreeReducer = BTreeReducer::new();
        reducer.add_gate(reducer.root());

            let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 1);
        assert!(!input[0]);

        let state: Vec<bool> = reducer.state();
        assert_eq!(state.len(), 2);
        assert!(!state[0]);
        assert!(!state[1]);

        let output: bool = reducer.output();
        assert!(!output);

        let series = reducer.add_gate(reducer.root());

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(!input[0]);
        assert!(!input[1]);

        let state: Vec<bool> = reducer.state();
        assert_eq!(state.len(), 3);
        assert!(!state[0]);
        assert!(!state[1]);
        assert!(!state[2]);

        let output: bool = reducer.output();
        assert!(!output);

        reducer.add_gate(series);

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(!input[0]);
        assert!(!input[1]);

        let state: Vec<bool> = reducer.state();
        assert_eq!(state.len(), 4);
        assert!(!state[0]);
        assert!(!state[1]);
        assert!(!state[2]);
        assert!(!state[3]);

        let output: bool = reducer.output();
        assert!(!output);
    }

    #[test]
    fn transition_input() -> Result<(), Error> {
        let mut reducer: BTreeReducer = BTreeReducer::new();
        reducer.add_gate(reducer.root());

            let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 1);
        assert!(!input[0]);

        let mut iv: Vec<bool> = Vec::new();
        iv.push(true);
        iv.push(true);
        assert!(reducer.transition_input(iv).is_err());

        let mut iv: Vec<bool> = Vec::new();
        iv.push(true);
        reducer.transition_input(iv)?;

            let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 1);
        assert!(input[0]);

        let mut iv: Vec<bool> = Vec::new();
        iv.push(true);
        reducer.transition_input(iv)?;

            let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 1);
        assert!(input[0]);

        let mut iv: Vec<bool> = Vec::new();
        iv.push(false);
        reducer.transition_input(iv)?;

            let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 1);
        assert!(!input[0]);

        let mut reducer: BTreeReducer = BTreeReducer::new();
        reducer.add_gate(reducer.root());
        reducer.add_gate(reducer.root());

        let mut iv: Vec<bool> = Vec::new();
        iv.push(true);
        assert!(reducer.transition_input(iv).is_err());

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(!input[0]);
        assert!(!input[1]);

        let mut iv: Vec<bool> = Vec::new();
        iv.push(true);
        iv.push(true);
        reducer.transition_input(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(input[0]);
        assert!(input[1]);

        let mut iv: Vec<bool> = Vec::new();
        iv.push(true);
        iv.push(true);
        reducer.transition_input(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(input[0]);
        assert!(input[1]);

        let mut iv: Vec<bool> = Vec::new();
        iv.push(false);
        iv.push(true);
        reducer.transition_input(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(!input[0]);
        assert!(input[1]);

        let mut iv: Vec<bool> = Vec::new();
        iv.push(true);
        iv.push(false);
        reducer.transition_input(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(input[0]);
        assert!(!input[1]);

        let mut iv: Vec<bool> = Vec::new();
        iv.push(false);
        iv.push(false);
        reducer.transition_input(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(!input[0]);
        assert!(!input[1]);
        Ok(())
    }

    #[test]
    fn transition_state() -> Result<(), Error> {
        let mut reducer: BTreeReducer = BTreeReducer::new();
        reducer.add_gate(reducer.root());

        let state: Vec<bool> = reducer.state();
        assert_eq!(state.len(), 2);
        assert!(!state[0]);
        assert!(!state[1]);

        let mut sv: Vec<bool> = Vec::new();
        sv.push(true);
        sv.push(true);
        sv.push(true);
        assert!(reducer.transition_state(sv).is_err());

        let mut sv: Vec<bool> = Vec::new();
        sv.push(true);
        assert!(reducer.transition_state(sv).is_err());

        let mut sv: Vec<bool> = Vec::new();
        sv.push(true);
        sv.push(true);
        reducer.transition_state(sv)?;

            let state: Vec<bool> = reducer.state();
        assert_eq!(state.len(), 2);
        assert!(state[0]);
        assert!(state[1]);

        let mut sv: Vec<bool> = Vec::new();
        sv.push(false);
        sv.push(true);
        reducer.transition_state(sv)?;

            let state: Vec<bool> = reducer.state();
        assert_eq!(state.len(), 2);
        assert!(!state[0]);
        assert!(state[1]);

        let mut sv: Vec<bool> = Vec::new();
        sv.push(false);
        sv.push(false);
        reducer.transition_state(sv)?;

            let state: Vec<bool> = reducer.state();
        assert_eq!(state.len(), 2);
        assert!(!state[0]);
        assert!(!state[1]);

        let mut sv: Vec<bool> = Vec::new();
        sv.push(false);
        sv.push(false);
        reducer.transition_state(sv)?;

            let state: Vec<bool> = reducer.state();
        assert_eq!(state.len(), 2);
        assert!(!state[0]);
        assert!(!state[1]);

        let mut sv: Vec<bool> = Vec::new();
        sv.push(true);
        sv.push(false);
        reducer.transition_state(sv)?;

            let state: Vec<bool> = reducer.state();
        assert_eq!(state.len(), 2);
        assert!(state[0]);
        assert!(!state[1]);

        Ok(())
    }

    #[test]
    fn and_truth_table() -> Result<(), Error> {
        let mut reducer: BTreeReducer = BTreeReducer::new();
        let series = reducer.add_gate(reducer.root());
        reducer.add_gate(series.clone());
        reducer.add_gate(series);

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

        let state: Vec<bool> = reducer.state();
        assert_eq!(state.len(), 4);
        assert!(!state[0]);
        assert!(!state[1]);
        assert!(!state[2]);
        assert!(!state[3]);

        let output: bool = reducer.output();
        assert!(!output);

        let mut iv: Vec<bool> = Vec::new();
        iv.push(true);
        iv.push(false);
        reducer.transition_input(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(input[0]);
        assert!(!input[1]);

        let state: Vec<bool> = reducer.state();
        assert_eq!(state.len(), 4);
        assert!(!state[0]);
        assert!(!state[1]);
        assert!(!state[2]);
        assert!(!state[3]);

        let output: bool = reducer.output();
        assert!(!output);

        let mut iv: Vec<bool> = Vec::new();
        iv.push(true);
        iv.push(true);
        reducer.transition_input(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(input[0]);
        assert!(input[1]);

        let state: Vec<bool> = reducer.state();
        assert_eq!(state.len(), 4);
        assert!(!state[0]);
        assert!(!state[1]);
        assert!(!state[2]);
        assert!(!state[3]);

        let output: bool = reducer.output();
        assert!(output);

        let mut iv: Vec<bool> = Vec::new();
        iv.push(false);
        iv.push(true);
        reducer.transition_input(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(!input[0]);
        assert!(input[1]);

        let state: Vec<bool> = reducer.state();
        assert_eq!(state.len(), 4);
        assert!(!state[0]);
        assert!(!state[1]);
        assert!(!state[2]);
        assert!(!state[3]);

        let output: bool = reducer.output();
        assert!(!output);
        Ok(())
    }

    #[test]
    fn nand_truth_table() -> Result<(), Error> {
        let mut reducer: BTreeReducer = BTreeReducer::new();
        let series = reducer.add_gate(reducer.root());
        reducer.add_gate(series.clone());
        reducer.add_gate(series);

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

        let state: Vec<bool> = reducer.state();
        assert_eq!(state.len(), 4);
        assert!(!state[0]);
        assert!(!state[1]);
        assert!(!state[2]);
        assert!(!state[3]);

        let output: bool = reducer.output();
        assert!(!output);

        let mut sv: Vec<bool> = Vec::new();
        sv.push(true);
        sv.push(false);
        sv.push(false);
        sv.push(false);
        reducer.transition_state(sv)?;

        let mut iv: Vec<bool> = Vec::new();
        iv.push(true);
        iv.push(false);
        reducer.transition_input(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(input[0]);
        assert!(!input[1]);

        let state: Vec<bool> = reducer.state();
        assert_eq!(state.len(), 4);
        assert!(state[0]);
        assert!(!state[1]);
        assert!(!state[2]);
        assert!(!state[3]);

        let output: bool = reducer.output();
        assert!(output);

        let mut iv: Vec<bool> = Vec::new();
        iv.push(true);
        iv.push(true);
        reducer.transition_input(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(input[0]);
        assert!(input[1]);

        let state: Vec<bool> = reducer.state();
        assert_eq!(state.len(), 4);
        assert!(state[0]);
        assert!(!state[1]);
        assert!(!state[2]);
        assert!(!state[3]);

        let output: bool = reducer.output();
        assert!(!output);

        let mut iv: Vec<bool> = Vec::new();
        iv.push(false);
        iv.push(true);
        reducer.transition_input(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(!input[0]);
        assert!(input[1]);

        let state: Vec<bool> = reducer.state();
        assert_eq!(state.len(), 4);
        assert!(state[0]);
        assert!(!state[1]);
        assert!(!state[2]);
        assert!(!state[3]);

        let output: bool = reducer.output();
        assert!(output);
        Ok(())
    }

    #[test]
    fn or_truth_table() -> Result<(), Error> {
        let mut reducer: BTreeReducer = BTreeReducer::new();
        let parallel = reducer.add_gate(reducer.root());
        reducer.add_gate(parallel.clone());
        reducer.add_gate(parallel);

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(!input[0]);
        assert!(!input[1]);

        let state: Vec<bool> = reducer.state();
        assert_eq!(state.len(), 4);
        assert!(!state[0]);
        assert!(!state[1]);
        assert!(!state[2]);
        assert!(!state[3]);

        let output: bool = reducer.output();
        assert!(!output);

        let mut iv: Vec<bool> = Vec::new();
        iv.push(true);
        iv.push(false);
        reducer.transition_input(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(input[0]);
        assert!(!input[1]);

        let state: Vec<bool> = reducer.state();
        assert_eq!(state.len(), 4);
        assert!(!state[0]);
        assert!(!state[1]);
        assert!(!state[2]);
        assert!(!state[3]);

        let output: bool = reducer.output();
        assert!(output);

        let mut iv: Vec<bool> = Vec::new();
        iv.push(true);
        iv.push(true);
        reducer.transition_input(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(input[0]);
        assert!(input[1]);

        let state: Vec<bool> = reducer.state();
        assert_eq!(state.len(), 4);
        assert!(!state[0]);
        assert!(!state[1]);
        assert!(!state[2]);
        assert!(!state[3]);

        let output: bool = reducer.output();
        assert!(output);

        let mut iv: Vec<bool> = Vec::new();
        iv.push(false);
        iv.push(true);
        reducer.transition_input(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(!input[0]);
        assert!(input[1]);

        let state: Vec<bool> = reducer.state();
        assert_eq!(state.len(), 4);
        assert!(!state[0]);
        assert!(!state[1]);
        assert!(!state[2]);
        assert!(!state[3]);

        let output: bool = reducer.output();
        assert!(output);
        Ok(())
    }

    #[test]
    fn nor_truth_table() -> Result<(), Error> {
        let mut reducer: BTreeReducer = BTreeReducer::new();
        let parallel = reducer.add_gate(reducer.root());
        reducer.add_gate(parallel.clone());
        reducer.add_gate(parallel);

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(!input[0]);
        assert!(!input[1]);

        let state: Vec<bool> = reducer.state();
        assert_eq!(state.len(), 4);
        assert!(!state[0]);
        assert!(!state[1]);
        assert!(!state[2]);
        assert!(!state[3]);

        let output: bool = reducer.output();
        assert!(!output);

        let mut sv: Vec<bool> = Vec::new();
        sv.push(true);
        sv.push(false);
        sv.push(false);
        sv.push(false);
        reducer.transition_state(sv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(!input[0]);
        assert!(!input[1]);

        let state: Vec<bool> = reducer.state();
        assert_eq!(state.len(), 4);
        assert!(state[0]);
        assert!(!state[1]);
        assert!(!state[2]);
        assert!(!state[3]);

        let output: bool = reducer.output();
        assert!(output);

        let mut sv: Vec<bool> = Vec::new();
        sv.push(false);
        sv.push(true);
        sv.push(false);
        sv.push(false);
        reducer.transition_state(sv)?;

        let mut iv: Vec<bool> = Vec::new();
        iv.push(true);
        iv.push(false);
        reducer.transition_input(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(input[0]);
        assert!(!input[1]);

        let state: Vec<bool> = reducer.state();
        assert_eq!(state.len(), 4);
        assert!(!state[0]);
        assert!(state[1]);
        assert!(!state[2]);
        assert!(!state[3]);

        let output: bool = reducer.output();
        assert!(!output);

        let mut iv: Vec<bool> = Vec::new();
        iv.push(true);
        iv.push(true);
        reducer.transition_input(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(input[0]);
        assert!(input[1]);

        let state: Vec<bool> = reducer.state();
        assert_eq!(state.len(), 4);
        assert!(!state[0]);
        assert!(state[1]);
        assert!(!state[2]);
        assert!(!state[3]);

        let output: bool = reducer.output();
        assert!(!output);

        let mut iv: Vec<bool> = Vec::new();
        iv.push(false);
        iv.push(true);
        reducer.transition_input(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(!input[0]);
        assert!(input[1]);

        let state: Vec<bool> = reducer.state();
        assert_eq!(state.len(), 4);
        assert!(!state[0]);
        assert!(state[1]);
        assert!(!state[2]);
        assert!(!state[3]);

        let output: bool = reducer.output();
        assert!(!output);
        Ok(())
    }

    #[test]
    fn xor_truth_table() -> Result<(), Error> {
        let mut reducer: BTreeReducer = BTreeReducer::new();
        let series_0 = reducer.add_gate(reducer.root());
        let parallel_1 = reducer.add_gate(series_0.clone());
        let series_1 = reducer.add_gate(series_0.clone());
        let input_0 = reducer.add_gate(parallel_1.clone());
        let input_1 = reducer.add_gate(parallel_1.clone());
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

        let state: Vec<bool> = reducer.state();
        assert_eq!(state.len(), 6);
        assert!(!state[0]);
        assert!(!state[1]);
        assert!(!state[2]);
        assert!(!state[3]);
        assert!(!state[4]);
        assert!(!state[5]);

        let output: bool = reducer.output();
        assert!(!output);

        let mut sv: Vec<bool> = Vec::new();
        sv.push(false);
        sv.push(false);
        sv.push(false);
        sv.push(true);
        sv.push(false);
        sv.push(false);
        reducer.transition_state(sv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(!input[0]);
        assert!(!input[1]);

        let state: Vec<bool> = reducer.state();
        assert_eq!(state.len(), 6);
        assert!(!state[0]);
        assert!(!state[1]);
        assert!(!state[2]);
        assert!(state[3]);
        assert!(!state[4]);
        assert!(!state[5]);

        let output: bool = reducer.output();
        assert!(!output);

        let mut iv: Vec<bool> = Vec::new();
        iv.push(true);
        iv.push(false);
        reducer.transition_input(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(input[0]);
        assert!(!input[1]);

        let state: Vec<bool> = reducer.state();
        assert_eq!(state.len(), 6);
        assert!(!state[0]);
        assert!(!state[1]);
        assert!(!state[2]);
        assert!(state[3]);
        assert!(!state[4]);
        assert!(!state[5]);

        let output: bool = reducer.output();
        assert!(output);

        let mut iv: Vec<bool> = Vec::new();
        iv.push(false);
        iv.push(true);
        reducer.transition_input(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(!input[0]);
        assert!(input[1]);

        let state: Vec<bool> = reducer.state();
        assert_eq!(state.len(), 6);
        assert!(!state[0]);
        assert!(!state[1]);
        assert!(!state[2]);
        assert!(state[3]);
        assert!(!state[4]);
        assert!(!state[5]);

        let output: bool = reducer.output();
        assert!(output);

        let mut iv: Vec<bool> = Vec::new();
        iv.push(false);
        iv.push(false);
        reducer.transition_input(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(!input[0]);
        assert!(!input[1]);

        let state: Vec<bool> = reducer.state();
        assert_eq!(state.len(), 6);
        assert!(!state[0]);
        assert!(!state[1]);
        assert!(!state[2]);
        assert!(state[3]);
        assert!(!state[4]);
        assert!(!state[5]);

        let output: bool = reducer.output();
        assert!(!output);
        Ok(())
    }

    #[test]
    fn xnor_truth_table() -> Result<(), Error> {
        let mut reducer: BTreeReducer = BTreeReducer::new();
        let series_0 = reducer.add_gate(reducer.root());
        let parallel_1 = reducer.add_gate(series_0.clone());
        let series_1 = reducer.add_gate(series_0.clone());
        let input_0 = reducer.add_gate(parallel_1.clone());
        let input_1 = reducer.add_gate(parallel_1.clone());
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

        let state: Vec<bool> = reducer.state();
        assert_eq!(state.len(), 6);
        assert!(!state[0]);
        assert!(!state[1]);
        assert!(!state[2]);
        assert!(!state[3]);
        assert!(!state[4]);
        assert!(!state[5]);

        let output: bool = reducer.output();
        assert!(!output);

        let mut sv: Vec<bool> = Vec::new();
        sv.push(true);
        sv.push(false);
        sv.push(false);
        sv.push(true);
        sv.push(false);
        sv.push(false);
        reducer.transition_state(sv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(!input[0]);
        assert!(!input[1]);

        let state: Vec<bool> = reducer.state();
        assert_eq!(state.len(), 6);
        assert!(state[0]);
        assert!(!state[1]);
        assert!(!state[2]);
        assert!(state[3]);
        assert!(!state[4]);
        assert!(!state[5]);

        let output: bool = reducer.output();
        assert!(output);

        let mut iv: Vec<bool> = Vec::new();
        iv.push(true);
        iv.push(false);
        reducer.transition_input(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(input[0]);
        assert!(!input[1]);

        let state: Vec<bool> = reducer.state();
        assert_eq!(state.len(), 6);
        assert!(state[0]);
        assert!(!state[1]);
        assert!(!state[2]);
        assert!(state[3]);
        assert!(!state[4]);
        assert!(!state[5]);

        let output: bool = reducer.output();
        assert!(!output);

        let mut iv: Vec<bool> = Vec::new();
        iv.push(false);
        iv.push(true);
        reducer.transition_input(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(!input[0]);
        assert!(input[1]);

        let state: Vec<bool> = reducer.state();
        assert_eq!(state.len(), 6);
        assert!(state[0]);
        assert!(!state[1]);
        assert!(!state[2]);
        assert!(state[3]);
        assert!(!state[4]);
        assert!(!state[5]);

        let output: bool = reducer.output();
        assert!(!output);

        let mut iv: Vec<bool> = Vec::new();
        iv.push(false);
        iv.push(false);
        reducer.transition_input(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(!input[0]);
        assert!(!input[1]);

        let state: Vec<bool> = reducer.state();
        assert_eq!(state.len(), 6);
        assert!(state[0]);
        assert!(!state[1]);
        assert!(!state[2]);
        assert!(state[3]);
        assert!(!state[4]);
        assert!(!state[5]);

        let output: bool = reducer.output();
        assert!(output);
        Ok(())
    }

    #[test]
    fn and_truth_table_trans_nand_truth_table() -> Result<(), Error> {
        let mut reducer: BTreeReducer = BTreeReducer::new();
        let series = reducer.add_gate(reducer.root());
        reducer.add_gate(series.clone());
        reducer.add_gate(series);

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

        let state: Vec<bool> = reducer.state();
        assert_eq!(state.len(), 4);
        assert!(!state[0]);
        assert!(!state[1]);
        assert!(!state[2]);
        assert!(!state[3]);

        let output: bool = reducer.output();
        assert!(!output);

        let mut iv: Vec<bool> = Vec::new();
        iv.push(true);
        iv.push(false);
        reducer.transition_input(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(input[0]);
        assert!(!input[1]);

        let state: Vec<bool> = reducer.state();
        assert_eq!(state.len(), 4);
        assert!(!state[0]);
        assert!(!state[1]);
        assert!(!state[2]);
        assert!(!state[3]);

        let output: bool = reducer.output();
        assert!(!output);

        let mut iv: Vec<bool> = Vec::new();
        iv.push(true);
        iv.push(true);
        reducer.transition_input(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(input[0]);
        assert!(input[1]);

        let state: Vec<bool> = reducer.state();
        assert_eq!(state.len(), 4);
        assert!(!state[0]);
        assert!(!state[1]);
        assert!(!state[2]);
        assert!(!state[3]);

        let output: bool = reducer.output();
        assert!(output);

        let mut iv: Vec<bool> = Vec::new();
        iv.push(false);
        iv.push(true);
        reducer.transition_input(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(!input[0]);
        assert!(input[1]);

        let state: Vec<bool> = reducer.state();
        assert_eq!(state.len(), 4);
        assert!(!state[0]);
        assert!(!state[1]);
        assert!(!state[2]);
        assert!(!state[3]);

        let output: bool = reducer.output();
        assert!(!output);

        let mut iv: Vec<bool> = Vec::new();
        iv.push(false);
        iv.push(false);
        reducer.transition_input(iv)?;

        let mut sv: Vec<bool> = Vec::new();
        sv.push(true);
        sv.push(false);
        sv.push(false);
        sv.push(false);
        reducer.transition_state(sv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(!input[0]);
        assert!(!input[1]);

        let state: Vec<bool> = reducer.state();
        assert_eq!(state.len(), 4);
        assert!(state[0]);
        assert!(!state[1]);
        assert!(!state[2]);
        assert!(!state[3]);

        let output: bool = reducer.output();
        assert!(output);

        let mut iv: Vec<bool> = Vec::new();
        iv.push(true);
        iv.push(false);
        reducer.transition_input(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(input[0]);
        assert!(!input[1]);

        let state: Vec<bool> = reducer.state();
        assert_eq!(state.len(), 4);
        assert!(state[0]);
        assert!(!state[1]);
        assert!(!state[2]);
        assert!(!state[3]);

        let output: bool = reducer.output();
        assert!(output);

        let mut iv: Vec<bool> = Vec::new();
        iv.push(true);
        iv.push(true);
        reducer.transition_input(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(input[0]);
        assert!(input[1]);

        let state: Vec<bool> = reducer.state();
        assert_eq!(state.len(), 4);
        assert!(state[0]);
        assert!(!state[1]);
        assert!(!state[2]);
        assert!(!state[3]);

        let output: bool = reducer.output();
        assert!(!output);

        let mut iv: Vec<bool> = Vec::new();
        iv.push(false);
        iv.push(true);
        reducer.transition_input(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(!input[0]);
        assert!(input[1]);

        let state: Vec<bool> = reducer.state();
        assert_eq!(state.len(), 4);
        assert!(state[0]);
        assert!(!state[1]);
        assert!(!state[2]);
        assert!(!state[3]);

        let output: bool = reducer.output();
        assert!(output);

        Ok(())
    }

    #[test]
    fn xor_truth_table_trans_xnor_truth_table() -> Result<(), Error> {
        let mut reducer: BTreeReducer = BTreeReducer::new();
        let series_0 = reducer.add_gate(reducer.root());
        let parallel_1 = reducer.add_gate(series_0.clone());
        let series_1 = reducer.add_gate(series_0.clone());
        let input_0 = reducer.add_gate(parallel_1.clone());
        let input_1 = reducer.add_gate(parallel_1.clone());
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
        reducer.transition_state(sv)?;

        // 00 -> 0
        let mut iv: Vec<bool> = Vec::new();
        iv.push(false);
        iv.push(false);
        reducer.transition_input(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(!input[0]);
        assert!(!input[1]);

        let state: Vec<bool> = reducer.state();
        assert_eq!(state.len(), 6);
        assert!(!state[0]);
        assert!(!state[1]);
        assert!(!state[2]);
        assert!(state[3]);
        assert!(!state[4]);
        assert!(!state[5]);

        let output: bool = reducer.output();
        assert!(!output);

        // 10 -> 1
        let mut iv: Vec<bool> = Vec::new();
        iv.push(true);
        iv.push(false);
        reducer.transition_input(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(input[0]);
        assert!(!input[1]);

        let state: Vec<bool> = reducer.state();
        assert_eq!(state.len(), 6);
        assert!(!state[0]);
        assert!(!state[1]);
        assert!(!state[2]);
        assert!(state[3]);
        assert!(!state[4]);
        assert!(!state[5]);

        let output: bool = reducer.output();
        assert!(output);

        // 01 -> 1
        let mut iv: Vec<bool> = Vec::new();
        iv.push(false);
        iv.push(true);
        reducer.transition_input(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(!input[0]);
        assert!(input[1]);

        let state: Vec<bool> = reducer.state();
        assert_eq!(state.len(), 6);
        assert!(!state[0]);
        assert!(!state[1]);
        assert!(!state[2]);
        assert!(state[3]);
        assert!(!state[4]);
        assert!(!state[5]);

        let output: bool = reducer.output();
        assert!(output);

        // 11 -> 0
        let mut iv: Vec<bool> = Vec::new();
        iv.push(true);
        iv.push(true);
        reducer.transition_input(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(input[0]);
        assert!(input[1]);

        let state: Vec<bool> = reducer.state();
        assert_eq!(state.len(), 6);
        assert!(!state[0]);
        assert!(!state[1]);
        assert!(!state[2]);
        assert!(state[3]);
        assert!(!state[4]);
        assert!(!state[5]);

        let output: bool = reducer.output();
        assert!(!output);

        // XOR -> XNOR
        let mut sv: Vec<bool> = Vec::new();
        sv.push(true);
        sv.push(false);
        sv.push(false);
        sv.push(true);
        sv.push(false);
        sv.push(false);
        reducer.transition_state(sv)?;

        // 00 -> 1
        let mut iv: Vec<bool> = Vec::new();
        iv.push(false);
        iv.push(false);
        reducer.transition_input(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(!input[0]);
        assert!(!input[1]);

        let state: Vec<bool> = reducer.state();
        assert_eq!(state.len(), 6);
        assert!(state[0]);
        assert!(!state[1]);
        assert!(!state[2]);
        assert!(state[3]);
        assert!(!state[4]);
        assert!(!state[5]);

        let output: bool = reducer.output();
        assert!(output);

        // 10 -> 0
        let mut iv: Vec<bool> = Vec::new();
        iv.push(true);
        iv.push(false);
        reducer.transition_input(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(input[0]);
        assert!(!input[1]);

        let state: Vec<bool> = reducer.state();
        assert_eq!(state.len(), 6);
        assert!(state[0]);
        assert!(!state[1]);
        assert!(!state[2]);
        assert!(state[3]);
        assert!(!state[4]);
        assert!(!state[5]);

        let output: bool = reducer.output();
        assert!(!output);

        // 01 -> 0
        let mut iv: Vec<bool> = Vec::new();
        iv.push(false);
        iv.push(true);
        reducer.transition_input(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(!input[0]);
        assert!(input[1]);

        let state: Vec<bool> = reducer.state();
        assert_eq!(state.len(), 6);
        assert!(state[0]);
        assert!(!state[1]);
        assert!(!state[2]);
        assert!(state[3]);
        assert!(!state[4]);
        assert!(!state[5]);

        let output: bool = reducer.output();
        assert!(!output);

        // 11 -> 1
        let mut iv: Vec<bool> = Vec::new();
        iv.push(true);
        iv.push(true);
        reducer.transition_input(iv)?;

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 2);
        assert!(input[0]);
        assert!(input[1]);

        let state: Vec<bool> = reducer.state();
        assert_eq!(state.len(), 6);
        assert!(state[0]);
        assert!(!state[1]);
        assert!(!state[2]);
        assert!(state[3]);
        assert!(!state[4]);
        assert!(!state[5]);

        let output: bool = reducer.output();
        assert!(output);

        Ok(())
    }

    #[test]
    fn xor_truth_table_trans_xnor_truth_table_string() -> Result<(), Error> {
        let mut reducer: BTreeReducer = BTreeReducer::new();
        let series_0 = reducer.add_gate(reducer.root());
        let parallel_1 = reducer.add_gate(series_0.clone());
        let series_1 = reducer.add_gate(series_0.clone());
        let input_0 = reducer.add_gate(parallel_1.clone());
        let input_1 = reducer.add_gate(parallel_1.clone());
        reducer.short(series_1.clone(), input_0)?;
        reducer.short(series_1, input_1)?;

        let ps: String = String::from("010100");
        reducer.reprogram(ps)?;

        let ss: String = String::from("000100");
        reducer.transition_state(ss)?;

        // 00 -> 0
        let is: String = String::from("00");
        reducer.transition_input(is)?;

        let input: String = reducer.input();
        assert_eq!(input.as_str(), "00");

        let state: String = reducer.state();
        assert_eq!(state.as_str(), "000100");

        let output: String = reducer.output();
        assert_eq!(output, "0");

        // 10 -> 1
        let is: String = String::from("10");
        reducer.transition_input(is)?;

        let input: String = reducer.input();
        assert_eq!(input.as_str(), "10");

        let state: String = reducer.state();
        assert_eq!(state.as_str(), "000100");

        let output: String = reducer.output();
        assert_eq!(output, "1");

        // 01 -> 1
        let is: String = String::from("01");
        reducer.transition_input(is)?;

        let input: String = reducer.input();
        assert_eq!(input.as_str(), "01");

        let state: String = reducer.state();
        assert_eq!(state.as_str(), "000100");

        let output: String = reducer.output();
        assert_eq!(output, "1");

        // 11 -> 0
        let is: String = String::from("11");
        reducer.transition_input(is)?;

        let input: String = reducer.input();
        assert_eq!(input.as_str(), "11");

        let state: String = reducer.state();
        assert_eq!(state.as_str(), "000100");

        let output: String = reducer.output();
        assert_eq!(output, "0");

        // XOR -> XNOR
        let ss: String = String::from("100100");
        reducer.transition_state(ss)?;

        // 00 -> 1
        let is: String = String::from("00");
        reducer.transition_input(is)?;

        let input: String = reducer.input();
        assert_eq!(input.as_str(), "00");

        let state: String = reducer.state();
        assert_eq!(state.as_str(), "100100");

        let output: String = reducer.output();
        assert_eq!(output, "1");

        // 10 -> 0
        let is: String = String::from("10");
        reducer.transition_input(is)?;

        let input: String = reducer.input();
        assert_eq!(input.as_str(), "10");

        let state: String = reducer.state();
        assert_eq!(state.as_str(), "100100");

        let output: String = reducer.output();
        assert_eq!(output, "0");

        // 01 -> 0
        let is: String = String::from("01");
        reducer.transition_input(is)?;

        let input: String = reducer.input();
        assert_eq!(input.as_str(), "01");

        let state: String = reducer.state();
        assert_eq!(state.as_str(), "100100");

        let output: String = reducer.output();
        assert_eq!(output, "0");

        // 11 -> 1
        let is: String = String::from("11");
        reducer.transition_input(is)?;

        let input: String = reducer.input();
        assert_eq!(input.as_str(), "11");

        let state: String = reducer.state();
        assert_eq!(state.as_str(), "100100");

        let output: String = reducer.output();
        assert_eq!(output, "1");

        Ok(())
    }
}
