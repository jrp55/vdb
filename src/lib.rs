// didactic: notes marked as such are intended for curious newcomers to Rust 
//           in order to explain a concept that's generally seen for the first time in the file.
//           Caveat lector: I am very much a Rust newbie so my explanations may not be accurate. I
//                          try to link to relevant materials.
// In general enthusiastic readers should read https://doc.rust-lang.org/book/title-page.html - it's a 
// great grounding in the language.

mod vdb {
    use std::collections::HashMap;
    use indexmap::IndexMap;
    type EngineIndex = usize;

    // didactic: Enums are cool sum-types / tagged unions. A bit like variants in C++.
    // tagged unions: https://en.wikipedia.org/wiki/Tagged_union
    // rust enums: https://doc.rust-lang.org/book/ch06-00-enums.html

    pub enum DistributionMethod {
        LoadBalance,             // balance the load over all engines in the vdb mapping
        FailOver(EngineIndex),   // The EngineIndex value should be treated as the primary engine
    }

    pub enum VDBKind 
    {
        Combinator,                       // Combines all the data from each child engine
        Distributor(DistributionMethod),  // Chooses a single engine to use, based on DistributionMethod
    }

    // didactic: structs are like named tuples
    // rust structs: https://doc.rust-lang.org/book/ch05-00-structs.html

    pub struct VDB
    {
        name: String,
        kind: VDBKind,
        mapping: IndexMap<EngineIndex, String>,
    }

    pub struct ResolvedEngine 
    {
        engine: EngineIndex,
        databasematch: String,
    }

    // didactic: traits define shared behaviour. They are _a bit_ like interfaces
    // traits in Rust: https://doc.rust-lang.org/book/ch10-02-traits.html

    pub trait DatabaseMapping {
        // didactic: function signatures look like `fn fn_name(parameters...) -> return_type
        // functions in Rust: https://doc.rust-lang.org/book/ch03-03-how-functions-work.html

        // didactic: Result type is an enum commonly used for error handling
        // Result type: https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html

        fn select_database(&self) -> Result<Vec<ResolvedEngine>, String>;
        fn set_primary_engine(&mut self, primary_engine: EngineIndex) -> Result<(), String>;
    }

    // didactic: impl (implementation) blocks define methods on structs
    // impl blocks / struct methods: https://doc.rust-lang.org/book/ch05-03-method-syntax.html

    impl VDB {
        fn select_single_failover(&self, primary_engine: EngineIndex) -> Result<Vec<ResolvedEngine>, String> {
            match self.mapping.get(&primary_engine) {
                Some(value) => Ok(vec![ResolvedEngine{engine: primary_engine, databasematch: value.to_string()}]),
                None => Err(format!("PrimaryEngine {} is not a configured engine of VDB {}", &primary_engine, self.name)),
            }
        }

        fn select_single_loadbalance(&self) -> Result<Vec<ResolvedEngine>, String> {
            match self.mapping.get_index(rand::random::<usize>() % self.mapping.len()) {
                Some(selected) => Ok(vec![ResolvedEngine{engine: *selected.0, databasematch: selected.1.to_string()}]),
                None => Err(format!("LoadBalance somehow didn't select an engine for VDB {}!", self.name)),
            }
        }
    }

    // didactic: `impl <trait> for <type>` implements a trait ("interface"-ish) for the type
    // traits in Rust: https://doc.rust-lang.org/book/ch10-02-traits.html
    impl DatabaseMapping for VDB {
        // The smelliness of this implementation also points towards the smelliness of the interface
        fn set_primary_engine(&mut self, primary_engine: EngineIndex) -> Result<(), String> {
            match &mut self.kind {
                VDBKind::Combinator => Err("Tried to set a primary engine on a combinator".to_string()),
                VDBKind::Distributor(dist_method) => {
                    match dist_method {
                        DistributionMethod::LoadBalance => Err("Tried to set a primary engine on a load balancing distributor".to_string()),
                        DistributionMethod::FailOver(cur_primary) => { 
                            if primary_engine > self.mapping.len() {
                                Err("Primary engine out of range".to_string())
                            }
                            else {
                                *cur_primary = primary_engine; 
                                Ok(()) 
                            }
                        }
                    }
                }
            }
        }

        fn select_database(&self) -> Result<Vec<ResolvedEngine>, String> {
            match &self.kind {
                VDBKind::Combinator => Ok(self.mapping.iter().map(|x| ResolvedEngine{engine: *x.0, databasematch: x.1.to_string()}).collect()),
                VDBKind::Distributor(dist_method) => {
                    match dist_method {
                        DistributionMethod::LoadBalance => self.select_single_loadbalance(),
                        DistributionMethod::FailOver(primary_engine) => self.select_single_failover(*primary_engine),
                    }
                }
            }
        }

    }

