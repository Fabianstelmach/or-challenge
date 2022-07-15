# Simulated Annealing for Orteca
Main idea: generate some initial solution and then explore the neighborhood to find better solutions.
```
let problem;
let solution = initial_solution(problem);
loop {
    let neighbor_solution = generate_neighbor(problem, solution);
    if neighbor_solution.objective() < solution.objective() {
        solution = neighbor_solution
    }
}
return solution
```

## Swap chains
In order to generate neighboring solutions, we introduce the concept of a swap-chain. The solution to the orteca scheduling problem is mapping from a reservation to a cottage: `s: R -> C`. A swap-chain is a list of modifications that can be applied to chain `s` in order to obtain another chain `z`. Lets assume s: `(R_0, C_0), (R_1, C_1)`. The operations in a swap-chain can be either:
- Removal of an assignment. Example: Applying `R(R_1, C_1)` to `s` produces: `s_1: (R_0, C_0)`.
- Addition of an assignment. Example: Applying `A(R_1, C_0)` to `s_1` produces: `s_2: (R_0, C_0), (R_1, C_0)`
Therefore, applying the swap-chain `[R(R_1, C_1),A(R_1, C_0)]` to `s` produces `s_2: (R_0, C_0), (R_1, C_0)`
Notice that a sigular operation can produce mapping `s_i` that is not viable, for example due to the fact that not all reservations are mapped to a cottage. It is assumed that a swap-chain only produces viable solutions, even if the initial solution `s` is not viable.

The process of generating a neighboring solution can now be translated in terms of swap-chains:
```
let problem, solution
let swap_chain = generate_swap_chain(problem, solution)
let neighbor_solution = apply(swap_chain, solution)
```

## Generating swap chains
We have chosen to generate swap chains in a nondeterministic manner:
```
let problem, solution

let swap_chain = empty_swap_chain()
let reservation = random_reservation(problem)
swap_chain.add(RemoveMapping(reservation))

while apply(swap_chain, solution) has unmapped reservations {
    let reservation = random_reservation(problem)
    let cottage = random_viable_cottage(problem, reservation)
    if cottage is taken by overlapping_reservation {
        swap_chain.add(RemoveReservation(overlapping_reservation))
    }
    swap_chain.add(AddMapping(reservation, cottage))
}
```

## Generating initial solution
An initial solution can be computed using the method described above. This is a special case in which we begin with an empty mapping. Then, we generate a swap-chain that produces a valid initial solution `s`.

# Improvements
- Reducing dimensionality
- Introducing reservation priority
- Simulated annealing
- Partial evaluation
