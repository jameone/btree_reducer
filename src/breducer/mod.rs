use crate::arrangement::Arrangement;
use crate::breducer::api::{Input, State, TransitionInput, TransitionState};
use crate::xor::api::{Configuration, Input as XorInput, Output, Reconfigure, Toggle};
use crate::xor::XOR;
use alloc::collections::BTreeSet;
use alloc::vec::Vec;
use btree_dag::error::Error;
use btree_dag::{AddEdge, AddVertex, BTreeDag, Connections, RemoveVertex, Vertices};

mod api;

#[derive(PartialEq, PartialOrd, Ord, Eq, Clone, Debug)]
pub struct Contact {
    id: usize,
    gate: XOR,
    wiring: Arrangement,
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct BTreeReducer {
    dag: BTreeDag<Contact>,
}

impl BTreeReducer {
    fn new() -> Self {
        let mut dag: BTreeDag<Contact> = BTreeDag::new();
        let contact_zero: Contact = Contact {
            id: 0,
            gate: XOR::new(),
            wiring: Arrangement::Parallel,
        };
        dag.add_vertex(contact_zero);
        BTreeReducer { dag }
    }

    fn add_gate(&mut self, c: Contact, a: Arrangement) -> Contact {
        let vertices: Vec<&Contact> = self.dag.vertices().into_iter().collect();
        let contact: Contact = Contact {
            id: vertices[vertices.len() - 1].id + 1,
            gate: XOR::new(),
            wiring: a,
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

    pub fn toggle(&mut self, c: Contact, b: bool) -> Contact {
        // First create a copy of the contact to update;
        let mut updated_c = c.clone();
        // then make the update,
        if !b {
            updated_c.gate.toggle();
        } else {
            updated_c.gate.reconfigure();
        }

        let previous_parents: BTreeSet<Contact> = self
            .dag
            .vertices()
            .into_iter()
            .cloned()
            .map(|v| -> (Contact, &BTreeSet<Contact>) {
                (v.clone(), self.dag.connections(v).unwrap())
            })
            .filter(|t| -> bool { t.1.contains(&c) })
            .map(|t| -> Contact { t.0 })
            .collect();

        // Get all the edges from the previous vertex;
        let result = self.dag.remove_vertex(c);
        self.dag.add_vertex(updated_c.clone());
        if let Some(previous_children) = result.unwrap() {
            // Add children back.
            for previous_child in previous_children {
                self.dag
                    .add_edge(updated_c.clone(), previous_child)
                    .unwrap();
            }
        }
        // Add parents back.
        for previous_parent in previous_parents {
            self.dag
                .add_edge(previous_parent.clone(), updated_c.clone())
                .unwrap();
            self._resolve_state(previous_parent);
        }
        updated_c
    }

    fn get_input_contacts(&self) -> Vec<Contact> {
        self.dag
            .vertices()
            .into_iter()
            .cloned()
            .filter(|c| -> bool { self.dag.connections(c.clone()).unwrap().is_empty() })
            .collect()
    }

    pub fn output(&mut self) -> bool {
        self._resolve_state(self.root())
    }

    pub fn short(&mut self, x: Contact, y: Contact) -> Result<Option<BTreeSet<Contact>>, Error> {
        self.dag.add_edge(x, y)
    }

    fn _resolve_state(&mut self, c: Contact) -> bool {
        let mut final_state: bool = c.gate.output();
        if let Some(contacts) = self.dag.connections(c.clone()) {
            if !contacts.is_empty() {
                let state: bool = c.gate.input();
                let mut assumed_state: bool = c.wiring.clone().into();
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
                    final_state = self.toggle(c, false).gate.output();
                }
            }
        }
        // If there are no adjacent vertices, then this node is a leaf node;
        // the state is simply the output of the contact's XOR gate.
        final_state
    }
}

impl Default for BTreeReducer {
    fn default() -> Self {
        Self::new()
    }
}

impl Input for BTreeReducer {
    fn input(&self) -> Vec<bool> {
        self.get_input_contacts()
            .iter()
            .map(|c| -> bool { c.gate.input() })
            .collect()
    }
}

impl TransitionInput for BTreeReducer {
    fn transition_input(&mut self, sv: Vec<bool>) -> Result<Vec<bool>, Error> {
        if sv.len() != self.input().len() {
            return Err(Error::EdgeExistsError);
        }
        for (vertex, state) in self.get_input_contacts().iter().zip(sv.clone()) {
            if vertex.gate.input() != state {
                self.toggle(vertex.clone(), false);
            }
        }
        Ok(sv)
    }
}

impl State for BTreeReducer {
    fn state(&self) -> Vec<bool> {
        self.dag
            .vertices()
            .into_iter()
            .map(|c| -> bool { c.gate.configuration() })
            .collect()
    }
}

impl TransitionState for BTreeReducer {
    fn transition_state(&mut self, sv: Vec<bool>) -> Result<Vec<bool>, Error> {
        if sv.len() != self.state().len() {
            return Err(Error::EdgeExistsError);
        }
        for (vertex, state) in self.dag.clone().vertices().into_iter().zip(sv.clone()) {
            if vertex.gate.configuration() != state {
                self.toggle(vertex.clone(), true);
            }
        }
        Ok(sv)
    }
}

#[cfg(test)]
mod unit_tests {
    use crate::arrangement::Arrangement;
    use crate::breducer::api::{Input, State, TransitionInput, TransitionState};
    use crate::breducer::{BTreeReducer, Contact};
    use crate::xor::api::{Configuration, Input as XorInput, Output};
    use crate::xor::XOR;
    use alloc::vec::Vec;
    use btree_dag::error::Error;

