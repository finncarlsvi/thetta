/// Diagnostic: load a STEP file, extract profiles, and write them to an SVG.
/// Usage: cargo run --example dump_profiles -- /path/to/file.step [thickness_mm]
use std::path::Path;
use thetta_step::{ExtractionConfig, import_step};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: dump_profiles <step_file> [thickness_mm]");
        std::process::exit(1);
    }
    let path = Path::new(&args[1]);
    let thickness: Option<f64> = args.get(2).and_then(|s| s.parse().ok());
    let config = ExtractionConfig::default();

    eprintln!("Loading {:?} ...", path);
    let result = import_step(path, thickness, &config).unwrap_or_else(|e| {
        eprintln!("Import failed: {e}");
        std::process::exit(1);
    });

    eprintln!("Thickness groups:");
    for g in &result.thicknesses {
        eprintln!("  {:.3} mm  ({} solids)", g.value_mm, g.count);
    }
    eprintln!("Selected: {:.3} mm", result.selected_thickness_mm);
    eprintln!("Pieces: {}", result.pieces.len());

    // Compute global bounding box
    let mut gmin_x = f64::INFINITY;
    let mut gmin_y = f64::INFINITY;
    let mut gmax_x = f64::NEG_INFINITY;
    let mut gmax_y = f64::NEG_INFINITY;
    for piece in &result.pieces {
        for pt in &piece.polygon.outer {
            gmin_x = gmin_x.min(pt.x);
            gmin_y = gmin_y.min(pt.y);
            gmax_x = gmax_x.max(pt.x);
            gmax_y = gmax_y.max(pt.y);
        }
    }

    let padding = 10.0;
    let cols = (result.pieces.len() as f64).sqrt().ceil() as usize;
    let cell_w = (gmax_x - gmin_x) + padding * 2.0;
    let cell_h = (gmax_y - gmin_y) + padding * 2.0;

    eprintln!(
        "Profile bbox: ({:.1}, {:.1}) - ({:.1}, {:.1})",
        gmin_x, gmin_y, gmax_x, gmax_y
    );
    eprintln!(
        "Profile size: {:.1} x {:.1}",
        gmax_x - gmin_x,
        gmax_y - gmin_y
    );

    // Also dump individual piece stats
    for (i, piece) in result.pieces.iter().enumerate() {
        let mut pmin_x = f64::INFINITY;
        let mut pmin_y = f64::INFINITY;
        let mut pmax_x = f64::NEG_INFINITY;
        let mut pmax_y = f64::NEG_INFINITY;
        for pt in &piece.polygon.outer {
            pmin_x = pmin_x.min(pt.x);
            pmin_y = pmin_y.min(pt.y);
            pmax_x = pmax_x.max(pt.x);
            pmax_y = pmax_y.max(pt.y);
        }
        eprintln!(
            "  Piece {} (id={}): {} outer pts, {} holes, bbox {:.1}x{:.1}",
            i,
            piece.id.0,
            piece.polygon.outer.len(),
            piece.polygon.holes.len(),
            pmax_x - pmin_x,
            pmax_y - pmin_y,
        );
    }

    let rows = (result.pieces.len() + cols - 1) / cols;
    let svg_w = cols as f64 * cell_w;
    let svg_h = rows as f64 * cell_h;

    let mut svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {svg_w} {svg_h}" width="{}" height="{}">"##,
        svg_w.min(1600.0),
        svg_h.min(1600.0),
    );
    svg.push_str(r##"<rect width="100%" height="100%" fill="#222"/>"##);

    let colors = [
        "#E67E22", "#2E86AB", "#2A9D8F", "#BC4749", "#8D6E63", "#3A86FF",
    ];

    for (i, piece) in result.pieces.iter().enumerate() {
        let col = i % cols;
        let row = i / cols;
        let ox = col as f64 * cell_w + padding - gmin_x;
        let oy = row as f64 * cell_h + padding - gmin_y;

        let points: String = piece
            .polygon
            .outer
            .iter()
            .map(|pt| format!("{:.2},{:.2}", pt.x + ox, pt.y + oy))
            .collect::<Vec<_>>()
            .join(" ");

        let color = colors[i % colors.len()];
        svg.push_str(&format!(
            r##"<polygon points="{points}" fill="{color}" fill-opacity="0.5" stroke="{color}" stroke-width="0.5"/>"##,
        ));

        // Draw holes
        for hole in &piece.polygon.holes {
            let hpoints: String = hole
                .iter()
                .map(|pt| format!("{:.2},{:.2}", pt.x + ox, pt.y + oy))
                .collect::<Vec<_>>()
                .join(" ");
            svg.push_str(&format!(
                r##"<polygon points="{hpoints}" fill="#222" stroke="#888" stroke-width="0.3"/>"##,
            ));
        }

        // Label
        let cx = piece.polygon.outer.iter().map(|p| p.x).sum::<f64>()
            / piece.polygon.outer.len() as f64
            + ox;
        let cy = piece.polygon.outer.iter().map(|p| p.y).sum::<f64>()
            / piece.polygon.outer.len() as f64
            + oy;
        let font_size = (cell_w.min(cell_h) * 0.05).max(3.0).min(14.0);
        svg.push_str(&format!(
            r##"<text x="{cx:.1}" y="{cy:.1}" fill="white" font-size="{font_size:.1}" text-anchor="middle" dominant-baseline="central">{i}</text>"##,
        ));
    }

    svg.push_str("</svg>");

    let out_path = path.with_extension("svg");
    std::fs::write(&out_path, &svg).unwrap();
    eprintln!("Wrote {}", out_path.display());
}
