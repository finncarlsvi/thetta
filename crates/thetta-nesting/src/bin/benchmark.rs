use jagua_rs::geometry::geo_enums::RotationRange;
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use thetta_geometry::types::{Piece, SheetConfig};
use thetta_nesting::benchmark::{known_best, load_benchmark};
use thetta_nesting::compression::compress_strip;
use thetta_nesting::engine::NestingEngine;
use thetta_nesting::gls::{GLSOptimizer, OptimizerConfig};
use thetta_nesting::lbf::{LbfConfig, run_lbf};

#[derive(Debug, Default)]
struct Args {
    input: Option<PathBuf>,
    all: bool,
    time: f64,
    csv: Option<PathBuf>,
}

#[derive(Debug, Clone)]
struct Row {
    instance: String,
    placed: usize,
    total: usize,
    strip_length: f64,
    utilization: f64,
    best_known: Option<f64>,
    ratio: Option<f64>,
    elapsed_s: f64,
}

fn main() -> anyhow::Result<()> {
    let args = parse_args()?;
    if !args.all && args.input.is_none() {
        anyhow::bail!("provide --input <path> or --all");
    }

    let inputs = collect_inputs(&args)?;
    let mut rows = Vec::new();

    for path in inputs {
        let (pieces, sheet, orientations, name) = load_input(&path)?;
        let row = run_instance(
            &name,
            &pieces,
            sheet,
            Some(orientations),
            Duration::from_secs_f64(args.time.max(0.1)),
        );
        rows.push(row);
    }

    print_table(&rows);

    if let Some(csv_path) = args.csv {
        write_csv(&csv_path, &rows)?;
        eprintln!("wrote {}", csv_path.display());
    }

    Ok(())
}

fn parse_args() -> anyhow::Result<Args> {
    let mut args = Args {
        time: 60.0,
        ..Args::default()
    };

    let argv = env::args().skip(1).collect::<Vec<_>>();
    let mut i = 0;
    while i < argv.len() {
        match argv[i].as_str() {
            "--all" => {
                args.all = true;
                i += 1;
            }
            "--input" => {
                let Some(val) = argv.get(i + 1) else {
                    anyhow::bail!("--input requires a path value");
                };
                args.input = Some(PathBuf::from(val));
                i += 2;
            }
            "--time" => {
                let Some(val) = argv.get(i + 1) else {
                    anyhow::bail!("--time requires a numeric value");
                };
                args.time = val.parse::<f64>()?;
                i += 2;
            }
            "--csv" => {
                let Some(val) = argv.get(i + 1) else {
                    anyhow::bail!("--csv requires a path value");
                };
                args.csv = Some(PathBuf::from(val));
                i += 2;
            }
            "-h" | "--help" => {
                println!(
                    "benchmark --input <file.json> [--time <sec>] [--csv <out.csv>]\nbenchmark --all [--time <sec>] [--csv <out.csv>]"
                );
                std::process::exit(0);
            }
            other => anyhow::bail!("unknown argument: {other}"),
        }
    }

    Ok(args)
}

fn collect_inputs(args: &Args) -> anyhow::Result<Vec<PathBuf>> {
    if args.all {
        let dir = Path::new("research/repos/sparrow/data/input");
        let mut files = fs::read_dir(dir)?
            .filter_map(|e| e.ok().map(|x| x.path()))
            .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("json"))
            .collect::<Vec<_>>();
        files.sort();
        Ok(files)
    } else {
        Ok(vec![args.input.clone().expect("validated")])
    }
}