    #[test]
    fn new() {
        let breducer: BTreeReducer = BTreeReducer::new();
        assert_eq!(breducer, BTreeReducer::default())
    }

    #[test]
    fn input() {
        let breducer: BTreeReducer = BTreeReducer::new();
        assert_eq!(breducer.input().len(), 1);
        assert!(!breducer.input()[0])
    }

    #[test]
    fn state() {
        let breducer: BTreeReducer = BTreeReducer::new();
        assert_eq!(breducer.state().len(), 1);
        assert!(!breducer.state()[0])
    }

    #[test]
    fn output() {
        let mut breducer: BTreeReducer = BTreeReducer::new();
        assert!(!breducer.output())
    }

    #[test]
    fn root() {
        let mut breducer: BTreeReducer = BTreeReducer::new();
        let root = breducer.root();
        assert_eq!(
            root,
            Contact {
                id: 0,
                gate: XOR::new(),
                wiring: Arrangement::Parallel,
            }
        );

        breducer.add_gate(root, Arrangement::Series);
        let root = breducer.root();
        assert_eq!(
            root,
            Contact {
                id: 0,
                gate: XOR::new(),
                wiring: Arrangement::Parallel,
            }
        );
    }

    #[test]
    fn toggle() {
        let mut breducer: BTreeReducer = BTreeReducer::new();
        let root = breducer.root();
        assert!(!root.gate.input());
        assert!(!root.gate.configuration());
        assert!(!root.gate.output());

        let new_root = breducer.toggle(root.clone(), false);
        assert!(new_root.gate.input());
        assert!(!new_root.gate.configuration());
        assert!(new_root.gate.output());

        let toggled_root = breducer.toggle(new_root, false);
        assert_eq!(toggled_root, root.clone());

        let new_root = breducer.toggle(root, true);
        assert!(!new_root.gate.input());
        assert!(new_root.gate.configuration());
        assert!(new_root.gate.output());

        let root = breducer.toggle(new_root, true);
        assert!(!root.gate.input());
        assert!(!root.gate.configuration());
        assert!(!root.gate.output());

        let series = breducer.add_gate(breducer.root(), Arrangement::Series);
        assert!(!series.gate.input());
        assert!(!series.gate.configuration());
        assert!(!series.gate.output());
        let root = breducer.root();
        assert!(!root.gate.input());
        assert!(!root.gate.configuration());
        assert!(!root.gate.output());

        let new_series = breducer.toggle(series.clone(), false);
        assert!(new_series.gate.input());
        assert!(!new_series.gate.configuration());
        assert!(new_series.gate.output());
        let root = breducer.root();
        assert!(root.gate.input());
        assert!(!root.gate.configuration());
        assert!(root.gate.output());

        let toggled_series = breducer.toggle(new_series, false);
        assert_eq!(toggled_series, series.clone());
        assert!(!toggled_series.gate.input());
        assert!(!toggled_series.gate.configuration());
        assert!(!toggled_series.gate.output());
        let root = breducer.root();
        assert!(!root.gate.input());
        assert!(!root.gate.configuration());
        assert!(!root.gate.output());

        let new_series = breducer.toggle(series, true);
        assert!(!new_series.gate.input());
        assert!(new_series.gate.configuration());
        assert!(new_series.gate.output());
        let root = breducer.root();
        assert!(root.gate.input());
        assert!(!root.gate.configuration());
        assert!(root.gate.output());

        let mut breducer: BTreeReducer = BTreeReducer::new();
        let series_0 = breducer.add_gate(breducer.root(), Arrangement::Series);
        let series_1 = breducer.add_gate(breducer.root(), Arrangement::Series);

        assert_eq!(breducer.input().len(), 2);
        assert!(!breducer.input()[0]);
        assert!(!breducer.input()[1]);

        assert_eq!(breducer.state().len(), 3);
        assert!(!breducer.state()[0]);
        assert!(!breducer.state()[1]);
        assert!(!breducer.state()[2]);
        assert!(!breducer.output());

        let toggled_series_0 = breducer.toggle(series_0.clone(), false);

        assert_eq!(breducer.input().len(), 2);
        assert!(breducer.input()[0]);
        assert!(!breducer.input()[1]);

        assert_eq!(breducer.state().len(), 3);
        assert!(!breducer.state()[0]);
        assert!(!breducer.state()[1]);
        assert!(!breducer.state()[2]);
        assert!(breducer.output());

        let series_0_after_toggle = breducer.toggle(toggled_series_0, false);
        assert_eq!(series_0, series_0_after_toggle);
        assert_eq!(breducer.root().id, 0);

        assert_eq!(breducer.input().len(), 2);
        assert!(!breducer.input()[0]);
        assert!(!breducer.input()[1]);

        assert_eq!(breducer.state().len(), 3);
        assert!(!breducer.state()[0]);
        assert!(!breducer.state()[1]);
        assert!(!breducer.state()[2]);
        assert!(!breducer.output());

        let toggled_series_1 = breducer.toggle(series_1, false);

        assert_eq!(breducer.input().len(), 2);
        assert!(!breducer.input()[0]);
        assert!(breducer.input()[1]);

        assert_eq!(breducer.state().len(), 3);
        assert!(!breducer.state()[0]);
        assert!(!breducer.state()[1]);
        assert!(!breducer.state()[2]);
        assert!(breducer.output());

        breducer.toggle(toggled_series_1, false);

        assert_eq!(breducer.input().len(), 2);
        assert!(!breducer.input()[0]);
        assert!(!breducer.input()[1]);

        assert_eq!(breducer.state().len(), 3);
        assert!(!breducer.state()[0]);
        assert!(!breducer.state()[1]);
        assert!(!breducer.state()[2]);
        assert!(!breducer.output());
    }

