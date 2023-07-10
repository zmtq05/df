use df_rs::model::AuctionInfo;
use egui::{
    Align, CentralPanel, Context, Event, FontId, InputState, Key, Layout, Spinner, TextEdit, Ui,
    WidgetText,
};
use egui_extras::{Column, TableBuilder, TableRow};
use thousands::Separable;

pub(crate) fn draw(app: &mut crate::App, ctx: &Context) {
    CentralPanel::default().show(ctx, |ui| {
        // ctx.set_debug_on_hover(true);

        ui.vertical_centered_justified(|ui| {
            let error_exists = !app.search_state.error_msg.is_empty();

            ui.horizontal(|ui| {
                let input = TextEdit::singleline(&mut app.input)
                    .horizontal_align(Align::Center)
                    .hint_text("검색할 아이템")
                    .font(FontId::proportional(15.))
                    .show(ui)
                    .response;

                ui.hyperlink_to(
                    "Powered by Neople OpenAPI",
                    "https://developers.neople.co.kr",
                );

                if error_exists {
                    show_error_label(ui, "아이템명을 입력해주세요.");
                }

                if ui.input(|i| i.key_pressed(Key::Enter)) && input.lost_focus() {
                    input.request_focus();
                    app.search();
                }

                if ui.input(|i| key_pressed_hack(i, Key::S)) {
                    input.request_focus();
                }

                if input.changed() {
                    app.search_state.error_msg.clear();
                }
            });

            ui.separator();

            ui.vertical_centered_justified(|ui| {
                match app.search_state.results {
                    crate::PromiseState::None => {
                        // ui.centered_and_justified(|ui| ui.label(RichText::new("...").size(15.)));
                    }
                    crate::PromiseState::Pending => {
                        ui.centered_and_justified(|ui| ui.add(Spinner::new().size(15.)));
                    }
                    crate::PromiseState::Err(ref e) => {
                        show_error_label(ui, &e.to_string());
                    }
                    crate::PromiseState::Ok(_) => show_auction_results(ui, app),
                }
            });
        })
    });
}

fn show_auction_results(ui: &mut Ui, app: &mut crate::App) {
    let results = app.search_state.results.get_mut().unwrap();
    TableBuilder::new(ui)
        .striped(true)
        // .auto_shrink([false, true])
        .column(Column::exact(30.))
        .columns(Column::auto(), 4)
        .header(20., |mut header| {
            for header_name in ["", "이름", "개수", "총 가격"] {
                header.col(|ui| {
                    ui.centered_and_justified(|ui| {
                        ui.label(header_name);
                    });
                });
            }
            header.col(|ui| {
                ui.centered_and_justified(|ui| {
                    let is_asc = &mut app.search_state.sort_asc;
                    let current_sort_icon = if *is_asc {
                        egui_phosphor::SORT_DESCENDING
                    } else {
                        egui_phosphor::SORT_ASCENDING
                    };
                    if ui
                        .button(format!("개당 가격 {current_sort_icon}"))
                        .clicked()
                    {
                        results.sort_by(|a, b| {
                            let mut order = a.unit_price.cmp(&b.unit_price);
                            if *is_asc {
                                order = order.reverse();
                            }
                            order
                        });
                        *is_asc = !*is_asc;
                    }
                });
            });
        })
        .body(|mut body| {
            fn right_align_row(
                row: &mut TableRow<'_, '_>,
                text: impl Into<WidgetText>,
            ) -> (egui::Rect, egui::Response) {
                row.col(|ui| {
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(text);
                    });
                })
            }

            body.ui_mut().style_mut().wrap = Some(false);

            for auction_row in results.iter() {
                let AuctionInfo {
                    count,
                    current_price,
                    unit_price,
                    item,
                    ..
                } = auction_row;
                body.row(30., |mut row| {
                    row.col(|ui| match app.images.get(&item.id) {
                        Some(image) => {
                            ui.centered_and_justified(|ui| {
                                image.show(ui);
                            });
                        }
                        None => {
                            app.images.request(item);
                            ui.spinner();
                        }
                    });

                    row.col(|ui| {
                        ui.centered_and_justified(|ui| {
                            ui.label(&item.name);
                        });
                    });

                    [count, current_price, unit_price]
                        .into_iter()
                        .map(Separable::separate_with_commas)
                        .for_each(|text| {
                            right_align_row(&mut row, text);
                        });
                });
            }
        });
}

fn show_error_label(ui: &mut Ui, text: &str) -> egui::Response {
    ui.colored_label(ui.visuals().error_fg_color, text)
}

// 한글 적용
fn key_pressed_hack(input_state: &InputState, pressed_key: Key) -> bool {
    input_state.events.iter().any(|event| {
        matches!(
            event,
            Event::Key {
                key,
                ..
            }
            if key == &pressed_key
        )
    })
}
