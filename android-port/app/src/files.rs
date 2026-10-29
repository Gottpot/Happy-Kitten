//! Bridge to the Android activity: file picker, saving into the gallery.
//!
//! Android hands results back later (`onActivityResult`), so nothing here blocks. The activity
//! (`java/de/gottpot/happykitten/MainActivity.java`) copies picked files into the app's own folder
//! and calls the `native*` functions below; the app collects the answers with [`poll`] once per
//! frame. This is the only file of the Android app that talks JNI.

use jni::objects::{JClass, JObject, JObjectArray, JString, JValue};
use jni::sys::{jboolean, jint};
use jni::{JNIEnv, JavaVM};
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock, PoisonError};

/// What a picked file is for; sent to the activity and back so answers find their way.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Purpose {
    Import = 0,
    Subtitles = 1,
    Project = 2,
    Graphics = 3,
}

impl Purpose {
    fn from_i32(v: i32) -> Option<Purpose> {
        [
            Purpose::Import,
            Purpose::Subtitles,
            Purpose::Project,
            Purpose::Graphics,
        ]
        .into_iter()
        .find(|p| *p as i32 == v)
    }
}

pub enum Event {
    Picked(Purpose, Vec<PathBuf>),
    /// The activity finished copying an export into the gallery (or failed to).
    Saved { ok: bool, message: String },
}

static QUEUE: Mutex<VecDeque<Event>> = Mutex::new(VecDeque::new());
static CTX: OnceLock<egui::Context> = OnceLock::new();
static APP: OnceLock<android_activity::AndroidApp> = OnceLock::new();

/// Remembers the activity; called once from `android_main`.
pub fn init_app(app: &android_activity::AndroidApp) {
    let _ = APP.set(app.clone());
}

pub fn init(ctx: &egui::Context) {
    let _ = CTX.set(ctx.clone());
}

fn push(ev: Event) {
    QUEUE
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .push_back(ev);
    if let Some(ctx) = CTX.get() {
        ctx.request_repaint();
    }
}

/// Everything that arrived since the last call.
pub fn poll() -> Vec<Event> {
    QUEUE
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .drain(..)
        .collect()
}

/// Runs `f` with the JNI environment and the activity object of this process.
fn with_activity<R>(
    f: impl FnOnce(&mut JNIEnv, &JObject) -> jni::errors::Result<R>,
) -> Option<R> {
    let app = APP.get()?;
    // SAFETY: android-activity owns the JavaVM and keeps a global reference to the activity;
    // both live as long as `app`, which sits in a static.
    let vm = unsafe { JavaVM::from_raw(app.vm_as_ptr().cast()) }.ok()?;
    let mut env = vm.attach_current_thread().ok()?;
    // SAFETY: see above. The wrapper does not own the reference and never deletes it.
    let activity = unsafe { JObject::from_raw(app.activity_as_ptr().cast()) };
    match f(&mut env, &activity) {
        Ok(v) => Some(v),
        Err(e) => {
            log::error!("JNI: {e}");
            let _ = env.exception_describe();
            let _ = env.exception_clear();
            None
        }
    }
}

/// Window insets in pixels (top, bottom, left, right), asked from the activity.
pub fn insets_px() -> Option<[i32; 4]> {
    with_activity(|env, activity| {
        let arr = env
            .call_method(activity, "getInsets", "()[I", &[])?
            .l()?;
        let arr = jni::objects::JIntArray::from(arr);
        let mut out = [0i32; 4];
        env.get_int_array_region(&arr, 0, &mut out)?;
        Ok(out)
    })
}

/// A short buzz, the "it took hold" feedback of a long press.
pub fn haptic() {
    with_activity(|env, activity| {
        env.call_method(activity, "haptic", "()V", &[])?;
        Ok(())
    });
}

/// Opens the system file picker. `mimes` like `["video/*", "image/*"]`; answers arrive as
/// [`Event::Picked`] (an empty list when the user cancels).
pub fn pick(purpose: Purpose, mimes: &[&str], multiple: bool) {
    with_activity(|env, activity| {
        let string = env.find_class("java/lang/String")?;
        let array = env.new_object_array(mimes.len() as i32, string, JObject::null())?;
        for (i, m) in mimes.iter().enumerate() {
            let s = env.new_string(m)?;
            env.set_object_array_element(&array, i as i32, s)?;
        }
        env.call_method(
            activity,
            "pick",
            "(I[Ljava/lang/String;Z)V",
            &[
                JValue::Int(purpose as jint),
                JValue::Object(&array),
                JValue::Bool(multiple as jboolean),
            ],
        )?;
        Ok(())
    });
}

/// Copies `path` into the gallery (Movies/Happy-Kitten or Pictures/Happy-Kitten); the answer
/// arrives as [`Event::Saved`].
pub fn save_to_gallery(path: &std::path::Path, display_name: &str, video: bool) {
    let path = path.to_string_lossy().into_owned();
    let ok = with_activity(|env, activity| {
        let p = env.new_string(&path)?;
        let n = env.new_string(display_name)?;
        env.call_method(
            activity,
            "saveToGallery",
            "(Ljava/lang/String;Ljava/lang/String;Z)V",
            &[
                JValue::Object(&p),
                JValue::Object(&n),
                JValue::Bool(video as jboolean),
            ],
        )?;
        Ok(())
    });
    if ok.is_none() {
        push(Event::Saved {
            ok: false,
            message: "Die Galerie ist nicht erreichbar".into(),
        });
    }
}

fn java_string(env: &mut JNIEnv, s: &JString) -> String {
    env.get_string(s).map(Into::into).unwrap_or_default()
}

// The two functions are looked up by name by the Java VM (`native` methods of MainActivity).
#[unsafe(no_mangle)]
pub extern "system" fn Java_de_gottpot_happykitten_MainActivity_nativePicked(
    mut env: JNIEnv,
    _class: JClass,
    purpose: jint,
    paths: JObjectArray,
) {
    let Some(purpose) = Purpose::from_i32(purpose) else {
        return;
    };
    let n = env.get_array_length(&paths).unwrap_or(0);
    let mut out = Vec::new();
    for i in 0..n {
        if let Ok(obj) = env.get_object_array_element(&paths, i) {
            let s = JString::from(obj);
            out.push(PathBuf::from(java_string(&mut env, &s)));
        }
    }
    push(Event::Picked(purpose, out));
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_de_gottpot_happykitten_MainActivity_nativeSaved(
    mut env: JNIEnv,
    _class: JClass,
    ok: jboolean,
    message: JString,
) {
    let message = java_string(&mut env, &message);
    push(Event::Saved {
        ok: ok != 0,
        message,
    });
}