    #[test]
    fn add_gate() {
        let mut breducer: BTreeReducer = BTreeReducer::new();
        breducer.add_gate(breducer.root(), Arrangement::Series);
        assert_eq!(breducer.input().len(), 1);
        assert!(!breducer.input()[0]);

        assert_eq!(breducer.state().len(), 2);
        assert!(!breducer.state()[0]);
        assert!(!breducer.state()[1]);

        assert!(!breducer.output());

        let series = breducer.add_gate(breducer.root(), Arrangement::Series);

        assert_eq!(breducer.input().len(), 2);
        assert!(!breducer.input()[0]);
        assert!(!breducer.input()[1]);

        assert_eq!(breducer.state().len(), 3);
        assert!(!breducer.state()[0]);
        assert!(!breducer.state()[1]);
        assert!(!breducer.state()[2]);

        assert!(!breducer.output());

        breducer.add_gate(series, Arrangement::Series);

        assert_eq!(breducer.input().len(), 2);
        assert!(!breducer.input()[0]);
        assert!(!breducer.input()[1]);

        assert_eq!(breducer.state().len(), 4);
        assert!(!breducer.state()[0]);
        assert!(!breducer.state()[1]);
        assert!(!breducer.state()[2]);
        assert!(!breducer.state()[3]);

        assert!(!breducer.output());
    }

