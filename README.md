# Genome Sorter

This Rust project downloads, parses, and processes genome assembly data from
the NCBI database. It filters the data to include only the latest complete
genomes from NCBI RefSeq, extracts the genome sizes of various organisms, and
saves the sorted results into a CSV file. Additionally, it displays the sorted
genome sizes in a formatted manner.

## Usage
Clone the repository:
```
git clone git@github.com:nhukc/genome-sorting.git
cd genome-sorting
```

Run the project:
```rust
cargo run
```
