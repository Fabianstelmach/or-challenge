use crate::{instance::Instance, problem::Problem, solution::Solution};

pub fn run_simulated_annealing(problem: Problem, id: usize) -> Instance {
    let solution = Solution::empty(&problem);
    let mut instance = Instance::new(problem, solution);
    println!("Generating initial solution on thread {id}");

    instance.repair();
    instance.accept_chain();
    println!("Initial solution generated on {id}");

    let now = std::time::Instant::now();
    let max_runtime_mins = 50f64;

    let n: i64 = 1500000000;
    for i in 0..n {
        instance.neighbor();

        let chain_cost = instance.chain.cost().objective();

        if chain_cost < 0 {
            instance.accept_chain();
        } else {
            let temp = 1f64 - ((i + 1) as f64 / n as f64);
            let temp = 0.2 * temp;
            let criterion = (-chain_cost as f64 / temp as f64).exp();
            if fastrand::f64() < criterion {
                instance.accept_chain();
            } else {
                instance.reject_chain();
            }
            if i % (n / 50) == 0 {
                println!("T{}: {}%", id, (i * 100) as f64 / n as f64);
                if now.elapsed().as_secs_f64() > max_runtime_mins * 60f64 {
                    println!("T{}: breaking, time limit reached", id);
                    break;
                }
            }
        }
    }

    println!("Thread {id} done");

    return instance;
}
