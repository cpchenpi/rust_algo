use recursive_function::{Callable2, RecursiveFunction2};

/// return fa, bridge \
/// bridge\[u\] means edge between u and fa\[u\] is bridge
pub fn tarjan_bridge(adj: &Vec<Vec<usize>>) -> (Vec<usize>, Vec<bool>) {
    let n = adj.len();
    let mut fa = vec![n; n];
    let mut low = vec![0; n];
    let mut dfn = vec![0; n];
    let mut time = 0;
    let mut bridge = vec![false; n];
    for i in 0..n {
        if dfn[i] == 0 {
            let mut dfs = RecursiveFunction2::new(|sf, u: usize, f: usize| {
                fa[u] = f;
                time += 1;
                dfn[u] = time;
                low[u] = time;
                for &v in &adj[u] {
                    if dfn[v] == 0 {
                        sf.call(v, u);
                        low[u] = low[u].min(low[v]);
                        if low[v] > dfn[u] {
                            bridge[v] = true;
                        }
                    } else if dfn[v] < dfn[u] && v != f {
                        low[u] = low[u].min(dfn[v]);
                    }
                }
            });
            dfs.call(i, n);
        }
    }
    (fa, bridge)
}
