#![deny(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    use df::{load_font, App};
    use eframe::IconData;
    tracing_subscriber::fmt::init();

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    let _enter = rt.enter();

    let native_options = eframe::NativeOptions {
        icon_data: Some(IconData::try_from_png_bytes(include_bytes!("favicon-32x32.png")).unwrap()),
        ..Default::default()
    };
    eframe::run_native(
        "던파 경매장 탐색기",
        native_options,
        Box::new(|cc| {
            load_font(cc);
            Box::new(App::new(cc))
        }),
    )
    .unwrap();
}

// when compiling to web using trunk.
#[cfg(target_arch = "wasm32")]
fn main() {
    compile_error!("This crate does not support compiling to wasm32.");
    // use df::{load_font, App};

    // eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    // let web_options = eframe::WebOptions::default();

    // // TODO: resolve CORS issue
    // wasm_bindgen_futures::spawn_local(async {
    //     eframe::WebRunner::new()
    //         .start(
    //             "the_canvas_id",
    //             web_options,
    //             Box::new(|cc| {
    //                 load_font(cc);
    //                 Box::new(App::new(cc))
    //             }),
    //         )
    //         .await
    //         .expect("failed to start web runner");
    // });
}
