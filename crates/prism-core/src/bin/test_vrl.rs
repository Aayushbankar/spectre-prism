use std::collections::BTreeMap;
use vrl::{compiler::{compile, Context, TargetValue, TimeZone, state::RuntimeState}, stdlib::all, value::{Value, Secrets}};

fn main() {
    let script = r#"
        parts = split(string!(.message), ",")
        .srcip = parts[7]
        .srcport = parts[24]
    "#;
    let fns = all();
    let program = compile(script, &fns).unwrap().program;

    let mut map = BTreeMap::new();
    map.insert("message".into(), Value::from("1,2,3,4,5,6,7,192.168.1.5,9"));
    let mut target = TargetValue {
        value: Value::Object(map),
        metadata: Value::Object(BTreeMap::new()),
        secrets: Secrets::new(),
    };
    let mut state = RuntimeState::default();
    let tz = TimeZone::default();
    let mut ctx = Context::new(&mut target, &mut state, &tz);

    let res = program.resolve(&mut ctx);
    println!("Res: {:?}", res);
    println!("Target: {:?}", target.value);
}
