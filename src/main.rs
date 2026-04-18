use macroquad::prelude::*;
use ::rand::{rng, RngExt, rngs::ThreadRng};
use std::time::Instant;
use std::env;

const BORDER: f32 = 50.0;
const DEBUG_OUTPUT: bool = false;

enum Alg {
    SA,
    CLASSICAL,
    TWO_OPT,
}

struct State {
    cities: Vec<[f32; 2]>,
    tour: Vec<usize>,
    n: usize,
    temp: f64,
    dist: f32,
    rng: ThreadRng,
    running: bool,
    alg: Alg,
}

impl State {
    fn new(n: usize) -> Self {
        Self {
            cities: Vec::new(),
            tour: Vec::new(),
            n,
            temp: 1000.0,
            dist: 0.0,
            rng: rng(),
            running: false,
            alg: Alg::SA,
        }
    }

    fn randomize(&mut self) {
        self.cities.clear();
        self.tour.clear();

        let w = screen_width();
        let h = screen_height();

        let mut available_cities = Vec::new();

        // Initializing the cities
        for i in 0..self.n {
            let city: [f32; 2] = [
                self.rng.random_range(BORDER..(w - BORDER)),
                self.rng.random_range(BORDER..(h - BORDER)),
            ];
            self.cities.push(city);
            available_cities.push(i);
        }

        // Initializing the tour
        while !available_cities.is_empty() {
            let tour_index = self.rng.random_range(0..available_cities.len());
            let city_index = available_cities.swap_remove(tour_index);
            self.tour.push(city_index);
            if available_cities.is_empty() {break;}
        }

        // Initialize helper fields for simulated annealing
        self.temp = 1000.0;
        self.dist = self.calculate_total_dist();
    }

    fn randomize_tour(&mut self) {
        let mut available_cities = Vec::new();

        self.tour.clear();

        // Initializing the cities
        for i in 0..self.n {
            available_cities.push(i);
        }

        while !available_cities.is_empty() {
            let tour_index = self.rng.random_range(0..available_cities.len());
            let city_index = available_cities.swap_remove(tour_index);
            self.tour.push(city_index);
            if available_cities.is_empty() {break;}
        }

        self.temp = 1000.0;
        self.dist = self.calculate_total_dist();
        
    }

    // simulated annealing
    fn update_sa(&mut self) -> bool {
        if self.temp < 0.00001 { return false; }
        
        // randomly choosing 2 city indexes
        let i = self.rng.random_range(0..self.n);
        let j = self.rng.random_range(0..self.n);
        
        // if 2 same cities are chosen, dont do anything
        if i == j { return true; }
        let (smaller, bigger) = if i > j {(j, i)} else {(i, j)};
        let current_dist = self.calculate_total_dist();

        // 2-opt. reversing a small portion of the tour to remove tangles
        self.tour[smaller..=bigger].reverse();

        let next_dist = self.calculate_total_dist();

        // checking if we should accept the new tour
        let mut accept = false;
        if next_dist < current_dist {
            accept = true;
        } else {
            let delta = (next_dist - current_dist) as f64;
            let acceptance_prob = (-delta / self.temp).exp();
            if self.rng.random_range(0.0..=1.0) < acceptance_prob {
                accept = true;
            }
        }

        if accept {
            self.dist = next_dist;
        } else {
            // Undo the change and keep the old distance
            self.tour[smaller..=bigger].reverse();
            self.dist = current_dist;
        }

        // cooling down
        self.temp *= 0.9998;
        true
    }

    fn update_two_opt(&mut self) -> bool {
        let mut best_dist = self.calculate_total_dist();
        let mut flip_happened = false;
        for i in 0..(self.n-1) {
            for j in (i+1)..self.n {
                // 2-opt. reversing a small portion of the tour to remove tangles
                self.tour[i..=j].reverse();

                let next_dist = self.calculate_total_dist();

                // checking if we should accept the new tour
                if next_dist > best_dist {
                    self.tour[i..=j].reverse();
                }
                else {
                    best_dist = next_dist;
                    flip_happened = true;
                }
            }
        }
        if self.dist == best_dist {return false;}
        self.dist = best_dist;
        flip_happened
    }

    fn display(&mut self) {
        // Displaying the tour as lines
        for i in 0..self.tour.len() {
            let c1 = self.cities[self.tour[i]];
            let c2 = self.cities[self.tour[(i + 1)%self.tour.len()]];
            draw_line(c1[0], c1[1], c2[0], c2[1], 1.0, BLACK);
        }

        // Displaying the cities as circles
        for i in 0..self.cities.len() {
            let city = self.cities[i];
            draw_circle(city[0], city[1], 10.0, GREEN);
        }

        let running_text = if self.running {&"State: running"} else {&"State: Done"};
        let running_color = if self.running {RED} else {DARKGREEN};
        let running_dims = measure_text(running_text, None, 30, 1.0);

        draw_text(&format!("Temp: {:.5}", self.temp), 20.0, 20.0, 30.0, BLACK);
        draw_text(&format!("Distance: {:.2}", self.dist), 20.0, 40.0, 30.0, BLACK);
        draw_text(running_text, screen_width() - 20.0 - running_dims.width, 20.0, 30.0, running_color);
    }

