use vrl::compiler::{compile, Context, TargetValue, TimeZone, state::RuntimeState};
use vrl::stdlib::all;
use vrl::value::{Value, Secrets};
use std::collections::BTreeMap;

fn main() {
    let script = r#"
        parts = split(string!(.message), ",")
        .srcip = parts[7]
        .dstport = parts[25]
    "#;
    let fns = all();
    let mut program = compile(script, &fns).unwrap().program;
    
    let mut map = BTreeMap::new();
    map.insert("message".into(), Value::from("1,2,3,4,5,6,7,192.168.1.1,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,80,26"));
    
    let mut target = TargetValue {
        value: Value::Object(map),
        metadata: Value::Object(BTreeMap::new()),
        secrets: Secrets::new(),
    };
    
    let mut state = RuntimeState::default();
    let tz = TimeZone::default();
    let mut ctx = Context::new(&mut target, &mut state, &tz);
    
    match program.resolve(&mut ctx) {
        Ok(_) => println!("Success: {:?}", target.value),
        Err(e) => println!("Error during resolve: {:?}", e),
    }
}
