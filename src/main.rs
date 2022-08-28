use std::fs;
use std::io::Write;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::Arc;

use deno_runtime::{BootstrapOptions, deno_core};
use deno_runtime::deno_broadcast_channel::InMemoryBroadcastChannel;
use deno_runtime::deno_core::anyhow::Result;
use deno_runtime::deno_core::FsModuleLoader;
use deno_runtime::permissions::Permissions;
use deno_runtime::worker::{MainWorker, WorkerOptions};
use tokio::runtime::Builder;

use crate::deno_core::error::AnyError;

pub struct MainWorkerOptions(WorkerOptions);

impl MainWorkerOptions {
    pub fn into_inner(self) -> WorkerOptions {
        self.0
    }
}

impl Deref for MainWorkerOptions {
    type Target = WorkerOptions;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for MainWorkerOptions {
    fn default() -> Self {
        let create_web_worker_cb = Arc::new(|_| {
            panic!("Web workers are not supported in the example");
        });
        let web_worker_event_cb = Arc::new(|_| {
            panic!("Web workers are not supported in the example");
        });

        Self(WorkerOptions {
            bootstrap: BootstrapOptions {
                args: vec![],
                cpu_count: 1,
                debug_flag: false,
                enable_testing_features: false,
                location: None,
                no_color: false,
                is_tty: false,
                runtime_version: "x".to_string(),
                ts_version: "x".to_string(),
                unstable: false,
                user_agent: "my_runtime 0.1".to_string(),
            },
            extensions: vec![],
            unsafely_ignore_certificate_errors: None,
            root_cert_store: None,
            seed: None,
            module_loader: Rc::new(FsModuleLoader),
            create_web_worker_cb,
            web_worker_preload_module_cb: web_worker_event_cb,
            format_js_error_fn: None,
            source_map_getter: None,
            maybe_inspector_server: None,
            should_break_on_first_statement: false,
            get_error_class_fn: None,
            origin_storage_dir: None,
            blob_store: Default::default(),
            broadcast_channel: InMemoryBroadcastChannel::default(),
            shared_array_buffer_store: None,
            compiled_wasm_module_store: None,
            stdio: Default::default(),
        })
    }
}

fn main() -> Result<()> {
    let options = MainWorkerOptions::default().into_inner();
    let js_file = format!("{}/src/rest.js", env!("CARGO_MANIFEST_DIR"));
    let url = deno_core::resolve_url_or_path(js_file.as_str())?;
    let permissions = Permissions::allow_all();

    let rt = Builder::new_current_thread().enable_all().build()?;

    let fut = async move {
        let mut worker = MainWorker::bootstrap_from_options(
            url.clone(),
            permissions,
            options,
        );

        // let url = deno_core::resolve_url_or_path("D:\\works\\NetEaseCloudMusicLyric\\test.js").unwrap();
        // let (bundler, _) = deno_bundler::bundle(url.clone(), deno_bundler::BundleOptions::default()).await?;
        // let mut file = fs::File::create(format!("{}/src/rest.js", env!("CARGO_MANIFEST_DIR")))?;
        // let mut file = fs::File::create(format!("{}/src/test.js", env!("CARGO_MANIFEST_DIR")))?;
        // file.write(bundler.as_bytes())?;

        worker.execute_main_module(&url).await?;
        worker.run_event_loop(false).await?;
        Ok::<_, AnyError>(())
    };

    let local = tokio::task::LocalSet::new();
    local.block_on(&rt, fut)?;

    Ok(())
}
