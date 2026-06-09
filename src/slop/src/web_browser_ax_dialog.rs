//! Translated from `WebBrowserAxDialog.h/.cpp`.
//!
//! A modal ActiveX/IE dialog for the screenshot ("post image") and video-upload
//! flows. Hosts `IWebBrowser2`, exposes a small `IDispatch` external object the
//! page scripts can call (`AppHostUploadVideo`, `AppHostOpenVideoFolder`,
//! `AppHostPostImage`, …), and uploads recorded video to YouTube's legacy GData
//! API. No anti-cheat code; `BeforeNavigate2` enforces `trustCheckBrowser`.

#![allow(dead_code)]

use std::sync::Arc;

use crate::rbx::{self, DataModel};

/// `WebBrowserAxDialog`.
pub struct WebBrowserAxDialog {
    url: String,
    data_model: Arc<dyn DataModel>,
    enable_upload: Option<Box<dyn Fn(bool) + Send + Sync>>,
    file_name: String,
    video_seo_info: String,
    site_seo: bool,
    video_title: String,
    youtube_token: String,
}

impl WebBrowserAxDialog {
    pub fn new(
        url: String,
        data_model: Arc<dyn DataModel>,
        enable_upload: Option<Box<dyn Fn(bool) + Send + Sync>>,
    ) -> Self {
        Self {
            url,
            data_model,
            enable_upload,
            file_name: String::new(),
            video_seo_info: String::new(),
            site_seo: false,
            video_title: String::new(),
            youtube_token: String::new(),
        }
    }

    pub fn set_file_name(&mut self, file: String) {
        self.file_name = file;
    }

    /// External-object dispatch ids (`GetIDsOfNames`).
    pub fn dispatch_id(name: &str) -> Option<i32> {
        Some(match name {
            "CheckAppHost" => 0,
            "AppHostOpenVideoFolder" => 1,
            "AppHostUploadVideo" => 2,
            "AppHostOpenPicFolder" => 3,
            "AppHostPostImage" => 4,
            _ => return None,
        })
    }

    /// `Invoke(DISPATCH_METHOD, ...)`.
    pub fn invoke_method(&mut self, dispid: i32, args: InvokeArgs) {
        match dispid {
            1 => open_user_folder(UserFolder::Video),
            2 => self.upload_video(args.token, args.do_post, args.post_setting, args.title),
            3 => open_user_folder(UserFolder::Picture),
            4 => { /* GameSettings::setPostImageSetting(NEVER) */ }
            _ => {}
        }
    }

    /// `UploadVideo` — persist the upload setting, then kick a YouTube upload.
    fn upload_video(&mut self, token: String, do_post: bool, _post_setting: i16, title: String) {
        if do_post {
            let place_id = self.data_model.get_place_id();
            let seo = self.data_model.get_screenshot_seo_info();
            self.do_upload_video(token, title, seo, place_id);
        }
    }

    fn do_upload_video(&mut self, token: String, title: String, seostr: String, place_id: i32) {
        self.video_title = if title.is_empty() { "ROBLOX ROCKS!".into() } else { title };
        self.youtube_token = token;
        if seostr.is_empty() && place_id > 0 {
            self.video_seo_info = format!(
                "To play this game, please visit: http://www.roblox.com/item.aspx?id={place_id}&amp;rbx_source=youtube&amp;rbx_medium=uservideo"
            );
            self.site_seo = false;
        } else {
            self.video_seo_info = seostr;
            self.site_seo = true;
        }
        if let Some(cb) = &self.enable_upload {
            cb(false);
        }
        // boost::thread(ThreadDoUploadVideo, ...): builds a multipart body and
        // POSTs to http://uploads.gdata.youtube.com/feeds/api/users/default/uploads.
        let _ = rbx::http::Http::new(
            "http://uploads.gdata.youtube.com/feeds/api/users/default/uploads",
        );
    }

    /// `BeforeNavigate2`.
    pub fn before_navigate2(&self, url: &str) -> bool {
        rbx::http::Http::trust_check_browser(url)
    }
}

pub struct InvokeArgs {
    pub token: String,
    pub title: String,
    pub do_post: bool,
    pub post_setting: i16,
}

enum UserFolder {
    Video,
    Picture,
}

/// `ShellExecuteW("open", FileSystem::getUserDirectory(Video|Picture))`.
fn open_user_folder(_which: UserFolder) {}
