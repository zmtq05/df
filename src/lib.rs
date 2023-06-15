#![warn(clippy::all, rust_2018_idioms)]

mod image_storage;
mod ui;

use std::{future::Future, mem};

use crate::image_storage::ImageStorage;
use df_client::{
    api::{auction::SortOrder, WordType},
    model::AuctionInfo,
};
use egui::{FontData, FontDefinitions, FontFamily};
use poll_promise::Promise;

type Result<T, E = df_client::Error> = std::result::Result<T, E>;

#[derive(Default)]
pub struct App {
    input: String,

    search_state: SearchState,

    images: ImageStorage,
}

struct SearchState {
    results: PromiseState<Vec<AuctionInfo>, df_client::Error>,
    error_msg: String,
    promise: Option<Promise<Result<Vec<AuctionInfo>, df_client::Error>>>,
    sort_asc: bool,
}

impl Default for SearchState {
    fn default() -> Self {
        Self {
            results: Default::default(),
            error_msg: Default::default(),
            promise: Default::default(),
            sort_asc: true,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn spawn_promise<F, T>(future: F) -> Promise<T>
where
    F: Future<Output = T> + 'static + Send,
    T: 'static + Send,
{
    Promise::spawn_async(future)
}

#[cfg(target_arch = "wasm32")]
fn spawn_promise<F, T>(future: F) -> Promise<T>
where
    F: Future<Output = T> + 'static,
    T: 'static + Send,
{
    Promise::spawn_local(future)
}

impl App {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        df_client::initialise(env!("API_KEY"));

        App::default()
    }

    fn search(&mut self) {
        if self.input.is_empty() {
            self.search_state.error_msg = "아이템명을 입력해주세요.".to_owned();
            return;
        }

        let item_name = self.input.clone();
        self.search_state.promise = Some(spawn_promise(async move {
            df_client::instance()
                .auction()
                .item_name(item_name)
                .limit(50)
                .word_type(WordType::Front)
                .sort_by_unit_price(SortOrder::Asc)
                .search()
                .await
        }));
    }

    fn update_state(&mut self) {
        let new = match self.search_state.promise.as_ref() {
            None => PromiseState::None,
            Some(promise) => match promise.ready() {
                None => PromiseState::Pending,
                Some(_) => match self.search_state.promise.take().unwrap().block_and_take() {
                    Ok(val) => PromiseState::Ok(val),
                    Err(e) => PromiseState::Err(e),
                },
            },
        };

        let old = &mut self.search_state.results;
        if new.is_done() || !old.is_done() {
            old.replace(new);
            self.search_state.sort_asc = true;
        }

        self.images.update_state();
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ui::draw(self, ctx);

        self.update_state();
    }
}

pub fn load_font(cc: &eframe::CreationContext<'_>) {
    let mut fonts = FontDefinitions::default();
    egui_phosphor::add_to_fonts(&mut fonts);

    let mut phosphor_data = fonts.font_data.get_mut("phosphor").unwrap();
    phosphor_data.tweak = egui::FontTweak {
        y_offset: 2.5,
        ..Default::default()
    };

    fonts.font_data.insert(
        "NanumGothic".to_owned(),
        FontData::from_static(include_bytes!("NanumGothic.ttf")),
    );
    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, "NanumGothic".to_owned());

    cc.egui_ctx.set_fonts(fonts);
}

#[derive(Default)]
enum PromiseState<T, E> {
    #[default]
    None,
    Pending,
    Ok(T),
    Err(E),
}

impl<T, E> PromiseState<T, E> {
    fn is_done(&self) -> bool {
        matches!(self, Self::Ok(_) | Self::Err(_))
    }

    fn replace(&mut self, new: Self) -> Self {
        mem::replace(self, new)
    }

    fn get_mut(&mut self) -> Option<&mut T> {
        match self {
            Self::Ok(val) => Some(val),
            _ => None,
        }
    }
}
