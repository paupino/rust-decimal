use core::{
    fmt::{self, Display},
    str::FromStr,
};
use rust_decimal::Decimal;
use serde_derive::{Deserialize, Serialize};
use std::{fs::File, path::Path};

#[derive(Serialize, Deserialize)]
struct FuzzInput {
    test: String,
    left: String,
    right: String,
    operator: Op,
    result: String,
}

impl FuzzInput {
    fn new(test_number: u32, a: &String, b: &String, op: Op) -> FuzzInput {
        // Use the decimal crate that wraps the C library to calculate
        // the result. This has higher precision so we then round it naively.
        FuzzInput {
            test: format!("{}.{}", test_number, op),
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
        }
        .to_string();
        if result.len() > 30 {
            // Absolute max of 29 + decimal point
            let (r1, r2) = result.split_at(30);
            //let negative = r1.chars().next().unwrap() == '-';
            let mut r1 = r1.to_string();
            // TODO: This rounding is naive and mostly wrong.
            let next_digit = r2.chars().next().unwrap().to_digit(10).unwrap();
            if next_digit >= 5 {
                let last_digit = r1.pop().unwrap().to_digit(10).unwrap() + 1;
                // If it is 10 then add one to the previous
                // Obviously, this could keep going but for now just do the next digit
                if last_digit > 9 {
                    let prev_digit = r1.pop().unwrap().to_digit(10).unwrap() + 1;
                    if prev_digit > 9 {
                        // We're in the loop scenario. Let's ignore rather than looping
                        // for now.
                        panic!("TODO");
                    }
                    r1.push_str(&format!("{}0", prev_digit));
                } else {
                    r1.push_str(&format!("{}", last_digit));
                }
            }
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

impl Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Op::Add => "add",
                Op::Sub => "sub",
                Op::Mul => "mul",
                Op::Div => "div",
            }
        )
    }
}

fn to_str<E: Display>(e: E) -> String {
    format!("{}", e)
}

pub fn generate(sample_size: u32, output: &Path) -> Result<(), String> {
    let pairs = generate_pairs(sample_size as usize);
    let mut test_number = 0;
    let input = pairs
        .iter()
        .flat_map(move |(a, b)| {
            test_number += 1;
            vec![
                FuzzInput::new(test_number, a, b, Op::Add),
                FuzzInput::new(test_number, a, b, Op::Sub),
                // TODO: These cause overflow but the C lib doesn't throw
                //FuzzInput::new(test_number, a, b, Op::Mul),
                FuzzInput::new(test_number, a, b, Op::Div),
            ]
        })
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
    let input: Vec<FuzzInput> = serde_json::from_reader(file).map_err(to_str)?;
    for item in input {
        let a = Decimal::from_str(&item.left).expect(&format!("Failed to unwrap left for test: {}", item.test));
        let b = Decimal::from_str(&item.right).expect(&format!("Failed to unwrap right for test: {}", item.test));
        let result = match item.operator {
            Op::Add => a + b,
            Op::Sub => a - b,
            Op::Mul => a * b,
            Op::Div => a / b,
        }
        .to_string();
        // Finally, see if we match the result
        if result.ne(&item.result) {
            panic!("Result mismatch for test {}: {}", item.test, result);
        }
    }
    Ok(())
}
