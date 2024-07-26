use reqwest::blocking::get;
use csv::ReaderBuilder;
use indicatif::{ProgressBar, ProgressStyle};
use std::collections::HashMap;
use std::error::Error;

const BASE_URL: &str = "https://ftp.ncbi.nlm.nih.gov/genomes/ASSEMBLY_REPORTS/assembly_summary_refseq.txt";

struct AssemblyRecord {
    organism_name: String,
    genome_size: Option<f64>,
}

fn download_assembly_summary() -> Result<String, Box<dyn Error>> {
    println!("Downloading assembly summary...");
    let response = get(BASE_URL)?.text()?;
    println!("Download successful.");
    Ok(response)
}

fn print_sample_lines(data: &str, n: usize) {
    for (i, line) in data.lines().take(n).enumerate() {
        println!("Line {}: {}", i + 1, line);
    }
}

fn parse_assembly_summary(data: &str) -> Result<Vec<AssemblyRecord>, Box<dyn Error>> {
    println!("Parsing assembly summary...");

    let mut reader = ReaderBuilder::new()
        .delimiter(b'\t')
        .has_headers(false)
        .flexible(true)
        .from_reader(data.as_bytes());

    let headers = reader.headers()?.clone();
    println!("Headers: {:?}", headers);

    let mut records = Vec::new();
    for result in reader.records() {
        let record = result?;
        if record.get(10) == Some("latest") &&
           record.get(11) == Some("Complete Genome") &&
           record.get(31) == Some("NCBI RefSeq") {
            // Adjust the indices based on the actual headers
            let organism_name = record.get(7).unwrap_or("").to_string();
            let genome_size = record.get(25).and_then(|s| s.parse().ok());
            records.push(AssemblyRecord { organism_name, genome_size });
        }
    }
    Ok(records)
}

fn get_genome_sizes(records: Vec<AssemblyRecord>) -> HashMap<String, f64> {
    let pb = ProgressBar::new(records.len() as u64);
    pb.set_style(ProgressStyle::default_bar()
        .template("{msg}\n{wide_bar} {pos}/{len} ({percent}%)")
        .progress_chars("=>-"));

    let mut genome_sizes = HashMap::new();
    for record in records {
        let message = format!("Processing {}", record.organism_name);
        pb.set_message(message);
        if let Some(size) = record.genome_size {
            genome_sizes.entry(record.organism_name.clone())
                .and_modify(|e| {
                    if *e < size {
                        *e = size;
                    }
                })
                .or_insert(size);
        }
        pb.inc(1);
    }
    pb.finish_with_message("Processing complete");
    genome_sizes
}

fn save_genome_sizes(genome_sizes: HashMap<String, f64>, output_file: &str) -> Result<(), Box<dyn Error>> {
    let mut wtr = csv::Writer::from_path(output_file)?;

    wtr.write_record(&["Species", "Genome Size (bp)"])?;
    let mut sorted_genome_sizes: Vec<_> = genome_sizes.iter().collect();
    sorted_genome_sizes.sort_by(|a, b| a.1.partial_cmp(b.1).unwrap());

    for (species, size) in sorted_genome_sizes {
        wtr.write_record(&[species, &format!("{:.0}", size)])?;
    }
    wtr.flush()?;
    println!("\nSorted genome sizes have been saved to {}", output_file);
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let assembly_summary_data = download_assembly_summary()?;

    // Print the first few lines to understand the structure
    print_sample_lines(&assembly_summary_data, 5);

    let filtered_data: String = assembly_summary_data.lines()
        .filter(|line| !line.starts_with('#'))
        .collect::<Vec<&str>>()
        .join("\n");

    let assembly_records = parse_assembly_summary(&filtered_data)?;
    let genome_sizes = get_genome_sizes(assembly_records);

    let output_file = "sorted_genome_sizes.csv";
    save_genome_sizes(genome_sizes.clone(), output_file)?;

    println!("\nSpecies by Genome Size (Base Pairs):");
    println!("{:<50} {:>20}", "Species", "Genome Size (bp)");
    println!("{:-<70}", "");

    let mut sorted_genome_sizes: Vec<_> = genome_sizes.iter().collect();
    sorted_genome_sizes.sort_by(|a, b| a.1.partial_cmp(b.1).unwrap());

    for (species, size) in sorted_genome_sizes {
        println!("{:<50} {:>20.0}", species, size);
    }

    Ok(())
}