    #[test]
    fn transition_input() -> Result<(), Error> {
        let mut breducer: BTreeReducer = BTreeReducer::new();
        breducer.add_gate(breducer.root(), Arrangement::Series);
        assert_eq!(breducer.input().len(), 1);
        assert!(!breducer.input()[0]);

        let mut sv: Vec<bool> = Vec::new();
        sv.push(true);
        sv.push(true);
        assert!(breducer.transition_input(sv).is_err());

        let mut sv: Vec<bool> = Vec::new();
        sv.push(true);
        breducer.transition_input(sv)?;
        assert_eq!(breducer.input().len(), 1);
        assert!(breducer.input()[0]);

        let mut sv: Vec<bool> = Vec::new();
        sv.push(true);
        breducer.transition_input(sv)?;
        assert_eq!(breducer.input().len(), 1);
        assert!(breducer.input()[0]);

        let mut sv: Vec<bool> = Vec::new();
        sv.push(false);
        breducer.transition_input(sv)?;
        assert_eq!(breducer.input().len(), 1);
        assert!(!breducer.input()[0]);

        let mut breducer: BTreeReducer = BTreeReducer::new();
        breducer.add_gate(breducer.root(), Arrangement::Series);
        breducer.add_gate(breducer.root(), Arrangement::Series);

        let mut sv: Vec<bool> = Vec::new();
        sv.push(true);
        assert!(breducer.transition_input(sv).is_err());

        assert_eq!(breducer.input().len(), 2);
        assert!(!breducer.input()[0]);
        assert!(!breducer.input()[1]);

        let mut sv: Vec<bool> = Vec::new();
        sv.push(true);
        sv.push(true);
        breducer.transition_input(sv)?;

        assert_eq!(breducer.input().len(), 2);
        assert!(breducer.input()[0]);
        assert!(breducer.input()[1]);

        let mut sv: Vec<bool> = Vec::new();
        sv.push(true);
        sv.push(true);
        breducer.transition_input(sv)?;

        assert_eq!(breducer.input().len(), 2);
        assert!(breducer.input()[0]);
        assert!(breducer.input()[1]);

        let mut sv: Vec<bool> = Vec::new();
        sv.push(false);
        sv.push(true);
        breducer.transition_input(sv)?;

        assert_eq!(breducer.input().len(), 2);
        assert!(!breducer.input()[0]);
        assert!(breducer.input()[1]);

        let mut sv: Vec<bool> = Vec::new();
        sv.push(true);
        sv.push(false);
        breducer.transition_input(sv)?;

        assert_eq!(breducer.input().len(), 2);
        assert!(breducer.input()[0]);
        assert!(!breducer.input()[1]);

        let mut sv: Vec<bool> = Vec::new();
        sv.push(false);
        sv.push(false);
        breducer.transition_input(sv)?;

        assert_eq!(breducer.input().len(), 2);
        assert!(!breducer.input()[0]);
        assert!(!breducer.input()[1]);
        Ok(())
    }

    #[test]
    fn transition_state() -> Result<(), Error> {
        let mut breducer: BTreeReducer = BTreeReducer::new();
        breducer.add_gate(breducer.root(), Arrangement::Series);
        assert_eq!(breducer.state().len(), 2);
        assert!(!breducer.state()[0]);
        assert!(!breducer.state()[1]);

        let mut sv: Vec<bool> = Vec::new();
        sv.push(true);
        sv.push(true);
        sv.push(true);
        assert!(breducer.transition_state(sv).is_err());

        let mut sv: Vec<bool> = Vec::new();
        sv.push(true);
        assert!(breducer.transition_state(sv).is_err());

        let mut sv: Vec<bool> = Vec::new();
        sv.push(true);
        sv.push(true);
        breducer.transition_state(sv)?;
        assert_eq!(breducer.state().len(), 2);
        assert!(breducer.state()[0]);
        assert!(breducer.state()[1]);

        let mut sv: Vec<bool> = Vec::new();
        sv.push(false);
        sv.push(true);
        breducer.transition_state(sv)?;
        assert_eq!(breducer.state().len(), 2);
        assert!(!breducer.state()[0]);
        assert!(breducer.state()[1]);

        let mut sv: Vec<bool> = Vec::new();
        sv.push(false);
        sv.push(false);
        breducer.transition_state(sv)?;
        assert_eq!(breducer.state().len(), 2);
        assert!(!breducer.state()[0]);
        assert!(!breducer.state()[1]);

        let mut sv: Vec<bool> = Vec::new();
        sv.push(false);
        sv.push(false);
        breducer.transition_state(sv)?;
        assert_eq!(breducer.state().len(), 2);
        assert!(!breducer.state()[0]);
        assert!(!breducer.state()[1]);

        let mut sv: Vec<bool> = Vec::new();
        sv.push(true);
        sv.push(false);
        breducer.transition_state(sv)?;
        assert_eq!(breducer.state().len(), 2);
        assert!(breducer.state()[0]);
        assert!(!breducer.state()[1]);

        Ok(())
    }

