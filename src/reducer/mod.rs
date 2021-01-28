use crate::reducer::api::{Configuration, Input, Output, Program, Reconfigure, Reinput, Reprogram};
use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec::Vec;
use btree_dag::error::Error;
use btree_dag::{AddEdge, AddVertex, BTreeDAG, Connections, RemoveEdge, RemoveVertex, Vertices};

mod api;

#[derive(PartialEq, PartialOrd, Ord, Eq, Clone, Debug)]
pub struct Contact {
    id: usize,
    input: bool,
    configuration: bool,
    program: bool,
}

impl Input<bool> for Contact {
    fn input(&self) -> bool {
        self.input
    }
}

impl Configuration<bool> for Contact {
    fn configuration(&self) -> bool {
        self.configuration
    }
}

impl Program<bool> for Contact {
    fn program(&self) -> bool {
        self.program
    }
}

impl Output<bool> for Contact {
    type Error = Error;
    fn output(&mut self) -> Result<bool, Self::Error> {
        Ok(self.input != self.configuration)
    }
}

impl Reinput<bool> for Contact {
    type Error = Error;
    fn reinput(&mut self, i: bool) -> Result<(), Self::Error> {
        self.input = i;
        Ok(())
    }
}

impl Reconfigure<bool> for Contact {
    type Error = Error;
    fn reconfigure(&mut self, c: bool) -> Result<(), Self::Error> {
        self.configuration = c;
        Ok(())
    }
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
            input: bool::default(),
            configuration: bool::default(),
            program: bool::default(),
        };
        dag.add_vertex(contact_zero);
        BTreeReducer { dag }
    }

    fn add_gate(&mut self, c: Contact) -> Contact {
        let vertices: Vec<&Contact> = self.dag.vertices().into_iter().collect();
        let contact: Contact = Contact {
            id: vertices[vertices.len() - 1].id + 1,
            input: bool::default(),
            configuration: bool::default(),
            program: bool::default(),
        };
        self.dag.add_vertex(contact.clone());
        self.dag.add_edge(c, contact.clone()).unwrap();
        self._resolve_state(self.root()).unwrap();
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
            self._resolve_state(previous_parent).unwrap();
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

    pub fn remove_short(&mut self, x: Contact, y: Contact) -> Result<BTreeSet<Contact>, Error> {
        self.dag.remove_edge(x, y)
    }

    fn _resolve_state(&mut self, mut c: Contact) -> Result<bool, Error> {
        let mut final_state: bool = c.output()?;
        if let Some(contacts) = self.dag.connections(c.clone()) {
            if !contacts.is_empty() {
                let state: bool = c.input();
                let mut assumed_state: bool = c.program;
                let mut state_set: bool = false;
                for contact in contacts.clone() {
                    if self._resolve_state(contact)? != assumed_state {
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
                    updated_c.reinput(assumed_state)?;
                    self.update(c, updated_c.clone());
                    final_state = updated_c.output()?;
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
            .map(|c| -> bool { c.input() })
            .collect()
    }
}

impl Input<String> for BTreeReducer
where
    Self: Input<Vec<bool>>,
{
    fn input(&self) -> String {
        BTreeReducer::bool_to_str(self.input())
    }
}

impl Output<bool> for BTreeReducer {
    type Error = Error;
    fn output(&mut self) -> Result<bool, Self::Error> {
        self._resolve_state(self.root())
    }
}

impl Output<String> for BTreeReducer {
    type Error = Error;
    fn output(&mut self) -> Result<String, Self::Error> {
        if self._resolve_state(self.root())? {
            Ok(String::from("1"))
        } else {
            Ok(String::from("0"))
        }
    }
}

impl Reinput<Vec<bool>> for BTreeReducer {
    type Error = Error;
    fn reinput(&mut self, iv: Vec<bool>) -> Result<(), Self::Error> {
        let current_iv: Vec<bool> = self.input();
        if iv.len() != current_iv.len() {
            return Err(Error::EdgeExistsError);
        }
        for (vertex, state) in self.get_input_contacts().iter().zip(iv.clone()) {
            if vertex.input() != state {
                let mut updated_vertex = vertex.clone();
                updated_vertex.reinput(state)?;
                self.update(vertex.clone(), updated_vertex);
            }
        }
        Ok(())
    }
}

impl Reinput<String> for BTreeReducer
    where
        Self: Reinput<Vec<bool>>,
{
    type Error = Error;
    fn reinput(&mut self, ss: String) -> Result<(), Self::Error> {
        let sv: Vec<bool> = BTreeReducer::try_str_to_bool(ss)?;
        self.reinput(sv)
    }
}

impl Configuration<Vec<bool>> for BTreeReducer {
    fn configuration(&self) -> Vec<bool> {
        self.dag
            .vertices()
            .into_iter()
            .map(|c| -> bool { c.configuration() })
            .collect()
    }
}

impl Configuration<String> for BTreeReducer {
    fn configuration(&self) -> String {
        BTreeReducer::bool_to_str(self.configuration())
    }
}

impl Program<Vec<bool>> for BTreeReducer {
    fn program(&self) -> Vec<bool> {
        self.dag
            .vertices()
            .into_iter()
            .map(|c| -> bool { c.program() })
            .collect()
    }
}

impl Program<String> for BTreeReducer {
    fn program(&self) -> String {
        BTreeReducer::bool_to_str(self.program())
    }
}

impl Reconfigure<Vec<bool>> for BTreeReducer {
    type Error = Error;
    fn reconfigure(&mut self, cv: Vec<bool>) -> Result<(), Self::Error> {
        let current_cv: Vec<bool> = self.configuration();
        if cv.len() != current_cv.len() {
            return Err(Error::EdgeExistsError);
        }
        for (vertex, state) in self.dag.clone().vertices().into_iter().zip(cv.clone()) {
            if vertex.configuration() != state {
                let mut updated_vertex = vertex.clone();
                updated_vertex.reconfigure(state)?;
                self.update(vertex.clone(), updated_vertex);
            }
        }
        Ok(())
    }
}

impl Reconfigure<String> for BTreeReducer
    where
        Self: Reconfigure<Vec<bool>>,
{
    type Error = Error;
    fn reconfigure(&mut self, ss: String) -> Result<(), Self::Error> {
        let sv: Vec<bool> = BTreeReducer::try_str_to_bool(ss)?;
        self.reconfigure(sv)
    }
}

impl Reprogram<Vec<bool>> for BTreeReducer {
    type Error = Error;
    fn reprogram(&mut self, pv: Vec<bool>) -> Result<(), Self::Error> {
        let current_pv: Vec<bool> = self.program();
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
        let pv_vec: Vec<bool> = BTreeReducer::try_str_to_bool(ps)?;
        self.reprogram(pv_vec)
    }
}

#[cfg(test)]
mod unit_tests {
    use crate::reducer::api::{Configuration, Input, Output, Reconfigure, Reinput, Reprogram};
    use crate::reducer::{BTreeReducer, Contact};
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
    fn configuration() {
        let reducer: BTreeReducer = BTreeReducer::new();
        let configuration: Vec<bool> = reducer.configuration();
        assert_eq!(configuration.len(), 1);
        assert!(!configuration[0])
    }

    #[test]
    fn output() -> Result<(), Error> {
        let mut reducer: BTreeReducer = BTreeReducer::new();
        let output: bool = reducer.output()?;
        assert!(!output);
        Ok(())
    }

    #[test]
    fn root() {
        let reducer: BTreeReducer = BTreeReducer::new();
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
        let mut reducer: BTreeReducer = BTreeReducer::new();
        let mut root = reducer.root();
        assert!(!root.input());
        assert!(!root.configuration());
        assert!(!root.output()?);

        let mut new_root = reducer.root();
        new_root.reinput(true)?;
        reducer.update(reducer.root(), new_root);

        assert!(reducer.root().input());
        assert!(!reducer.root().configuration());
        assert!(reducer.root().output()?);

        let mut new_root = reducer.root();
        new_root.reinput(false)?;
        reducer.update(reducer.root(), new_root);

        let mut new_root = reducer.root();
        new_root.reconfigure(true)?;
        reducer.update(reducer.root(), new_root);

        assert!(!reducer.root().input());
        assert!(reducer.root().configuration());
        assert!(reducer.root().output()?);

        let mut new_root = reducer.root();
        new_root.reconfigure(false)?;
        reducer.update(reducer.root(), new_root);

        assert!(!reducer.root().input());
        assert!(!reducer.root().configuration());
        assert!(!reducer.root().output()?);
        Ok(())
    }

    #[test]
    fn add_gate() -> Result<(), Error> {
        let mut reducer: BTreeReducer = BTreeReducer::new();
        reducer.add_gate(reducer.root());

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 1);
        assert!(!input[0]);

        let configuration: Vec<bool> = reducer.configuration();
        assert_eq!(configuration.len(), 2);
        assert!(!configuration[0]);
        assert!(!configuration[1]);

        let output: bool = reducer.output()?;
        assert!(!output);

        let series = reducer.add_gate(reducer.root());

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

        reducer.add_gate(series);

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
        let mut reducer: BTreeReducer = BTreeReducer::new();
        reducer.add_gate(reducer.root());

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

        let mut reducer: BTreeReducer = BTreeReducer::new();
        reducer.add_gate(reducer.root());
        reducer.add_gate(reducer.root());

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
        let mut reducer: BTreeReducer = BTreeReducer::new();
        reducer.add_gate(reducer.root());

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
        let mut reducer: BTreeReducer = BTreeReducer::new();
        let parallel = reducer.add_gate(reducer.root());
        reducer.add_gate(parallel.clone());
        reducer.add_gate(parallel);

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
        let mut reducer: BTreeReducer = BTreeReducer::new();
        let parallel = reducer.add_gate(reducer.root());
        reducer.add_gate(parallel.clone());
        reducer.add_gate(parallel);

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
}