fn load_input(
    path: &Path,
) -> anyhow::Result<(Vec<Piece>, SheetConfig, Vec<RotationRange>, String)> {
    let text = fs::read_to_string(path)?;
    let value: serde_json::Value = serde_json::from_str(&text)?;

    let is_benchmark = value.get("name").is_some() && value.get("items").is_some();
    if is_benchmark {
        let (pieces, sheet, meta) = load_benchmark(path)?;
        Ok((pieces, sheet, meta.orientations, meta.name))
    } else {
        let mut pieces: Vec<Piece> = serde_json::from_value(value)?;
        for p in &mut pieces {
            p.source_file = path.to_path_buf();
        }
        let orientations = (0..pieces.len())
            .map(|_| RotationRange::Continuous)
            .collect::<Vec<_>>();
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("native")
            .to_string();
        Ok((pieces, SheetConfig::default(), orientations, name))
    }
}

fn run_instance(
    name: &str,
    pieces: &[Piece],
    sheet: SheetConfig,
    orientations: Option<Vec<RotationRange>>,
    time_limit: Duration,
) -> Row {
    let start = Instant::now();

    let mut engine = NestingEngine::new_with_orientations(pieces, &sheet, orientations.as_deref());
    let _baseline = run_lbf(
        &mut engine,
        LbfConfig {
            orientations: orientations.clone(),
            ..LbfConfig::default()
        },
        None,
        |_| {},
    );

    let mut gls = GLSOptimizer::new(
        engine,
        OptimizerConfig {
            time_limit_secs: time_limit.as_secs_f64(),
            ..OptimizerConfig::default()
        },
    );
    let gls_result = gls.run(|_| {}, |_, _, _, _| {}, time_limit, None);

    let compressed = compress_strip(pieces, sheet, 5.0, orientations.as_deref(), None);

    let result = if compressed.unplaced.len() <= gls_result.unplaced.len() {
        compressed
    } else {
        gls_result
    };

    let placed = result.sheets.first().map_or(0, |s| s.placements.len());
    let total = pieces.len();
    let strip_length = result.sheets.first().map(|s| s.strip_length).unwrap_or(0.0);
    let utilization = result.sheets.first().map(|s| s.utilization).unwrap_or(0.0);
    let best = known_best(name);
    let ratio = best.map(|b| strip_length / b);

    Row {
        instance: name.to_string(),
        placed,
        total,
        strip_length,
        utilization,
        best_known: best,
        ratio,
        elapsed_s: start.elapsed().as_secs_f64(),
    }
}

fn print_table(rows: &[Row]) {
    println!(
        "{:<12} | {:<9} | {:<12} | {:<11} | {:<10} | {:<7} | {:<8}",
        "Instance", "Pieces", "Strip Length", "Utilization", "Best Known", "Ratio", "Time (s)"
    );
    println!(
        "{:-<12}-+-{:-<9}-+-{:-<12}-+-{:-<11}-+-{:-<10}-+-{:-<7}-+-{:-<8}",
        "", "", "", "", "", "", ""
    );

    for row in rows {
        let best = row
            .best_known
            .map(|v| format!("{v:.3}"))
            .unwrap_or_else(|| "-".to_string());
        let ratio = row
            .ratio
            .map(|v| format!("{v:.3}"))
            .unwrap_or_else(|| "-".to_string());
        println!(
            "{:<12} | {:>2}/{:<6} | {:>12.3} | {:>10.2}% | {:>10} | {:>7} | {:>8.1}",
            row.instance,
            row.placed,
            row.total,
            row.strip_length,
            row.utilization * 100.0,
            best,
            ratio,
            row.elapsed_s
        );
    }
}

fn write_csv(path: &Path, rows: &[Row]) -> anyhow::Result<()> {
    let mut file = fs::File::create(path)?;
    writeln!(
        file,
        "instance,pieces_placed,pieces_total,strip_length,utilization,best_known,ratio,time_s"
    )?;
    for row in rows {
        writeln!(
            file,
            "{},{},{},{:.6},{:.6},{},{},{}",
            row.instance,
            row.placed,
            row.total,
            row.strip_length,
            row.utilization,
            row.best_known
                .map(|v| format!("{v:.6}"))
                .unwrap_or_default(),
            row.ratio.map(|v| format!("{v:.6}")).unwrap_or_default(),
            row.elapsed_s
        )?;
    }
    Ok(())
}