    #[test]
    fn and_truth_table() -> Result<(), Error> {
        let mut breducer: BTreeReducer = BTreeReducer::new();
        let series = breducer.add_gate(breducer.root(), Arrangement::Series);
        breducer.add_gate(series.clone(), Arrangement::Series);
        breducer.add_gate(series, Arrangement::Series);

        assert_eq!(breducer.input().len(), 2);
        assert!(!breducer.input()[0]);
        assert!(!breducer.input()[1]);

        assert_eq!(breducer.state().len(), 4);
        assert!(!breducer.state()[0]);
        assert!(!breducer.state()[1]);
        assert!(!breducer.state()[2]);
        assert!(!breducer.state()[3]);

        assert!(!breducer.output());

        let mut sv: Vec<bool> = Vec::new();
        sv.push(true);
        sv.push(false);
        breducer.transition_input(sv)?;

        assert_eq!(breducer.input().len(), 2);
        assert!(breducer.input()[0]);
        assert!(!breducer.input()[1]);

        assert_eq!(breducer.state().len(), 4);
        assert!(!breducer.state()[0]);
        assert!(!breducer.state()[1]);
        assert!(!breducer.state()[2]);
        assert!(!breducer.state()[3]);

        assert!(!breducer.output());

        let mut sv: Vec<bool> = Vec::new();
        sv.push(true);
        sv.push(true);
        breducer.transition_input(sv)?;

        assert_eq!(breducer.input().len(), 2);
        assert!(breducer.input()[0]);
        assert!(breducer.input()[1]);

        assert_eq!(breducer.state().len(), 4);
        assert!(!breducer.state()[0]);
        assert!(!breducer.state()[1]);
        assert!(!breducer.state()[2]);
        assert!(!breducer.state()[3]);

        assert!(breducer.output());

        let mut sv: Vec<bool> = Vec::new();
        sv.push(false);
        sv.push(true);
        breducer.transition_input(sv)?;

        assert_eq!(breducer.input().len(), 2);
        assert!(!breducer.input()[0]);
        assert!(breducer.input()[1]);

        assert_eq!(breducer.state().len(), 4);
        assert!(!breducer.state()[0]);
        assert!(!breducer.state()[1]);
        assert!(!breducer.state()[2]);
        assert!(!breducer.state()[3]);

        assert!(!breducer.output());
        Ok(())
    }

    #[test]
    fn nand_truth_table() -> Result<(), Error> {
        let mut breducer: BTreeReducer = BTreeReducer::new();
        let series = breducer.add_gate(breducer.root(), Arrangement::Series);
        breducer.add_gate(series.clone(), Arrangement::Series);
        breducer.add_gate(series, Arrangement::Series);

        assert_eq!(breducer.input().len(), 2);
        assert!(!breducer.input()[0]);
        assert!(!breducer.input()[1]);

        assert_eq!(breducer.state().len(), 4);
        assert!(!breducer.state()[0]);
        assert!(!breducer.state()[1]);
        assert!(!breducer.state()[2]);
        assert!(!breducer.state()[3]);

        assert!(!breducer.output());

        let mut sv: Vec<bool> = Vec::new();
        sv.push(true);
        sv.push(false);
        sv.push(false);
        sv.push(false);
        breducer.transition_state(sv)?;

        let mut sv: Vec<bool> = Vec::new();
        sv.push(true);
        sv.push(false);
        breducer.transition_input(sv)?;

        assert_eq!(breducer.input().len(), 2);
        assert!(breducer.input()[0]);
        assert!(!breducer.input()[1]);

        assert_eq!(breducer.state().len(), 4);
        assert!(breducer.state()[0]);
        assert!(!breducer.state()[1]);
        assert!(!breducer.state()[2]);
        assert!(!breducer.state()[3]);

        assert!(breducer.output());

        let mut sv: Vec<bool> = Vec::new();
        sv.push(true);
        sv.push(true);
        breducer.transition_input(sv)?;

        assert_eq!(breducer.input().len(), 2);
        assert!(breducer.input()[0]);
        assert!(breducer.input()[1]);

        assert_eq!(breducer.state().len(), 4);
        assert!(breducer.state()[0]);
        assert!(!breducer.state()[1]);
        assert!(!breducer.state()[2]);
        assert!(!breducer.state()[3]);

        assert!(!breducer.output());

        let mut sv: Vec<bool> = Vec::new();
        sv.push(false);
        sv.push(true);
        breducer.transition_input(sv)?;

        assert_eq!(breducer.input().len(), 2);
        assert!(!breducer.input()[0]);
        assert!(breducer.input()[1]);

        assert_eq!(breducer.state().len(), 4);
        assert!(breducer.state()[0]);
        assert!(!breducer.state()[1]);
        assert!(!breducer.state()[2]);
        assert!(!breducer.state()[3]);

        assert!(breducer.output());
        Ok(())
    }

