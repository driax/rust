/**

An implementation of the Graph500 Breadth First Search problem in Rust.

*/

use std;
import std::time;
import std::map;
import std::map::hashmap;
import std::deque;
import std::deque::t;
import std::par;
import io::writer_util;
import comm::*;
import int::abs;

type node_id = i64;
type graph = ~[~[node_id]];
type bfs_result = ~[node_id];

fn make_edges(scale: uint, edgefactor: uint) -> ~[(node_id, node_id)] {
    let r = rand::xorshift();

    fn choose_edge(i: node_id, j: node_id, scale: uint, r: rand::rng)
        -> (node_id, node_id) {

        let A = 0.57;
        let B = 0.19;
        let C = 0.19;
 
        if scale == 0u {
            (i, j)
        }
        else {
            let i = i * 2i64;
            let j = j * 2i64;
            let scale = scale - 1u;
            
            let x = r.gen_float();

            if x < A {
                choose_edge(i, j, scale, r)
            }
            else {
                let x = x - A;
                if x < B {
                    choose_edge(i + 1i64, j, scale, r)
                }
                else {
                    let x = x - B;
                    if x < C {
                        choose_edge(i, j + 1i64, scale, r)
                    }
                    else {
                        choose_edge(i + 1i64, j + 1i64, scale, r)
                    }
                }
            }
        }
    }

    do vec::from_fn((1u << scale) * edgefactor) |_i| {
        choose_edge(0i64, 0i64, scale, r)
    }
}

fn make_graph(N: uint, edges: ~[(node_id, node_id)]) -> graph {
    let graph = do vec::from_fn(N) |_i| {
        map::hashmap::<node_id, ()>(|x| x as uint , |x, y| x == y )
    };

    do vec::each(edges) |e| {
        let (i, j) = e;
        map::set_add(graph[i], j);
        map::set_add(graph[j], i);
        true
    }

    do graph.map() |v| {
        map::vec_from_set(v)
    }
}

fn gen_search_keys(graph: graph, n: uint) -> ~[node_id] {
    let keys = map::hashmap::<node_id, ()>(|x| x as uint , |x, y| x == y );
    let r = rand::rng();

    while keys.size() < n {
        let k = r.gen_uint_range(0u, graph.len());

        if graph[k].len() > 0u && vec::any(graph[k], |i| {
            i != k as node_id
        }) {
            map::set_add(keys, k as node_id);
        }
    }
    map::vec_from_set(keys)
}

/**
 * Returns a vector of all the parents in the BFS tree rooted at key.
 *
 * Nodes that are unreachable have a parent of -1.
 */
fn bfs(graph: graph, key: node_id) -> bfs_result {
    let marks : ~[mut node_id] 
        = vec::to_mut(vec::from_elem(vec::len(graph), -1i64));

    let Q = deque::create();

    Q.add_back(key);
    marks[key] = key;

    while Q.size() > 0u {
        let t = Q.pop_front();

        do graph[t].each() |k| {
            if marks[k] == -1i64 {
                marks[k] = t;
                Q.add_back(k);
            }
            true
        };
    }

    vec::from_mut(marks)
}

/**
 * Another version of the bfs function.
 *
 * This one uses the same algorithm as the parallel one, just without
 * using the parallel vector operators.
 */
