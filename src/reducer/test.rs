#[cfg(test)]
mod unit_tests {
    use crate::reducer::api::{
        AddGate, Configuration, Input, Output, Reconfigure, Reinput, RemoveShort, Reprogram, Short,
        Transition,
    };
    use crate::reducer::{BTreeReducer, Gate};
    use crate::Error;
    use alloc::collections::BTreeSet;
    use alloc::string::String;
    use alloc::vec::Vec;

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
        let output: bool = reducer.output();
        assert!(!output);
        Ok(())
    }

    #[test]
    fn root() {
        let reducer: BTreeReducer<bool> = BTreeReducer::new();
        assert_eq!(
            reducer.root(),
            Gate {
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
        assert!(!root.output());

        let mut newroot = reducer.root();
        newroot.reinput(true)?;
        reducer.update(reducer.root(), newroot);

        assert!(reducer.root().input());
        assert!(!reducer.root().configuration());
        assert!(reducer.root().output());

        let mut newroot = reducer.root();
        newroot.reinput(false)?;
        reducer.update(reducer.root(), newroot);

        let mut newroot = reducer.root();
        newroot.reconfigure(true)?;
        reducer.update(reducer.root(), newroot);

        assert!(!reducer.root().input());
        assert!(reducer.root().configuration());
        assert!(reducer.root().output());

        let mut newroot = reducer.root();
        newroot.reconfigure(false)?;
        reducer.update(reducer.root(), newroot);

        assert!(!reducer.root().input());
        assert!(!reducer.root().configuration());
        assert!(!reducer.root().output());
        Ok(())
    }

    #[test]
    fn add_gate() -> Result<(), Error> {
        let mut reducer: BTreeReducer<bool> = BTreeReducer::new();
        reducer.add_gate(reducer.root());

        let input: Vec<bool> = reducer.input();
        assert_eq!(input.len(), 1);
        assert!(!input[0]);

        let configuration: Vec<bool> = reducer.configuration();
        assert_eq!(configuration.len(), 2);
        assert!(!configuration[0]);
        assert!(!configuration[1]);

        let output: bool = reducer.output();
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

        let output: bool = reducer.output();
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

        let output: bool = reducer.output();
        assert!(!output);
        Ok(())
    }

    #[test]
    fn reinput() -> Result<(), Error> {
        let mut reducer: BTreeReducer<bool> = BTreeReducer::new();
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

        let mut reducer: BTreeReducer<bool> = BTreeReducer::new();
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
        let mut reducer: BTreeReducer<bool> = BTreeReducer::new();
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
        let mut reducer: BTreeReducer<bool> = BTreeReducer::new();
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

        let output: bool = reducer.output();
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

        let output: bool = reducer.output();
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

        let output: bool = reducer.output();
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

        let output: bool = reducer.output();
        assert!(!output);
        Ok(())
    }

    #[test]
    fn nand_truth_table() -> Result<(), Error> {
        let mut reducer: BTreeReducer<bool> = BTreeReducer::new();
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

        let output: bool = reducer.output();
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

        let output: bool = reducer.output();
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

        let output: bool = reducer.output();
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

        let output: bool = reducer.output();
        assert!(output);
        Ok(())
    }

    #[test]
    fn or_truth_table() -> Result<(), Error> {
        let mut reducer: BTreeReducer<bool> = BTreeReducer::new();
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

        let output: bool = reducer.output();
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

        let output: bool = reducer.output();
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

        let output: bool = reducer.output();
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

        let output: bool = reducer.output();
        assert!(output);
        Ok(())
    }

    #[test]
    fn nor_truth_table() -> Result<(), Error> {
        let mut reducer: BTreeReducer<bool> = BTreeReducer::new();
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

        let output: bool = reducer.output();
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

        let output: bool = reducer.output();
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

        let output: bool = reducer.output();
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

        let output: bool = reducer.output();
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

        let output: bool = reducer.output();
        assert!(!output);
        Ok(())
    }

    #[test]
    fn xor_truth_table() -> Result<(), Error> {
        let mut reducer: BTreeReducer<bool> = BTreeReducer::new();
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

        let output: bool = reducer.output();
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

        let output: bool = reducer.output();
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

        let output: bool = reducer.output();
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

        let output: bool = reducer.output();
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

        let output: bool = reducer.output();
        assert!(!output);
        Ok(())
    }

    #[test]
    fn xnor_truth_table() -> Result<(), Error> {
        let mut reducer: BTreeReducer<bool> = BTreeReducer::new();
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

        let output: bool = reducer.output();
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

        let output: bool = reducer.output();
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

        let output: bool = reducer.output();
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

        let output: bool = reducer.output();
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

        let output: bool = reducer.output();
        assert!(output);
        Ok(())
    }

    #[test]
    fn and_truth_table_trans_nand_truth_table() -> Result<(), Error> {
        let mut reducer: BTreeReducer<bool> = BTreeReducer::new();
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

        let output: bool = reducer.output();
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

        let output: bool = reducer.output();
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

        let output: bool = reducer.output();
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

        let output: bool = reducer.output();
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

        let output: bool = reducer.output();
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

        let output: bool = reducer.output();
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

        let output: bool = reducer.output();
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

        let output: bool = reducer.output();
        assert!(output);

        Ok(())
    }

    #[test]
    fn xor_truth_table_trans_xnor_truth_table() -> Result<(), Error> {
        let mut reducer: BTreeReducer<bool> = BTreeReducer::new();
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

        let output: bool = reducer.output();
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

        let output: bool = reducer.output();
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

        let output: bool = reducer.output();
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

        let output: bool = reducer.output();
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

        let output: bool = reducer.output();
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

        let output: bool = reducer.output();
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

        let output: bool = reducer.output();
        assert!(output);

        Ok(())
    }

    #[test]
    fn xor_truth_table_trans_xnor_truth_table_string() -> Result<(), Error> {
        let mut reducer: BTreeReducer<bool> = BTreeReducer::new();
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

        let output: String = reducer.output();
        assert_eq!(output, "0");

        // 10 -> 1
        let is: String = String::from("10");
        reducer.reinput(is)?;

        let input: String = reducer.input();
        assert_eq!(input.as_str(), "10");

        let configuration: String = reducer.configuration();
        assert_eq!(configuration.as_str(), "000100");

        let output: String = reducer.output();
        assert_eq!(output, "1");

        // 01 -> 1
        let is: String = String::from("01");
        reducer.reinput(is)?;

        let input: String = reducer.input();
        assert_eq!(input.as_str(), "01");

        let configuration: String = reducer.configuration();
        assert_eq!(configuration.as_str(), "000100");

        let output: String = reducer.output();
        assert_eq!(output, "1");

        // 11 -> 0
        let is: String = String::from("11");
        reducer.reinput(is)?;

        let input: String = reducer.input();
        assert_eq!(input.as_str(), "11");

        let configuration: String = reducer.configuration();
        assert_eq!(configuration.as_str(), "000100");

        let output: String = reducer.output();
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

        let output: String = reducer.output();
        assert_eq!(output, "1");

        // 10 -> 0
        let is: String = String::from("10");
        reducer.reinput(is)?;

        let input: String = reducer.input();
        assert_eq!(input.as_str(), "10");

        let configuration: String = reducer.configuration();
        assert_eq!(configuration.as_str(), "100100");

        let output: String = reducer.output();
        assert_eq!(output, "0");

        // 01 -> 0
        let is: String = String::from("01");
        reducer.reinput(is)?;

        let input: String = reducer.input();
        assert_eq!(input.as_str(), "01");

        let configuration: String = reducer.configuration();
        assert_eq!(configuration.as_str(), "100100");

        let output: String = reducer.output();
        assert_eq!(output, "0");

        // 11 -> 1
        let is: String = String::from("11");
        reducer.reinput(is)?;

        let input: String = reducer.input();
        assert_eq!(input.as_str(), "11");

        let configuration: String = reducer.configuration();
        assert_eq!(configuration.as_str(), "100100");

        let output: String = reducer.output();
        assert_eq!(output, "1");

        Ok(())
    }

    #[test]
    fn vowels() -> Result<(), Error> {
        impl Transition<char> for Gate<char> {
            fn transition(&mut self) {
                self.program = 'y';
            }
        }

        impl Output<char> for Gate<char> {
            type Error = Error;
            fn output(&mut self) -> char {
                let mut vowels: BTreeSet<char> = BTreeSet::new();
                vowels.insert('a');
                vowels.insert('e');
                vowels.insert('i');
                vowels.insert('o');
                vowels.insert('u');
                vowels.insert('y');
                if vowels.contains(&self.input) {
                    'y'
                } else {
                    'n'
                }
            }
        }

        let mut reducer: BTreeReducer<char> = BTreeReducer::new();
        reducer.add_gate(reducer.root());
        reducer.add_gate(reducer.root());
        reducer.add_gate(reducer.root());

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

        assert_eq!(reducer.output(), 'y');

        let mut input: Vec<char> = Vec::new();
        input.push('c');
        input.push('a');
        input.push('t');
        reducer.reinput(input.clone())?;

        assert_eq!(reducer.output(), 'y');

        let mut input: Vec<char> = Vec::new();
        input.push('p');
        input.push('s');
        input.push('m');
        reducer.reinput(input.clone())?;

        assert_eq!(reducer.output(), 'n');

        let mut input: Vec<char> = Vec::new();
        input.push('i');
        input.push('b');
        input.push('m');
        reducer.reinput(input.clone())?;

        assert_eq!(reducer.output(), 'y');

        let mut input: Vec<char> = Vec::new();
        input.push('d');
        input.push('o');
        input.push('g');
        reducer.reinput(input.clone())?;

        assert_eq!(reducer.output(), 'y');

        let mut input: Vec<char> = Vec::new();
        input.push('t');
        input.push('l');
        input.push('s');
        reducer.reinput(input.clone())?;

        assert_eq!(reducer.output(), 'n');

        Ok(())
    }

    #[test]
    fn remove_short() -> Result<(), Error> {
        let mut reducer: BTreeReducer<bool> = BTreeReducer::new();
        let series_0 = reducer.add_gate(reducer.root());
        let parallel_1 = reducer.add_gate(series_0.clone());
        let series_1 = reducer.add_gate(series_0.clone());
        let input_0 = reducer.add_gate(parallel_1.clone());
        let input_1 = reducer.add_gate(parallel_1.clone());
        reducer.short(series_1.clone(), input_0)?;
        reducer.short(series_1.clone(), input_1.clone())?;
        reducer.remove_short(series_1, input_1)?;
        Ok(())
    }
}