    #[test]
    fn or_truth_table() -> Result<(), Error> {
        let mut breducer: BTreeReducer = BTreeReducer::new();
        let parallel = breducer.add_gate(breducer.root(), Arrangement::Parallel);
        breducer.add_gate(parallel.clone(), Arrangement::Series);
        breducer.add_gate(parallel, Arrangement::Series);

        assert_eq!(breducer.input().len(), 2);
        assert!(!breducer.input()[0]);
        assert!(!breducer.input()[1]);

        assert_eq!(breducer.state().len(), 4);
        assert!(!breducer.state()[0]);
        assert!(!breducer.state()[1]);
        assert!(!breducer.state()[2]);
        assert!(!breducer.state()[3]);

        assert!(!breducer.output());

        let mut sv: Vec<bool> = Vec::new();
        sv.push(true);
        sv.push(false);
        breducer.transition_input(sv)?;

        assert_eq!(breducer.input().len(), 2);
        assert!(breducer.input()[0]);
        assert!(!breducer.input()[1]);

        assert_eq!(breducer.state().len(), 4);
        assert!(!breducer.state()[0]);
        assert!(!breducer.state()[1]);
        assert!(!breducer.state()[2]);
        assert!(!breducer.state()[3]);

        assert!(breducer.output());

        let mut sv: Vec<bool> = Vec::new();
        sv.push(true);
        sv.push(true);
        breducer.transition_input(sv)?;

        assert_eq!(breducer.input().len(), 2);
        assert!(breducer.input()[0]);
        assert!(breducer.input()[1]);

        assert_eq!(breducer.state().len(), 4);
        assert!(!breducer.state()[0]);
        assert!(!breducer.state()[1]);
        assert!(!breducer.state()[2]);
        assert!(!breducer.state()[3]);

        assert!(breducer.output());

        let mut sv: Vec<bool> = Vec::new();
        sv.push(false);
        sv.push(true);
        breducer.transition_input(sv)?;

        assert_eq!(breducer.input().len(), 2);
        assert!(!breducer.input()[0]);
        assert!(breducer.input()[1]);

        assert_eq!(breducer.state().len(), 4);
        assert!(!breducer.state()[0]);
        assert!(!breducer.state()[1]);
        assert!(!breducer.state()[2]);
        assert!(!breducer.state()[3]);

        assert!(breducer.output());
        Ok(())
    }

    #[test]
    fn nor_truth_table() -> Result<(), Error> {
        let mut breducer: BTreeReducer = BTreeReducer::new();
        let parallel = breducer.add_gate(breducer.root(), Arrangement::Parallel);
        breducer.add_gate(parallel.clone(), Arrangement::Series);
        breducer.add_gate(parallel, Arrangement::Series);

        assert_eq!(breducer.input().len(), 2);
        assert!(!breducer.input()[0]);
        assert!(!breducer.input()[1]);

        assert_eq!(breducer.state().len(), 4);
        assert!(!breducer.state()[0]);
        assert!(!breducer.state()[1]);
        assert!(!breducer.state()[2]);
        assert!(!breducer.state()[3]);

        assert!(!breducer.output());

        let mut sv: Vec<bool> = Vec::new();
        sv.push(true);
        sv.push(false);
        sv.push(false);
        sv.push(false);
        breducer.transition_state(sv)?;

        assert_eq!(breducer.input().len(), 2);
        assert!(!breducer.input()[0]);
        assert!(!breducer.input()[1]);

        assert_eq!(breducer.state().len(), 4);
        assert!(breducer.state()[0]);
        assert!(!breducer.state()[1]);
        assert!(!breducer.state()[2]);
        assert!(!breducer.state()[3]);

        assert!(breducer.output());

        let mut sv: Vec<bool> = Vec::new();
        sv.push(false);
        sv.push(true);
        sv.push(false);
        sv.push(false);
        breducer.transition_state(sv)?;

        let mut sv: Vec<bool> = Vec::new();
        sv.push(true);
        sv.push(false);
        breducer.transition_input(sv)?;

        assert_eq!(breducer.input().len(), 2);
        assert!(breducer.input()[0]);
        assert!(!breducer.input()[1]);

        assert_eq!(breducer.state().len(), 4);
        assert!(!breducer.state()[0]);
        assert!(breducer.state()[1]);
        assert!(!breducer.state()[2]);
        assert!(!breducer.state()[3]);

        assert!(!breducer.output());

        let mut sv: Vec<bool> = Vec::new();
        sv.push(true);
        sv.push(true);
        breducer.transition_input(sv)?;

        assert_eq!(breducer.input().len(), 2);
        assert!(breducer.input()[0]);
        assert!(breducer.input()[1]);

        assert_eq!(breducer.state().len(), 4);
        assert!(!breducer.state()[0]);
        assert!(breducer.state()[1]);
        assert!(!breducer.state()[2]);
        assert!(!breducer.state()[3]);

        assert!(!breducer.output());

        let mut sv: Vec<bool> = Vec::new();
        sv.push(false);
        sv.push(true);
        breducer.transition_input(sv)?;

        assert_eq!(breducer.input().len(), 2);
        assert!(!breducer.input()[0]);
        assert!(breducer.input()[1]);

        assert_eq!(breducer.state().len(), 4);
        assert!(!breducer.state()[0]);
        assert!(breducer.state()[1]);
        assert!(!breducer.state()[2]);
        assert!(!breducer.state()[3]);

        assert!(!breducer.output());
        Ok(())
    }