    fn calculate_total_dist(&mut self) -> f32 {
        let mut dist = 0.0;
        for i in 0..self.tour.len() {
            let c1 = self.cities[self.tour[i]];
            let c2 = self.cities[self.tour[(i + 1)%self.tour.len()]];
            dist += ((c1[0] - c2[0]).powi(2) + (c1[1] - c2[1]).powi(2)).sqrt();
        }
        dist
    }

    fn add_city(&mut self) {
        let city: [f32; 2] = [
            self.rng.random_range(BORDER..(screen_width() - BORDER)),
            self.rng.random_range(BORDER..(screen_height() - BORDER)),
        ];
        self.cities.push(city);
        self.tour.push(self.n);
        self.n += 1;
        self.dist = self.calculate_total_dist();
    }

    // Held-Karp (Classical) exact dynamic programming algorithm
    fn update_classical(&mut self) -> bool {
        // We cap it at 23 cities to prevent integer overflow and Out-Of-Memory crashes.
        if self.n > 23 {
            println!("N is too large for Held-Karp (N > 23). Lower N to use this algorithm.");
            return false;
        }

        let n = self.n;
        
        // Precompute the distance matrix
        let mut dist = vec![vec![0.0_f32; n]; n];
        for i in 0..n {
            for j in 0..n {
                let c1 = self.cities[i];
                let c2 = self.cities[j];
                dist[i][j] = ((c1[0] - c2[0]).powi(2) + (c1[1] - c2[1]).powi(2)).sqrt();
            }
        }

        // DP table and parent array setup for path reconstruction
        // dp[mask][i] = min distance to visit all cities in 'mask', ending at city 'i'
        let mut dp = vec![vec![f32::INFINITY; n]; 1 << n];
        let mut parent = vec![vec![0_usize; n]; 1 << n];

        // Base case: starting at city 0 (mask = 1)
        dp[1][0] = 0.0;

        // Populating the DP table
        for mask in 1..(1 << n) {
            // Only considering states that include our starting city 0
            if mask & 1 == 0 { continue; }

            for u in 0..n {
                // If city u is in the current mask
                if (mask & (1 << u)) != 0 {
                    // Try to reach u from any other city v in the mask
                    for v in 0..n {
                        if u != v && (mask & (1 << v)) != 0 {
                            let prev_mask = mask ^ (1 << u);
                            let cost = dp[prev_mask][v] + dist[v][u];
                            if cost < dp[mask][u] {
                                dp[mask][u] = cost;
                                parent[mask][u] = v;
                            }
                        }
                    }
                }
            }
        }

        // Findint the optimal return path back to city 0
        let final_mask = (1 << n) - 1;
        let mut min_cost = f32::INFINITY;
        let mut last_city = 0;

        for u in 1..n {
            let cost = dp[final_mask][u] + dist[u][0];
            if cost < min_cost {
                min_cost = cost;
                last_city = u;
            }
        }

        // Reconstructing the optimal tour
        let mut best_tour = Vec::new();
        let mut curr_mask = final_mask;
        let mut curr_city = last_city;

        while curr_city != 0 {
            best_tour.push(curr_city);
            let next_city = parent[curr_mask][curr_city];
            curr_mask ^= 1 << curr_city;
            curr_city = next_city;
        }
        best_tour.push(0);
        best_tour.reverse(); // Flip it so the tour begins at 0

        // Applying the optimal results to the state
        self.tour = best_tour;
        self.dist = min_cost;
        
        false
    }
}

