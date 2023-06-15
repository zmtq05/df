use std::collections::{HashMap, VecDeque};

use df_client::util::AsItem;
use egui_extras::RetainedImage;
use poll_promise::Promise;

use crate::spawn_promise;

#[derive(Default)]
pub struct ImageStorage {
    storage: HashMap<String, RetainedImage>,
    request_queue: VecDeque<Option<(String, Promise<crate::Result<RetainedImage>>)>>,
}

impl ImageStorage {
    pub fn get(&self, id: &str) -> Option<&RetainedImage> {
        self.storage.get(id)
    }

    pub fn request<T: AsItem>(&mut self, item: &T) {
        let id = item.id().to_owned();

        if self.in_queue(&id) {
            return;
        }

        let name = item.name().to_owned();
        let promise = spawn_promise(async move {
            df_client::instance()
                .image()
                ._item(&id)
                .await
                .map(|bytes| RetainedImage::from_image_bytes(&name, &bytes).unwrap())
        });

        self.request_queue
            .push_back(Some((item.id().to_string(), promise)));
    }

    fn in_queue(&self, id: &str) -> bool {
        self.request_queue
            .iter()
            .any(|o| o.as_ref().is_some_and(|(_id, _)| _id == id))
    }

    pub(crate) fn update_state(&mut self) {
        self.request_queue.retain(|o| o.is_some());

        for option in &mut self.request_queue {
            let (_, promise) = option.as_mut().unwrap();
            match promise.ready_mut() {
                Some(_) => {
                    let (id, promise) = option.take().unwrap();
                    self.storage.insert(id, promise.block_and_take().unwrap());
                }
                None => {
                    continue;
                }
            }
        }
    }
}