    #[test]
    fn xor_truth_table() -> Result<(), Error> {
        let mut breducer: BTreeReducer = BTreeReducer::new();
        let series_0 = breducer.add_gate(breducer.root(), Arrangement::Series);
        let parallel_1 = breducer.add_gate(series_0.clone(), Arrangement::Parallel);
        let series_1 = breducer.add_gate(series_0.clone(), Arrangement::Series);
        let input_0 = breducer.add_gate(parallel_1.clone(), Arrangement::Parallel);
        let input_1 = breducer.add_gate(parallel_1.clone(), Arrangement::Parallel);
        breducer.short(series_1.clone(), input_0)?;
        breducer.short(series_1, input_1)?;

        assert_eq!(breducer.input().len(), 2);
        assert!(!breducer.input()[0]);
        assert!(!breducer.input()[1]);

        assert_eq!(breducer.state().len(), 6);
        assert!(!breducer.state()[0]);
        assert!(!breducer.state()[1]);
        assert!(!breducer.state()[2]);
        assert!(!breducer.state()[3]);
        assert!(!breducer.state()[4]);
        assert!(!breducer.state()[5]);

        assert!(!breducer.output());

        let mut sv: Vec<bool> = Vec::new();
        sv.push(false);
        sv.push(false);
        sv.push(false);
        sv.push(true);
        sv.push(false);
        sv.push(false);
        breducer.transition_state(sv)?;

        assert_eq!(breducer.input().len(), 2);
        assert!(!breducer.input()[0]);
        assert!(!breducer.input()[1]);

        assert_eq!(breducer.state().len(), 6);
        assert!(!breducer.state()[0]);
        assert!(!breducer.state()[1]);
        assert!(!breducer.state()[2]);
        assert!(breducer.state()[3]);
        assert!(!breducer.state()[4]);
        assert!(!breducer.state()[5]);

        assert!(!breducer.output());

        let mut sv: Vec<bool> = Vec::new();
        sv.push(true);
        sv.push(false);
        breducer.transition_input(sv)?;

        assert_eq!(breducer.input().len(), 2);
        assert!(breducer.input()[0]);
        assert!(!breducer.input()[1]);

        assert_eq!(breducer.state().len(), 6);
        assert!(!breducer.state()[0]);
        assert!(!breducer.state()[1]);
        assert!(!breducer.state()[2]);
        assert!(breducer.state()[3]);
        assert!(!breducer.state()[4]);
        assert!(!breducer.state()[5]);

        assert!(breducer.output());

        let mut sv: Vec<bool> = Vec::new();
        sv.push(false);
        sv.push(true);
        breducer.transition_input(sv)?;

        assert_eq!(breducer.input().len(), 2);
        assert!(!breducer.input()[0]);
        assert!(breducer.input()[1]);

        assert_eq!(breducer.state().len(), 6);
        assert!(!breducer.state()[0]);
        assert!(!breducer.state()[1]);
        assert!(!breducer.state()[2]);
        assert!(breducer.state()[3]);
        assert!(!breducer.state()[4]);
        assert!(!breducer.state()[5]);

        assert!(breducer.output());

        let mut sv: Vec<bool> = Vec::new();
        sv.push(false);
        sv.push(false);
        breducer.transition_input(sv)?;

        assert_eq!(breducer.input().len(), 2);
        assert!(!breducer.input()[0]);
        assert!(!breducer.input()[1]);

        assert_eq!(breducer.state().len(), 6);
        assert!(!breducer.state()[0]);
        assert!(!breducer.state()[1]);
        assert!(!breducer.state()[2]);
        assert!(breducer.state()[3]);
        assert!(!breducer.state()[4]);
        assert!(!breducer.state()[5]);

        assert!(!breducer.output());
        Ok(())
    }