fn test_algs(max_cities: usize) {
    let min_cities: usize = 4;
    let n_tests: usize = 10;
    let n_algs: usize = 3;

    let mut times: Vec<Vec<Vec<f32>>> = Vec::new();
    for i in 0..n_algs {
        let helper: Vec<Vec<f32>> = Vec::new();
        times.push(helper);
        for j in 0..=max_cities {
            let helper: Vec<f32> = Vec::new();
            times[i].push(helper);
            for k in 0..n_tests {
                times[i][j].push(0.0);
            }
        }
    }

    let mut results: Vec<Vec<Vec<f32>>> = Vec::new();
    for i in 0..n_algs {
        let helper: Vec<Vec<f32>> = Vec::new();
        results.push(helper);
        for j in 0..=max_cities {
            let helper: Vec<f32> = Vec::new();
            results[i].push(helper);
            for k in 0..n_tests {
                results[i][j].push(0.0);
            }
        }
    }

    let mut state = State::new(min_cities);
    state.randomize();
    for cities in min_cities..=max_cities {
        for nth in 0..n_tests {
            for alg in 0..n_algs {
                state.randomize_tour();
                state.alg = match alg {
                    0_usize => Alg::SA,
                    1_usize => Alg::TWO_OPT,
                    2_usize => Alg::CLASSICAL,
                    _ => unreachable!(),
                };
                let mut not_done = true;
                let now = Instant::now();

                while not_done {
                    match alg {
                        0_usize => not_done = state.update_sa(),
                        1_usize => not_done = state.update_two_opt(),
                        2_usize => not_done = state.update_classical(),
                        _ => unreachable!(),
                    };
                }

                let elapsed = now.elapsed().as_secs_f32() * 1000.0;

                times[alg][cities][nth] = elapsed;
                results[alg][cities][nth] = state.dist;
                println!("{} {} {} {}", alg, cities, nth, elapsed);
            }
        }
        state.add_city();
    }

    for alg in 0..3 {
        let mut time_per_cities = Vec::new();
        let mut result_per_cities = Vec::new();
        for cities in 0..=max_cities {
            let mut sum_times = 0.0;
            let mut sum_results = 0.0;
            for nth in 0..n_tests {
                sum_times += times[alg][cities][nth];
                sum_results += results[alg][cities][nth];
            }
            time_per_cities.push(sum_times/(max_cities - min_cities + 1) as f32);
            result_per_cities.push(sum_results/(max_cities - min_cities + 1) as f32);
        }
        if alg == 0 {
            println!("Simulated Annealing");
        }
        else if alg == 1 {
            println!("2-Opt");
        }
        else {
            println!("Classical");
        }
        println!("Times:");
        if DEBUG_OUTPUT {
            for i in 0..time_per_cities.len() {
                println!("{} {}", i, time_per_cities[i]);
            }
        }
        else {
            for i in min_cities..time_per_cities.len() {
                println!("{} cities: {} avg time", i, time_per_cities[i]);
            }
        }
        println!("Results:");
        if DEBUG_OUTPUT {
            for i in 0..result_per_cities.len() {
                println!("{} {}", i, result_per_cities[i]);
            }
        }
        else {
            for i in min_cities..result_per_cities.len() {
                println!("{} cities: {} avg result", i, result_per_cities[i]);
            }
        }
        println!("");
    }
}

#[macroquad::main("Travelling Salesman Problem")]
async fn main() {

    let args: Vec<String> = env::args().collect();
    let mut n_cities: usize = 4;

    // arguments handling
    if args.len() > 1 {
        if args[1] == "--test" {
            if args.len() > 2 {
                let max_cities = args[2].parse().unwrap_or(4);
                test_algs(max_cities);
            }
            else {
                test_algs(10);
            }
            return;
        }
        if args[1] == "--cities" {
            if args.len() > 2 {
                n_cities = args[2].parse().unwrap_or(4);
            }
        }
    }

    let mut state = State::new(n_cities);
    state.randomize();
    let mut working_next_frame = false;

    // Main loop
    loop {
        // Logic & Input
        if is_key_pressed(KeyCode::Escape) {
            break;
        }
        if is_key_pressed(KeyCode::H) {
            state.randomize();
            state.running = false;
        }
        if is_key_pressed(KeyCode::A) {
            state.randomize_tour();
            state.alg = Alg::SA;
            state.running = true;
        }
        if is_key_pressed(KeyCode::T) {
            state.randomize_tour();
            state.alg = Alg::TWO_OPT;
            state.running = true;
        }
        if is_key_pressed(KeyCode::R) {
            state.randomize_tour();
            state.running = false;
        }
        if is_key_pressed(KeyCode::S) {
            state.running = false;
        }
        if is_key_pressed(KeyCode::C) {
            state.running = true;
            state.alg = Alg::CLASSICAL;
            working_next_frame = true;
        }
        if is_key_pressed(KeyCode::N) {
            state.add_city();
        }
        

        // Rendering
        clear_background(LIGHTGRAY);
        state.display();

        // if state.running == true, the algorithm is running
        if state.running == true {
            if matches!(state.alg, Alg::SA) {
                for _i in 0..10000 {
                    if !state.update_sa() {
                        state.running = false;
                    }
                }
            }
            else if matches!(state.alg, Alg::TWO_OPT) {
                for _i in 0..1 {
                    if !state.update_two_opt() {
                        state.running = false;
                    }
                }
            }
            else if matches!(state.alg, Alg::CLASSICAL) {
                if !working_next_frame {
                    for _i in 0..1 {
                        if !state.update_classical() {
                            state.running = false;
                        }
                    }
                }
                else {
                    working_next_frame = false;
                }
            }
        }

        // Wait for the next frame
        next_frame().await
    }
}