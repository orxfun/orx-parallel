use orx_parallel::*;

// ── In-memory directory tree ──────────────────────────────────────────────────

/// Represents a directory in the file system.
struct Dir {
    name: &'static str,
    /// Number of files directly inside this directory (not counting subdirs).
    file_count: usize,
    children: Vec<Dir>,
}

impl Dir {
    fn new(name: &'static str, file_count: usize, children: Vec<Dir>) -> Self {
        Self {
            name,
            file_count,
            children,
        }
    }

    /// Sequential total file count for verification.
    fn total_files(&self) -> usize {
        self.file_count + self.children.iter().map(|c| c.total_files()).sum::<usize>()
    }

    /// Sequential total files only in leaf directories, for verification.
    fn total_files_in_leaves(&self) -> usize {
        match self.children.is_empty() {
            true => self.file_count,
            false => self
                .children
                .iter()
                .map(|c| c.total_files_in_leaves())
                .sum(),
        }
    }
}

// ── Build a realistic-looking synthetic file tree ─────────────────────────────

fn build_tree() -> Dir {
    Dir::new(
        "project",
        3,
        vec![
            Dir::new(
                "src",
                0,
                vec![
                    Dir::new("core", 8, vec![Dir::new("tests", 5, vec![])]),
                    Dir::new("utils", 4, vec![]),
                    Dir::new("models", 6, vec![Dir::new("tests", 3, vec![])]),
                ],
            ),
            Dir::new(
                "docs",
                12,
                vec![Dir::new("api", 7, vec![]), Dir::new("guides", 9, vec![])],
            ),
            Dir::new(
                "tests",
                0,
                vec![
                    Dir::new("unit", 15, vec![]),
                    Dir::new("integration", 10, vec![]),
                    Dir::new("fixtures", 4, vec![]),
                ],
            ),
            Dir::new("scripts", 6, vec![]),
            Dir::new("config", 3, vec![Dir::new("env", 2, vec![])]),
        ],
    )
}

// ── Parallel recursive traversal ─────────────────────────────────────────────

fn main() {
    let root = build_tree();

    // `into_par_recursive` starts with an initial set of items — here just the
    // root directory — and an `extend` function that enqueues each node's
    // children when the node is processed.
    //
    // The queue grows dynamically as threads discover new nodes, so the full
    // traversal order need not be known upfront. Threads keep pulling from the
    // shared queue until it is empty.
    //
    // The explicit lifetime `'a` on `extend` unifies the lifetime of items
    // stored inside the queue (`&&'a Dir`) with the lifetime of the items being
    // pushed (`dir.children.iter()` yields `&'a Dir`). This is required because
    // the queue's internal storage is invariant over its element type.
    fn extend<'a>(dir: &&'a Dir, queue: &Queue<&'a Dir>) {
        queue.extend(dir.children.iter());
    }

    // After `into_par_recursive` we have a regular `ParIter` — all the usual
    // iterator adaptors work here, just as on any other parallel iterator.
    let total_files: usize = [&root]
        .into_par_recursive(extend)
        .map(|dir| dir.file_count)
        .sum();

    assert_eq!(total_files, root.total_files());
    println!("Total files under '{}': {total_files}", root.name);

    // ── Counting only files in leaf directories ───────────────────────────────
    // Because we have a full `ParIter` we can chain `filter` to restrict the
    // computation to a subset of nodes — here, directories with no children.
    let files_in_leaves: usize = [&root]
        .into_par_recursive(extend)
        .filter(|dir| dir.children.is_empty())
        .map(|dir| dir.file_count)
        .sum();

    assert_eq!(files_in_leaves, root.total_files_in_leaves());
    println!("Files in leaf directories only: {files_in_leaves}");

    // ── Collecting directory names matching a pattern ─────────────────────────
    // `collect` works too — here we gather the names of all `tests` directories
    // anywhere in the tree.
    let mut test_dirs: Vec<&str> = [&root]
        .into_par_recursive(extend)
        .filter(|dir| dir.name == "tests")
        .map(|dir| dir.name)
        .collect();

    test_dirs.sort_unstable();
    assert_eq!(test_dirs, vec!["tests", "tests", "tests"]);
    println!("Found {} 'tests' directories", test_dirs.len());
}