    pub struct VDBCollection
    {
        vdbs: HashMap<String, VDB>,
    }

    pub trait VDBResolver {
        fn resolve(&self, databasematch: String) -> Result<Vec<ResolvedEngine>, String>;
    }

    impl VDBResolver for VDBCollection {
        fn resolve(&self, databasematch: String) -> Result<Vec<ResolvedEngine>, String> {
            let missing_vdbs: Vec<&str> = databasematch.split(',').filter(|&x| !self.vdbs.contains_key(x)).collect();
        
            if missing_vdbs.len() > 0 {
                return Err(format!("VDB collection does not have entries for databases: {}", missing_vdbs.iter().map(|x| format!("\"{}\"", x)).collect::<Vec<String>>().join(" ")));
            }

            let mut blah : IndexMap<EngineIndex, Vec<String>> = IndexMap::new();
            for vdb_name in databasematch.split(',') {
                match self.vdbs.get(vdb_name).ok_or(format!("VDB collection does not have entry for database name {}", vdb_name)).unwrap().select_database() {
                    Ok(result) => {
                        for resolved_engine in result.iter() {
                            match blah.get_mut(&resolved_engine.engine) {
                                Some(entry) => entry.push(resolved_engine.databasematch.to_string()),
                                None => match blah.insert(resolved_engine.engine, vec![resolved_engine.databasematch.to_string()]) {
                                    Some(_) => panic!("Scary"),
                                    None => ()
                                },
                            }
                        }
                    },
                    Err(e) => panic!(e),
                }
            }

            Ok(blah.iter().map(|x| ResolvedEngine{engine: *x.0, databasematch: x.1.join(",")}).collect())
        }
    }

    // didactic: testing can be done "inline"
    // testing in rust: https://doc.rust-lang.org/book/ch11-00-testing.html
    #[cfg(test)]
    mod tests {

        fn setup_test_vdb(kind: super::VDBKind) -> super::VDB {
            let mut mapping = indexmap::IndexMap::new();
            mapping.insert(0, "foo".to_string());
            mapping.insert(1, "bar".to_string());

            super::VDB{ name: "testVDB".to_string(), kind: kind, mapping: mapping }
        }
        use super::*;
        #[test]
        fn basic_combinator() -> Result<(), String> {
            let vdb = setup_test_vdb(VDBKind::Combinator);
            let result = vdb.select_database();
            match result {
                Ok(result_mapping) => {
                    assert_eq!(result_mapping.len(), 2);
                    assert_eq!(result_mapping[0].engine, 0);
                    assert_eq!(result_mapping[0].databasematch, "foo");
                    assert_eq!(result_mapping[1].engine, 1);
                    assert_eq!(result_mapping[1].databasematch, "bar");
                    Ok(())
                },
                Err(e) => Err(e),
            }
        }

        #[test]
        fn basic_distributor_failover() -> Result<(), String> {
            let vdb = setup_test_vdb(VDBKind::Distributor(DistributionMethod::FailOver(0)));
            let result = vdb.select_database();
            match result {
                Ok(result_mapping) => {
                    assert_eq!(result_mapping.len(), 1);
                    assert_eq!(result_mapping[0].engine, 0);
                    assert_eq!(result_mapping[0].databasematch, "foo");
                    Ok(())
                },
                Err(e) => Err(e)
            }
        }

        #[test]
        fn basic_distributor_failover_set_primary() -> Result<(), String> {
            let mut vdb = setup_test_vdb(VDBKind::Distributor(DistributionMethod::FailOver(0)));
            vdb.set_primary_engine(1)?;
            let result = vdb.select_database();
            match result {
                Ok(result_mapping) => {
                    assert_eq!(result_mapping.len(), 1);
                    assert_eq!(result_mapping[0].engine, 1);
                    assert_eq!(result_mapping[0].databasematch, "bar");
                    Ok(())
                }
                Err(e) => Err(e)
            }
        }

        //// This is basically just to see if the reqwest library works...
        //// Will likely be how engine status is obtained in the future...
        //#[test]
        //fn reqwest_test() -> Result<(), Box<dyn std::error::Error>> {
            //let resp = reqwest::blocking::get("https://httpbin.org/ip")?
                //.json::<HashMap<String, String>>()?;
            //println!("{:#?}", resp);
            //Ok(())
        //}
    }
}

