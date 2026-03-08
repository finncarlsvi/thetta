//! Utility binary to generate test STEP files.
//! Run with: cargo test -p thetta-step --features occt --test gen_test_step

#![cfg(feature = "occt")]

use opencascade_sys::ffi;

#[test]
fn generate_test_step_files() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("test_data/step");
    std::fs::create_dir_all(&dir).unwrap();

    // Simple sheet-metal parts: flat boxes of varying sizes and thicknesses
    generate_single_box(&dir.join("single_box_20x10x2.step"), 20.0, 10.0, 2.0);
    generate_multi_thickness(&dir.join("multi_thickness.step"));
    eprintln!("Generated test STEP files in {}", dir.display());
}

fn generate_single_box(path: &std::path::Path, w: f64, h: f64, t: f64) {
    let origin = ffi::new_point(0.0, 0.0, 0.0);
    let mut maker = ffi::BRepPrimAPI_MakeBox_ctor(&origin, w, h, t);
    let shape = maker.pin_mut().Shape();

    let mut writer = ffi::STEPControl_Writer_ctor();
    ffi::transfer_shape(writer.pin_mut(), shape);
    ffi::write_step(writer.pin_mut(), path.to_str().unwrap().to_string());
}

fn generate_multi_thickness(path: &std::path::Path) {
    let mut writer = ffi::STEPControl_Writer_ctor();

    // 3 boxes at 2mm thickness
    for i in 0..3 {
        let origin = ffi::new_point(i as f64 * 30.0, 0.0, 0.0);
        let mut maker = ffi::BRepPrimAPI_MakeBox_ctor(&origin, 20.0, 10.0, 2.0);
        ffi::transfer_shape(writer.pin_mut(), maker.pin_mut().Shape());
    }

    // 1 box at 5mm thickness
    let origin = ffi::new_point(90.0, 0.0, 0.0);
    let mut maker = ffi::BRepPrimAPI_MakeBox_ctor(&origin, 15.0, 15.0, 5.0);
    ffi::transfer_shape(writer.pin_mut(), maker.pin_mut().Shape());

    ffi::write_step(writer.pin_mut(), path.to_str().unwrap().to_string());
}
