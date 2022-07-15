use std::io::Write;

use or_challenge::{read::read_problem_json, simulated_annealing::run_simulated_annealing};

fn main() {
    let problem = read_problem_json(
        "./cottages.json".to_string(),
        "./reservations.json".to_string(),
    );

    // run_simulated_annealing(problem);

    let cpus = num_cpus::get_physical();

    let now = std::time::Instant::now();
    let mut handles = Vec::new();
    for i in 0..cpus {
        let x = problem.clone();
        let handle = std::thread::spawn(move || run_simulated_annealing(x, i));
        handles.push(handle);
    }

    let mut instances = Vec::new();
    for handle in handles {
        let instance = handle.join().unwrap();
        let objective = or_challenge::cost::Cost::calculate_instance(&instance).objective();

        instances.push((instance, objective));
    }

    let best = instances.iter().min_by_key(|x| x.1).unwrap();
    let mapping = best.0.solution.mapping();
    let output_mapping = mapping
        .iter()
        .map(|x| problem.cottages.id[x.expect("Mapping is always there")]);

    let mut file = std::fs::File::create(format!("solution_{}", best.1))
        .expect("Could not create solution file");

    for om in output_mapping {
        write!(file, "{}\n", om).unwrap();
    }
    println!("Done! took {} seconds", now.elapsed().as_secs_f64());
}
