use std::fs::File;
use std::path::Path;
use std::str::FromStr;

use rust_decimal::Decimal;

#[derive(Serialize, Deserialize)]
struct FuzzInput {
    left: String,
    right: String,
    operator: Op,
    result: String,
}

impl FuzzInput {
    fn new(a: &String, b: &String, op: Op) -> FuzzInput {
        // Use the decimal crate that wraps the C library to calculate
        // the result. This has higher precision so we then round it naively.
        FuzzInput {
            left: a.to_string(),
            right: b.to_string(),
            operator: op,
            result: FuzzInput::calculate(a, b, op),
        }
    }

    fn calculate(a: &String, b: &String, op: Op) -> String {
        use decimal::d128;

        let a = d128::from_str(a).unwrap();
        let b = d128::from_str(b).unwrap();
        let result = match op {
            Op::Add => a + b,
            Op::Sub => a - b,
            Op::Mul => a * b,
            Op::Div => a / b,
        }.to_string();
        if result.len() > 30 {
            // Absolute max of 29 + decimal point
            let (r1, _r2) = result.split_at(30);
            let r1 = r1.to_string();
            // TODO: Round
            r1
        } else {
            result
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
enum Op {
    Add,
    Sub,
    Mul,
    Div,
}

fn to_str<E: ::std::fmt::Display>(e: E) -> String {
    format!("{}", e)
}

pub fn generate(sample_size: u32, output: &Path) -> Result<(), String> {
    let pairs = generate_pairs(sample_size as usize);
    let input = pairs.iter()
                    .flat_map(|(a, b)|
                        vec![
                            FuzzInput::new(a, b, Op::Add),
                            FuzzInput::new(a, b, Op::Sub),
                            // TODO: These cause overflow but the C lib doesn't throw
                            //FuzzInput::new(a, b, Op::Mul),
                            FuzzInput::new(a, b, Op::Div),
                        ]
                    )
                    .collect::<Vec<_>>();
    let file = File::create(output).map_err(to_str)?;
    serde_json::to_writer_pretty(file, &input).map_err(to_str)?;
    Ok(())
}

fn generate_pairs(size: usize) -> Vec<(String, String)> {
    let mut v = Vec::new();
    for _ in 0..size {
        let i_int = rand::random::<u64>();
        let i_frac = rand::random::<u64>();
        let j_int = rand::random::<u64>();
        let j_frac = rand::random::<u64>();
        v.push((format!("{}.{}", i_int, i_frac), format!("{}.{}", j_int, j_frac)));
    }
    v
}

pub fn run(path: &Path) -> Result<(), String> {
    // Load in the json input
    let file = File::open(path).map_err(to_str)?;
    let input : Vec<FuzzInput> = serde_json::from_reader(file).map_err(to_str)?;
    let mut index = 0;
    for item in input {
        let a = Decimal::from_str(&item.left)
                    .expect(&format!("Failed to unwrap left for index: {}", index));
        let b = Decimal::from_str(&item.right)
                    .expect(&format!("Failed to unwrap right for index: {}", index));
        let result = match item.operator {
            Op::Add => a + b,
            Op::Sub => a - b,
            Op::Mul => a * b,
            Op::Div => a / b,
        }.to_string();
        // Finally, see if we match the result
        if result.ne(&item.result) {
            panic!("Result mismatch for index {}: {}", index, result);
        }
        index += 1;
    }
    Ok(())
}