fn bfs2(graph: graph, key: node_id) -> bfs_result {
    // This works by doing functional updates of a color vector.

    enum color {
        white,
        // node_id marks which node turned this gray/black.
        // the node id later becomes the parent.
        gray(node_id),
        black(node_id)
    };

    let mut colors = do vec::from_fn(graph.len()) |i| {
        if i as node_id == key {
            gray(key)
        }
        else {
            white
        }
    };

    fn is_gray(c: color) -> bool {
        alt c {
          gray(_) { true }
          _ { false }
        }
    }

    let mut i = 0u;
    while vec::any(colors, is_gray) {
        // Do the BFS.
        log(info, #fmt("PBFS iteration %?", i));
        i += 1u;
        colors = do colors.mapi() |i, c| {
            let c : color = c;
            alt c {
              white {
                let i = i as node_id;
                
                let neighbors = graph[i];
                
                let mut color = white;

                do neighbors.each() |k| {
                    if is_gray(colors[k]) {
                        color = gray(k);
                        false
                    }
                    else { true }
                };

                color
              }
              gray(parent) { black(parent) }
              black(parent) { black(parent) }
            }
        }
    }

    // Convert the results.
    do vec::map(colors) |c| {
        alt c {
          white { -1i64 }
          black(parent) { parent }
          _ { fail "Found remaining gray nodes in BFS" }
        }
    }
}

/// A parallel version of the bfs function.
fn pbfs(&&graph: arc::arc<graph>, key: node_id) -> bfs_result {
    // This works by doing functional updates of a color vector.

    enum color {
        white,
        // node_id marks which node turned this gray/black.
        // the node id later becomes the parent.
        gray(node_id),
        black(node_id)
    };

    let mut colors = do vec::from_fn((*arc::get(&graph)).len()) |i| {
        if i as node_id == key {
            gray(key)
        }
        else {
            white
        }
    };

    #[inline(always)]
    fn is_gray(c: color) -> bool {
        alt c {
          gray(_) { true }
          _ { false }
        }
    }

    let mut i = 0u;
    while par::any(colors, is_gray) {
        // Do the BFS.
        log(info, #fmt("PBFS iteration %?", i));
        i += 1u;
        let old_len = colors.len();

        let color = arc::arc(colors);

        colors = do par::mapi_factory(*arc::get(&color)) {
            let colors = arc::clone(&color);
            let graph = arc::clone(&graph);
            fn~(i: uint, c: color) -> color {
                let c : color = c;
                let colors = arc::get(&colors);
                let graph = arc::get(&graph);
                alt c {
                  white {
                    let i = i as node_id;
                    
                    let neighbors = graph[i];
                    
                    let mut color = white;
                    
                    do neighbors.each() |k| {
                        if is_gray(colors[k]) {
                            color = gray(k);
                            false
                        }
                        else { true }
                    };
                    color
                  }
                  gray(parent) { black(parent) }
                  black(parent) { black(parent) }
                }
            }
        };
        assert(colors.len() == old_len);
    }

    // Convert the results.
    do par::map(colors) |c| {
        alt c {
          white { -1i64 }
          black(parent) { parent }
          _ { fail "Found remaining gray nodes in BFS" }
        }
    }
}

/// Performs at least some of the validation in the Graph500 spec.
fn validate(edges: ~[(node_id, node_id)], 
            root: node_id, tree: bfs_result) -> bool {
    // There are 5 things to test. Below is code for each of them.

    // 1. The BFS tree is a tree and does not contain cycles.
    //
    // We do this by iterating over the tree, and tracing each of the
    // parent chains back to the root. While we do this, we also
    // compute the levels for each node.

    log(info, "Verifying tree structure...");

    let mut status = true;
    let level = do tree.map() |parent| {
        let mut parent = parent;
        let mut path = ~[];

        if parent == -1i64 {
            // This node was not in the tree.
            -1
        }
        else {
            while parent != root {
                if vec::contains(path, parent) {
                    status = false;
                }

                vec::push(path, parent);
                parent = tree[parent];
            }

            // The length of the path back to the root is the current
            // level.
            path.len() as int
        }
    };
    
    if !status { ret status }

    // 2. Each tree edge connects vertices whose BFS levels differ by
    //    exactly one.

    log(info, "Verifying tree edges...");

    let status = do tree.alli() |k, parent| {
        if parent != root && parent != -1i64 {
            level[parent] == level[k] - 1
        }
        else {
            true
        }
    };

    if !status { ret status }

    // 3. Every edge in the input list has vertices with levels that
    //    differ by at most one or that both are not in the BFS tree.

    log(info, "Verifying graph edges...");

    let status = do edges.all() |e| {
        let (u, v) = e;

        abs(level[u] - level[v]) <= 1
    };

    if !status { ret status }    

    // 4. The BFS tree spans an entire connected component's vertices.

    // This is harder. We'll skip it for now...

    // 5. A node and its parent are joined by an edge of the original
    //    graph.

    log(info, "Verifying tree and graph edges...");

    let status = do par::alli(tree) |u, v| {
        let u = u as node_id;
        if v == -1i64 || u == root {
            true
        }
        else {
            edges.contains((u, v)) || edges.contains((v, u))
        }
    };

    if !status { ret status }    

    // If we get through here, all the tests passed!
    true
}

fn main(args: ~[str]) {
    let args = if os::getenv("RUST_BENCH").is_some() {
        ~["", "15", "48"]
    } else if args.len() <= 1u {
        ~["", "10", "16"]
    } else {
        args
    };

    let scale = uint::from_str(args[1]).get();
    let num_keys = uint::from_str(args[2]).get();
    let do_validate = false;
    let do_sequential = true;

    let start = time::precise_time_s();
    let edges = make_edges(scale, 16u);
    let stop = time::precise_time_s();

    io::stdout().write_line(#fmt("Generated %? edges in %? seconds.",
                                 vec::len(edges), stop - start));

    let start = time::precise_time_s();
    let graph = make_graph(1u << scale, edges);
    let stop = time::precise_time_s();

    let mut total_edges = 0u;
    vec::each(graph, |edges| { total_edges += edges.len(); true });

    io::stdout().write_line(#fmt("Generated graph with %? edges in %? seconds.",
                                 total_edges / 2u,
                                 stop - start));

    let mut total_seq = 0.0;
    let mut total_par = 0.0;

    let graph_arc = arc::arc(copy graph);

    do gen_search_keys(graph, num_keys).map() |root| {
        io::stdout().write_line("");
        io::stdout().write_line(#fmt("Search key: %?", root));

        if do_sequential {
            let start = time::precise_time_s();
            let bfs_tree = bfs(graph, root);
            let stop = time::precise_time_s();
            
            //total_seq += stop - start;

            io::stdout().write_line(
                #fmt("Sequential BFS completed in %? seconds.",
                     stop - start));
            
            if do_validate {
                let start = time::precise_time_s();
                assert(validate(edges, root, bfs_tree));
                let stop = time::precise_time_s();
                
                io::stdout().write_line(
                    #fmt("Validation completed in %? seconds.",
                         stop - start));
            }
            
            let start = time::precise_time_s();
            let bfs_tree = bfs2(graph, root);
            let stop = time::precise_time_s();
            
            total_seq += stop - start;
            
            io::stdout().write_line(
                #fmt("Alternate Sequential BFS completed in %? seconds.",
                     stop - start));
            
            if do_validate {
                let start = time::precise_time_s();
                assert(validate(edges, root, bfs_tree));
                let stop = time::precise_time_s();
                
                io::stdout().write_line(
                    #fmt("Validation completed in %? seconds.",
                         stop - start));
            }
        }
        
        let start = time::precise_time_s();
        let bfs_tree = pbfs(graph_arc, root);
        let stop = time::precise_time_s();

        total_par += stop - start;

        io::stdout().write_line(#fmt("Parallel BFS completed in %? seconds.",
                                     stop - start));

        if do_validate {
            let start = time::precise_time_s();
            assert(validate(edges, root, bfs_tree));
            let stop = time::precise_time_s();
            
            io::stdout().write_line(#fmt("Validation completed in %? seconds.",
                                         stop - start));
        }
    };

    io::stdout().write_line("");
    io::stdout().write_line(
        #fmt("Total sequential: %? \t Total Parallel: %? \t Speedup: %?x",
             total_seq, total_par, total_seq / total_par));
}