    #[test]
    fn xnor_truth_table() -> Result<(), Error> {
        let mut breducer: BTreeReducer = BTreeReducer::new();
        let series_0 = breducer.add_gate(breducer.root(), Arrangement::Series);
        let parallel_1 = breducer.add_gate(series_0.clone(), Arrangement::Parallel);
        let series_1 = breducer.add_gate(series_0.clone(), Arrangement::Series);
        let input_0 = breducer.add_gate(parallel_1.clone(), Arrangement::Parallel);
        let input_1 = breducer.add_gate(parallel_1.clone(), Arrangement::Parallel);
        breducer.short(series_1.clone(), input_0)?;
        breducer.short(series_1, input_1)?;

        assert_eq!(breducer.input().len(), 2);
        assert!(!breducer.input()[0]);
        assert!(!breducer.input()[1]);

        assert_eq!(breducer.state().len(), 6);
        assert!(!breducer.state()[0]);
        assert!(!breducer.state()[1]);
        assert!(!breducer.state()[2]);
        assert!(!breducer.state()[3]);
        assert!(!breducer.state()[4]);
        assert!(!breducer.state()[5]);

        assert!(!breducer.output());

        let mut sv: Vec<bool> = Vec::new();
        sv.push(true);
        sv.push(false);
        sv.push(false);
        sv.push(true);
        sv.push(false);
        sv.push(false);
        breducer.transition_state(sv)?;

        assert_eq!(breducer.input().len(), 2);
        assert!(!breducer.input()[0]);
        assert!(!breducer.input()[1]);

        assert_eq!(breducer.state().len(), 6);
        assert!(breducer.state()[0]);
        assert!(!breducer.state()[1]);
        assert!(!breducer.state()[2]);
        assert!(breducer.state()[3]);
        assert!(!breducer.state()[4]);
        assert!(!breducer.state()[5]);

        assert!(breducer.output());

        let mut sv: Vec<bool> = Vec::new();
        sv.push(true);
        sv.push(false);
        breducer.transition_input(sv)?;

        assert_eq!(breducer.input().len(), 2);
        assert!(breducer.input()[0]);
        assert!(!breducer.input()[1]);

        assert_eq!(breducer.state().len(), 6);
        assert!(breducer.state()[0]);
        assert!(!breducer.state()[1]);
        assert!(!breducer.state()[2]);
        assert!(breducer.state()[3]);
        assert!(!breducer.state()[4]);
        assert!(!breducer.state()[5]);

        assert!(!breducer.output());

        let mut sv: Vec<bool> = Vec::new();
        sv.push(false);
        sv.push(true);
        breducer.transition_input(sv)?;

        assert_eq!(breducer.input().len(), 2);
        assert!(!breducer.input()[0]);
        assert!(breducer.input()[1]);

        assert_eq!(breducer.state().len(), 6);
        assert!(breducer.state()[0]);
        assert!(!breducer.state()[1]);
        assert!(!breducer.state()[2]);
        assert!(breducer.state()[3]);
        assert!(!breducer.state()[4]);
        assert!(!breducer.state()[5]);

        assert!(!breducer.output());

        let mut sv: Vec<bool> = Vec::new();
        sv.push(false);
        sv.push(false);
        breducer.transition_input(sv)?;

        assert_eq!(breducer.input().len(), 2);
        assert!(!breducer.input()[0]);
        assert!(!breducer.input()[1]);

        assert_eq!(breducer.state().len(), 6);
        assert!(breducer.state()[0]);
        assert!(!breducer.state()[1]);
        assert!(!breducer.state()[2]);
        assert!(breducer.state()[3]);
        assert!(!breducer.state()[4]);
        assert!(!breducer.state()[5]);

        assert!(breducer.output());
        Ok(())
    }
}


