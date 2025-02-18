use std::io::Write;

use notan::egui::Context;

pub type SaveFn = Box<dyn FnOnce(&mut dyn Write) -> Result<(), anyhow::Error>>;

pub trait FileExport {
    fn set_save_action(
        &mut self,
        save: SaveFn,
        default_filename: &str,
    ) -> Result<(), anyhow::Error>;
    fn update(&mut self, ctx: &Context) -> Result<(), anyhow::Error>;
}

#[cfg(target_arch = "wasm32")]
pub use web::make_exporter;

#[cfg(not(target_arch = "wasm32"))]
pub use native::make_exporter;

#[cfg(target_arch = "wasm32")]
mod web {
    use super::{Context, FileExport, SaveFn};
    use anyhow::anyhow;
    use wasm_bindgen::{JsCast, JsValue};
    struct Export;

    pub fn make_exporter() -> Box<dyn FileExport> {
        Box::new(Export)
    }

    fn jsv(v: JsValue) -> anyhow::Error {
        anyhow!("{:?}", v)
    }

    fn np() -> anyhow::Error {
        anyhow!("error in javascript code")
    }

    impl FileExport for Export {
        fn set_save_action(
            &mut self,
            save: SaveFn,
            default_filename: &str,
        ) -> Result<(), anyhow::Error> {
            let mut cursor = std::io::Cursor::new(Vec::new());
            save(&mut cursor)?;
            let arr = js_sys::Uint8Array::new_with_length(cursor.get_ref().len() as u32);
            arr.copy_from(cursor.get_ref());
            let outer = js_sys::Array::new();
            outer.push(&arr.buffer());
            let blob = web_sys::Blob::new_with_u8_array_sequence(&outer).map_err(jsv)?;
            let url = web_sys::Url::create_object_url_with_blob(&blob).map_err(jsv)?;
            let doc = web_sys::window()
                .ok_or_else(np)?
                .document()
                .ok_or_else(np)?;
            let a: web_sys::HtmlAnchorElement = doc
                .create_element("a")
                .map_err(jsv)?
                .dyn_into()
                .map_err(|_| np())?;
            a.set_href(&url);
            a.set_download(default_filename);
            let body = doc.body().ok_or_else(np)?;
            body.append_child(&a).map_err(jsv)?;
            a.click();
            body.remove_child(&a).map_err(jsv)?;
            web_sys::Url::revoke_object_url(&url).map_err(jsv)?;
            Ok(())
        }

        fn update(&mut self, _ctx: &Context) -> Result<(), anyhow::Error> {
            Ok(())
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use std::{fs::File, io::BufWriter};

    use egui_file_dialog::FileDialog;

    use super::{Context, FileExport, SaveFn};

    pub struct Export {
        dialog: FileDialog,
        save: Option<SaveFn>,
    }

    pub fn make_exporter() -> Box<dyn FileExport> {
        Box::new(Export {
            dialog: FileDialog::new(),
            save: None,
        })
    }

    impl FileExport for Export {
        fn set_save_action(
            &mut self,
            save: SaveFn,
            _default_filename: &str,
        ) -> Result<(), anyhow::Error> {
            self.dialog.save_file();
            self.save = Some(save);
            Ok(())
        }

        fn update(&mut self, ctx: &Context) -> Result<(), anyhow::Error> {
            if let Some(path) = self.dialog.update(ctx).picked() {
                if let Some(save) = self.save.take() {
                    let mut save_file = BufWriter::new(File::create(path)?);
                    save(&mut save_file)?;
                }
            }
            Ok(())
        }
    }
}
